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
	//
	//value := val.GetPubKey().Address()
	//
	//fmt.Println(value)
	//
	//data := "ABC"
	//
	//res, err := val.SignData([]byte(data))
	//if err != nil {
	//	panic(err)
	//}
	//
	//pk := val.GetPubKey()
	//result := pk.VerifyBytes([]byte(data), res)
	//fmt.Print("Verifying signature.. ")
	//fmt.Println(result)

	logger := tmlog.NewTMLogger(
		tmlog.NewSyncWriter(os.Stdout),
	).With("module", "validator")

	var keyImport = flag.String("import", "", "path to key file")
	var keyExport = flag.String("export", "", "path to key file")
	var configFile = flag.String("config", "", "path to configuration file")
	flag.Parse()

	if *keyImport != "" {
		signer.ImportKey(*val, *keyImport)
		return
	}

	if *keyExport != "" {
		signer.ExportKey(*val, *keyExport)
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
