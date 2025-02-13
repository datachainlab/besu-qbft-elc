{
  "chain": {
    "@type": "/relayer.chains.ethereum.config.ChainConfig",
    "chain_id": "ibc0",
    "eth_chain_id": 2018,
    "rpc_addr": "http://localhost:8545",
    "signer": {
      "@type": "/relayer.signers.hd.SignerConfig",
      "mnemonic": "math razor capable expose worth grape metal sunset metal sudden usage scheme",
      "path": "m/44'/60'/0'/0/0"
    },
    "ibc_address": "$IBC_ADDRESS",
    "initial_send_checkpoint": 1,
    "initial_recv_checkpoint": 1,
    "enable_debug_trace": false,
    "average_block_time_msec": 1000,
    "max_retry_for_inclusion": 5,
    "gas_estimate_rate": {
      "numerator": 1,
      "denominator": 1
    },
    "max_gas_limit": 10000000,
    "tx_type": "legacy",
    "abi_paths": ["./abis"],
    "allow_lc_functions": {
      "lc_address": "$LCP_CLIENT_ADDRESS",
      "allow_all": false,
      "selectors": [
        "0xa97c61d6",
        "0x6ac73aa0"
      ]
    }
  },
  "prover": {
    "@type": "/relayer.provers.lcp.config.ProverConfig",
    "origin_prover": {
      "@type": "/relayer.provers.qbft.config.ProverConfig",
      "consensus_type": "qbft",
      "trusting_period": "336h",
      "max_clock_drift": "30s"
    },
    "lcp_service_address": "localhost:50051",
    "mrenclave": "$MRENCLAVE",
    "allowed_quote_statuses": ["GROUP_OUT_OF_DATE"],
    "allowed_advisory_ids": ["INTEL-SA-00219","INTEL-SA-00289","INTEL-SA-00334","INTEL-SA-00477","INTEL-SA-00614","INTEL-SA-00615","INTEL-SA-00617", "INTEL-SA-00828"],
    "key_expiration": 604800,
    "elc_client_id": "hb-qbft-0",
    "message_aggregation": true,
    "is_debug_enclave": true
  }
}
