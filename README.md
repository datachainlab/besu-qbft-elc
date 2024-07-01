# besu-qbft-elc

[![test](https://github.com/datachainlab/besu-qbft-elc/actions/workflows/test.yml/badge.svg?branch=main)](https://github.com/datachainlab/besu-qbft-elc/actions/workflows/test.yml)

This repository provides a light client for the Hyperledger Besu's QBFT consensus algorithm. The light client is implemented as the Enclave Light Client (ELC) for [LCP](https://github.com/datachainlab/lcp).

## Replated Repositories

- [ibc-solidity](https://github.com/hyperledger-labs/yui-ibc-solidity): An IBC implementation in Solidity.
- [yui-relayer](https://github.com/hyperledger-labs/yui-relayer): A relayer implementation for the IBC protocol, which also supports heterogeneous blockchains.
- [besu-ibc-relay-prover](https://github.com/datachainlab/besu-ibc-relay-prover): The relayer's prover module for the Hyperledger Besu.

## E2E Test

**Prerequisites: Please check [the github actions workflow file](.github/workflows/test.yml) for the required dependencies.**

You can run the e2e relay test by running the following commands:

```bash
make -C e2e e2e-test
```
