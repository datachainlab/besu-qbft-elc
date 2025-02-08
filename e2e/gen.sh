#!/bin/bash -eu

TO=networkFiles
NETWORK=chains/chain1/network

mkdir -p $NETWORK
cp $TO/genesis.json $NETWORK

declare -i idx=0
for k in $TO/keys/0x*; do
  dst=$NETWORK/Node-${idx}/data
  mkdir -p $dst
  cp $k/key{,.pub} $dst
  ((idx++)) || true
done

exit 0
