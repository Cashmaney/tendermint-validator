package main

import (
	"encoding/base64"
	"flag"
	"fmt"
	"github.com/tendermint/tendermint/crypto/ed25519"
	tmlog "github.com/tendermint/tendermint/libs/log"
	tos "github.com/tendermint/tendermint/libs/os"
	svc "github.com/tendermint/tendermint/libs/service"
	"io/ioutil"
	"log"
	"net"
	"os"
	"path"
	"sync"
	"tendermint-signer/internal/signer"
	teeval "tendermint-signer/tee_validator"
	"time"
)

const (
	PrivateKeyLength = 64
)

func fileExists(filename string) bool {
	info, err := os.Stat(filename)
	if os.IsNotExist(err) {
		return false
	}
	return !info.IsDir()
}

func openPrivateKeyFile(filename string) (ed25519.PrivKeyEd25519, error) {
	cert, err := ioutil.ReadFile(filename)
	if err != nil {
		return ed25519.PrivKeyEd25519{}, err
	}
	text := string(cert)

	key, err := base64.StdEncoding.DecodeString(text)
	if err != nil {
		return ed25519.PrivKeyEd25519{}, err
	}

	if len(key) != PrivateKeyLength {
		panic(fmt.Sprintf("Tried to import key with an invalid key size %d, Expected 64", len(key)))
	}

	//privKey := ed25519.PrivKeyEd25519{
	//
	//}
	//
	//privKey := goed25519.NewKeyFromSeed(key)
	//
	var privKey ed25519.PrivKeyEd25519
	copy(privKey[:], key)

	return privKey, nil
}

func savePrivateKeyFile(filename string, privKey ed25519.PrivKeyEd25519) error {

	fmt.Println(len(privKey[:]))

	byteB64 := base64.StdEncoding.EncodeToString(privKey[:])

	err := ioutil.WriteFile(filename, []byte(byteB64), 0644)
	if err != nil {
		return err
	}

	return nil
}

func main() {
	val := &teeval.EnclavePV{}

	value := val.GetPubKey().Address()

	fmt.Println(value)

	data := "ABC"

	res, err := val.SignData([]byte(data))
	if err != nil {
		panic(err)
	}

	pk := val.GetPubKey()

	result := pk.VerifyBytes([]byte(data), res)
	fmt.Print("Verifying signature.. ")
	fmt.Println(result)

	logger := tmlog.NewTMLogger(
		tmlog.NewSyncWriter(os.Stdout),
	).With("module", "validator")

	var keyImport = flag.String("import", "", "path to key file")
	var keyExport = flag.String("export", "", "path to key file")
	var configFile = flag.String("config", "", "path to configuration file")
	flag.Parse()

	if *keyImport != "" {
		privKey, err := openPrivateKeyFile(*keyImport)
		if err != nil {
			panic(err)
		}
		err = val.ImportKey(privKey)
		if err != nil {
			panic(err)
		}
		fmt.Println(fmt.Sprintf("Imported key successfully!"))
		return
	}

	if *keyExport != "" {
		res, err := val.ExportKey([]byte(""))
		if err != nil {
			panic(err)
		}
		err = savePrivateKeyFile(*keyExport, res)
		if err != nil {
			panic(err)
		}

		fmt.Println(fmt.Sprintf("Exported key successfully!"))
		return
	}

	if *configFile == "" {
		panic("--config flag is required")
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
