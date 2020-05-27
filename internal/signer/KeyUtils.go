package signer

import (
	"encoding/base64"
	"encoding/json"
	"fmt"
	"github.com/tendermint/tendermint/crypto/ed25519"
	"github.com/tendermint/tendermint/libs/bech32"
	"golang.org/x/crypto/ssh/terminal"
	"io/ioutil"
	"log"
	"strings"
	teeval "tendermint-signer/tee_validator"
)

const (
	PrivateKeyLength = 64
)

func GetValidaorAddress(val teeval.EnclavePV, chainId string) {
	pubkey := val.GetPubKey()

	prefix := strings.Split(chainId, "-")[0] + "val" + "oper"

	result, err := bech32.ConvertAndEncode(prefix, pubkey.Address())
	if err != nil {
		log.Fatal(err)
	}

	fmt.Printf("validator-address: %s\n", result)
}

func GenerateKey(val teeval.EnclavePV, password string) {
	if password == "" {
		fmt.Print("Enter Password: ")
		bytePassword, err := terminal.ReadPassword(0)
		if err != nil {
			log.Fatal(err)
		}

		fmt.Print("\nRepeat Password: ")
		bytePassword2, err := terminal.ReadPassword(0)
		if err != nil {
			log.Fatal(err)
		}
		fmt.Print("\n")
		if string(bytePassword) != string(bytePassword2) {
			log.Fatal("Passwords do not match")
		}
		if len(bytePassword2) == 0 {
			log.Fatal("Password cannot be empty")
		}
		password = string(bytePassword)
	}

	err := val.GenerateKey([]byte(password))
	if err != nil {
		log.Fatal(err)
	}
	fmt.Println(fmt.Sprintf("Key generated successfully. You can now use --address to see your address"))
	return
}

func ImportKey(val teeval.EnclavePV, importPath string, password string) {
	if password == "" {
		fmt.Print("Enter Password: ")
		bytePassword, err := terminal.ReadPassword(0)
		if err != nil {
			log.Fatal(err)
		}

		fmt.Print("\nRepeat Password: ")
		bytePassword2, err := terminal.ReadPassword(0)
		if err != nil {
			log.Fatal(err)
		}
		fmt.Print("\n")
		if string(bytePassword) != string(bytePassword2) {
			log.Fatal("Passwords do not match")
		}
		if len(bytePassword2) == 0 {
			log.Fatal("Password cannot be empty")
		}
		password = string(bytePassword)
	}
	privKey, err := openPrivateKeyFile(importPath)
	if err != nil {
		log.Fatal(err)
	}
	err = val.ImportKey(privKey, []byte(password))
	if err != nil {
		log.Fatal(err)
	}
	fmt.Println(fmt.Sprintf("Imported key successfully!"))
	return
}

func ExportKey(val teeval.EnclavePV, exportPath string, password string) {
	if password == "" {
		fmt.Print("Enter Password: ")
		bytePassword, err := terminal.ReadPassword(0)
		if err != nil {
			log.Fatal(err)
		}
		password = string(bytePassword)
	}
	res, err := val.ExportKey([]byte(password))
	if err != nil {
		log.Fatal(err)
	}
	err = savePrivateKeyFile(exportPath, res)
	if err != nil {
		log.Fatal(err)
	}

	fmt.Println(fmt.Sprintf("Exported key successfully!"))
	return
}

func savePrivateKeyFile(filename string, privKey ed25519.PrivKeyEd25519) error {

	fmt.Println(len(privKey[:]))

	var keyFile CosmosValidatorKey

	var pubkeyBytes [32]byte

	copy(pubkeyBytes[:], privKey[32:])

	pubKey := base64.StdEncoding.EncodeToString(pubkeyBytes[:])
	byteB64 := base64.StdEncoding.EncodeToString(privKey[:])
	address := privKey.PubKey().Address().String()

	keyFile.PrivKey = TypeVal{
		Type:  "tendermint/PrivKeyEd25519",
		Value: byteB64,
	}
	keyFile.PubKey = TypeVal{
		Type:  "tendermint/PrivKeyEd25519",
		Value: pubKey,
	}
	keyFile.Address = address

	jsonBytes, err := json.Marshal(keyFile)

	err = ioutil.WriteFile(filename, jsonBytes, 0644)
	if err != nil {
		return err
	}

	return nil
}

func openPrivateKeyFile(filename string) (ed25519.PrivKeyEd25519, error) {
	keyFile, err := ioutil.ReadFile(filename)
	if err != nil {
		return ed25519.PrivKeyEd25519{}, err
	}

	// Try to parse a cosmos private validator key
	privKeyFile := parseCosmosKeyFile(keyFile)
	if privKeyFile == "" {
		// if that fails, just try the base64 string
		privKeyFile = string(keyFile)
	}

	key, err := base64.StdEncoding.DecodeString(privKeyFile)
	if err != nil {
		return ed25519.PrivKeyEd25519{}, err
	}

	if len(key) != PrivateKeyLength {
		panic(fmt.Sprintf("Tried to import key with an invalid key size %d, Expected 64", len(key)))
	}

	var privKey ed25519.PrivKeyEd25519
	copy(privKey[:], key)

	return privKey, nil
}

type TypeVal struct {
	Type  string `json:"type"`
	Value string `json:"value"`
}

// User struct which contains a name
// a type and a list of social links
type CosmosValidatorKey struct {
	Address string  `json:"address"`
	PubKey  TypeVal `json:"pub_key"`
	PrivKey TypeVal `json:"priv_key"`
}

func parseCosmosKeyFile(file []byte) string {
	var keyFile CosmosValidatorKey
	err := json.Unmarshal(file, &keyFile)
	if err != nil {
		return ""
	}

	if keyFile.PrivKey.Type != "tendermint/PrivKeyEd25519" {
		log.Fatalf("Unsupported key type %s - must be Ed25519", keyFile.PrivKey.Type)
	}

	return keyFile.PrivKey.Value
}
