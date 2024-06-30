#!/bin/bash
set -ex

RELAYER_DIR=$(dirname $(dirname "$0"))
LCP_BIN=${LCP_BIN:-lcp}

if [ $# -ne 1 ]; then
  echo "Usage: $0 <relayer-chain-id>" >&2
  exit 1
fi
CHAINID=$1

if [ -z "${IBC_HANDLER}" ]; then
  echo "Error: env var 'IBC_HANDLER' is not set." >&2
  exit 1
fi

if [ -z "${LCP_CLIENT}" ]; then
  echo "Error: env var 'LCP_CLIENT' is not set." >&2
  exit 1
fi

export MRENCLAVE=$(${LCP_BIN} enclave metadata --enclave=./bin/enclave.signed.so | jq -r .mrenclave)

TEMPLATE_DIR=${RELAYER_DIR}/configs/templates
CONFIG_DIR=${RELAYER_DIR}/configs/demo
mkdir -p $CONFIG_DIR
jq '.chain.ibc_address = env.IBC_HANDLER | .chain.allow_lc_functions.lc_address = env.LCP_CLIENT | .prover.mrenclave = env.MRENCLAVE' ${TEMPLATE_DIR}/${CHAINID}.json.tpl > ${CONFIG_DIR}/${CHAINID}.json
