package main

import (
	"log"

	"github.com/datachainlab/besu-ibc-relay-prover/module"
	"github.com/datachainlab/ethereum-ibc-relay-chain/pkg/relay/ethereum"
	"github.com/datachainlab/ethereum-ibc-relay-chain/pkg/relay/ethereum/signers/hd"
	lcp "github.com/datachainlab/lcp-go/relay"
	"github.com/hyperledger-labs/yui-relayer/cmd"
)

func main() {
	if err := cmd.Execute(
		ethereum.Module{},
		hd.Module{},
		module.Module{},
		lcp.Module{},
	); err != nil {
		log.Fatal(err)
	}
}
