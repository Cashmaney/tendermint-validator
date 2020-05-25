package main

import (
	"flag"
	"fmt"
	tmlog "github.com/tendermint/tendermint/libs/log"
	tos "github.com/tendermint/tendermint/libs/os"
	svc "github.com/tendermint/tendermint/libs/service"
	"log"
	"net"
	"os"
	"path"
	"sync"
	"tendermint-signer/internal/signer"
	teeval "tendermint-signer/tee_validator"
	"time"
)

func fileExists(filename string) bool {
	info, err := os.Stat(filename)
	if os.IsNotExist(err) {
		return false
	}
	return !info.IsDir()
}

func main() {

	val := &teeval.EnclavePV{}

	logger := tmlog.NewTMLogger(
		tmlog.NewSyncWriter(os.Stdout),
	).With("module", "validator")

	logger.Info("Verifying enclave...")
	err := val.HealthCheckEnclave()

	if err != nil {
		// ignore this for now...?
		if err.Error() == "SGX_ERROR_NO_DEVICE" {
			log.Fatal(fmt.Sprintf("Error starting enclave - No SGX device recognized. Is SGX properly installed?"))
		} else if err.Error() == "SGX_ERROR_UNEXPECTED" {
			log.Fatal(fmt.Sprintf("Error starting enclave - Unexpected error while starting enclave. Is SGX properly installed?"))
		}
	}
	logger.Info("Enclave running")

	var keyImport = flag.String("import", "", "Path to imported file")
	var keyExport = flag.String("export", "", "path to exported file")
	// var ValidatorAddress = flag.Bool("validator-address", true, "Show validator address")
	var Password = flag.String("password", "", "Set password without prompt. Used to set password without terminal interaction")
	var configFile = flag.String("config", "", "path to configuration file")
	flag.Parse()

	if *keyImport != "" {
		signer.ImportKey(*val, *keyImport, *Password)
		return
	}

	if *keyExport != "" {
		signer.ExportKey(*val, *keyExport, *Password)
		return
	}

	if *configFile == "" {
		*configFile = os.ExpandEnv("$HOME/.signer/config/config.toml")
	}

	config, err := signer.LoadConfigFromFile(*configFile)
	if err != nil {
		log.Fatal(err)
	}

	logger.Info(
		"Tendermint Validator",
		"priv-state-dir", config.PrivValStateDir,
	)

	if !teeval.IsHwSgxMode() {
		logger.Info("Warning: Running in software mode. This is for testing purposes only.")
	}

	signer.InitSerialization()

	// services to stop on shutdown
	var services []svc.Service

	chainID := config.ChainID
	if chainID == "" {
		log.Fatal("chain_id option is required")
	}

	stateFile := path.Join(config.PrivValStateDir, fmt.Sprintf("%s_priv_validator_state.json", chainID))

	if !fileExists(stateFile) {
		log.Fatalf("State file missing: %s\n", stateFile)
	}

	pv := &signer.PvGuard{PrivValidator: val}

	for _, node := range config.Nodes {
		dialer := net.Dialer{Timeout: 30 * time.Second}
		tsigner := signer.NewNodeClient(node.Address, logger, config.ChainID, pv, dialer)

		err := tsigner.Start()
		if err != nil {
			panic(err)
		}

		services = append(services, tsigner)
	}

	wg := sync.WaitGroup{}
	wg.Add(1)
	tos.TrapSignal(logger, func() {
		for _, service := range services {
			err := service.Stop()
			if err != nil {
				panic(err)
			}
		}
		wg.Done()
	})
	wg.Wait()
}
