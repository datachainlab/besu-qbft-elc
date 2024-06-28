#!/usr/bin/env bash
set -ex

export LCP_ENCLAVE_DEBUG=1

ENCLAVE_PATH=./bin/enclave.signed.so
LCP_BIN=${LCP_BIN:-lcp}
RLY_BIN=${RLY_BIN:-./bin/yrly}
export RLY_BIN
CERTS_DIR=./certs

./scripts/init_lcp.sh
source ./chain0.env.sh && ./relayer/scripts/gen-config.sh ibc0
source ./chain1.env.sh && ./relayer/scripts/gen-config.sh ibc1

if [ "$SGX_MODE" = "SW" ]; then
    export LCP_RA_ROOT_CERT_HEX=$(cat ${CERTS_DIR}/root.crt | xxd -p -c 1000000)
fi

make network
sleep 3
make deploy-contracts

${LCP_BIN} --log_level=info service start --enclave=${ENCLAVE_PATH} --address=127.0.0.1:50051 --threads=2 &
LCP_PID=$!

./relayer/scripts/init-rly
./relayer/scripts/handshake
./relayer/scripts/relay

kill $LCP_PID
