package signer

import (
	"fmt"
	"net"
	"time"

	"github.com/tendermint/tendermint/crypto/ed25519"
	cmn "github.com/tendermint/tendermint/libs/common"
	"github.com/tendermint/tendermint/libs/log"
	p2pconn "github.com/tendermint/tendermint/p2p/conn"
	"github.com/tendermint/tendermint/privval"
	"github.com/tendermint/tendermint/types"
)

// NodeClient dials a node responds to signature requests using its privVal.
type NodeClient struct {
	cmn.BaseService

	address string
	chainID string
	privKey ed25519.PrivKeyEd25519
	privVal types.PrivValidator

	dialer net.Dialer
}

// NewNodeClient return a NodeClient that will dial using the given
// dialer and respond to any signature requests over the connection
// using the given privVal.
//
// If the connection is broken, the NodeClient will attempt to reconnect.
func NewNodeClient(
	address string,
	logger log.Logger,
	chainID string,
	privVal types.PrivValidator,
	dialer net.Dialer,
) *NodeClient {
	rs := &NodeClient{
		address: address,
		chainID: chainID,
		privVal: privVal,
		dialer:  dialer,
		privKey: ed25519.GenPrivKey(),
	}

	rs.BaseService = *cmn.NewBaseService(logger, "RemoteSigner", rs)
	return rs
}

// OnStart implements cmn.Service.
func (rs *NodeClient) OnStart() error {
	go rs.loop()
	return nil
}

// main loop for NodeClient
func (rs *NodeClient) loop() {
	var conn net.Conn
	for {
		if !rs.IsRunning() {
			if conn != nil {
				if err := conn.Close(); err != nil {
					rs.Logger.Error("Close", "err", cmn.ErrorWrap(err, "closing listener failed"))
				}
			}
			return
		}

		for conn == nil {
			proto, address := cmn.ProtocolAndAddress(rs.address)
			netConn, err := rs.dialer.Dial(proto, address)
			if err != nil {
				rs.Logger.Error("Dialing", "err", err)
				rs.Logger.Info("Retrying", "sleep (s)", 3, "address", rs.address)
				time.Sleep(time.Second * 3)
				continue
			}

			rs.Logger.Info("Connected", "address", rs.address)
			conn, err = p2pconn.MakeSecretConnection(netConn, rs.privKey)
			if err != nil {
				conn = nil
				rs.Logger.Error("Secret Conn", "err", err)
				rs.Logger.Info("Retrying", "sleep (s)", 3, "address", rs.address)
				time.Sleep(time.Second * 3)
				continue
			}
		}

		// since dialing can take time, we check running again
		if !rs.IsRunning() {
			if err := conn.Close(); err != nil {
				rs.Logger.Error("Close", "err", cmn.ErrorWrap(err, "closing listener failed"))
			}
			return
		}

		req, err := ReadMsg(conn)
		if err != nil {
			rs.Logger.Error("readMsg", "err", err)
			conn.Close()
			conn = nil
			continue
		}

		res, err := rs.handleRequest(req)
		if err != nil {
			// only log the error; we'll reply with an error in res
			rs.Logger.Error("handleRequest", "err", err)
		}

		err = WriteMsg(conn, res)
		if err != nil {
			rs.Logger.Error("writeMsg", "err", err)
			conn.Close()
			conn = nil
		}
	}
}

func (rs *NodeClient) handleRequest(req privval.RemoteSignerMsg) (privval.RemoteSignerMsg, error) {
	var res privval.RemoteSignerMsg
	var err error

	switch typedReq := req.(type) {
	case *privval.PubKeyRequest:
		pubKey := rs.privVal.GetPubKey()
		res = &privval.PubKeyResponse{PubKey: pubKey, Error: nil}
	case *privval.SignVoteRequest:
		err = rs.privVal.SignVote(rs.chainID, typedReq.Vote)
		if err != nil {
			rs.Logger.Error("Failed to sign vote", "address", rs.address, "error", err, "vote", typedReq.Vote)
			res = &privval.SignedVoteResponse{
				Vote: nil,
				Error: &privval.RemoteSignerError{
					Code:        0,
					Description: err.Error(),
				},
			}
		} else {
			rs.Logger.Info("Signed vote", "address", rs.address, "vote", typedReq.Vote)
			res = &privval.SignedVoteResponse{Vote: typedReq.Vote, Error: nil}
		}
	case *privval.SignProposalRequest:
		err = rs.privVal.SignProposal(rs.chainID, typedReq.Proposal)
		if err != nil {
			rs.Logger.Error("Failed to sign proposal", "address", rs.address, "error", err, "proposal", typedReq.Proposal)
			res = &privval.SignedProposalResponse{
				Proposal: nil,
				Error: &privval.RemoteSignerError{
					Code:        0,
					Description: err.Error(),
				},
			}
		} else {
			rs.Logger.Info("Signed proposal", "address", rs.address, "proposal", typedReq.Proposal)
			res = &privval.SignedProposalResponse{
				Proposal: typedReq.Proposal,
				Error:    nil,
			}
		}
	case *privval.PingRequest:
		res = &privval.PingResponse{}
	default:
		err = fmt.Errorf("unknown msg: %v", typedReq)
	}

	return res, err
}
