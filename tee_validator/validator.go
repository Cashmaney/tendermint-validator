package tee_validator

import "C"
import (
	// "fmt"
	"github.com/tendermint/tendermint/crypto"
	"github.com/tendermint/tendermint/crypto/ed25519"
	tmtypes "github.com/tendermint/tendermint/types"
	goed25519 "golang.org/x/crypto/ed25519"
	"log"
	"os"
	"tendermint-signer/tee_validator/go-bridge/api"
)

type EnclavePV struct {
	pubKey crypto.PubKey
}

type PV interface {
	GetPubKey() crypto.PubKey
	SignVote(chainID string, vote *tmtypes.Vote) error
	SignProposal(chainID string, proposal *tmtypes.Proposal) error
}

// Ensure that we implement the PrivateValidator interface
var _ PV = &EnclavePV{}

func buildPubKey(inputBytes []byte) crypto.PubKey {
	var pubkeyBytes [ed25519.PubKeyEd25519Size]byte
	copy(pubkeyBytes[:], inputBytes[:])
	return ed25519.PubKeyEd25519(pubkeyBytes)
}

func (pv *EnclavePV) GetPubKey() crypto.PubKey {
	res, err := api.GetPubKey()
	if err != nil {
		log.Fatal(err)
	}
	pv.pubKey = buildPubKey(res)
	return pv.pubKey
}

func (pv *EnclavePV) SignVote(chainID string, vote *tmtypes.Vote) error {
	signBytes := vote.SignBytes(chainID)
	sig, err := api.Sign(signBytes)
	if err != nil {
		return err
	}
	vote.Signature = sig
	return nil
}

func (pv *EnclavePV) SignProposal(chainID string, proposal *tmtypes.Proposal) error {
	signBytes := proposal.SignBytes(chainID)
	sig, err := api.Sign(signBytes)
	if err != nil {
		return err
	}
	proposal.Signature = sig
	return nil
}

func (pv *EnclavePV) ImportKey(inputKey ed25519.PrivKeyEd25519, password []byte) error {

	err := api.Import(inputKey[:32], password)
	if err != nil {
		return err
	}
	return nil
}

func (pv *EnclavePV) ExportKey(password []byte) (ed25519.PrivKeyEd25519, error) {
	res, err := api.Export(password)
	if err != nil {
		return ed25519.PrivKeyEd25519{}, err
	}

	privKey := goed25519.NewKeyFromSeed(res)

	var privKeyEd ed25519.PrivKeyEd25519
	copy(privKeyEd[:], privKey)

	return privKeyEd, nil
}

func (pv *EnclavePV) SignData(data []byte) ([]byte, error) {
	res, err := api.Sign(data)
	if err != nil {
		return nil, err
	}

	return res, nil
}

func (pv *EnclavePV) GenerateKey(password []byte) error {
	err := api.Generate(password)
	if err != nil {
		return err
	}

	return nil
}

func (pv *EnclavePV) HealthCheckEnclave() error {
	err := api.CheckEnclave()
	if err != nil {
		return err
	}
	return nil
}

func IsHwSgxMode() bool {
	return os.Getenv("SGX_MODE") != "SW"
}
