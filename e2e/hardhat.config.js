require("@nomicfoundation/hardhat-toolbox");
require("./scripts/sendPacket");

const mnemonic =
  "math razor capable expose worth grape metal sunset metal sudden usage scheme";

/**
 * @type import('hardhat/config').HardhatUserConfig
 */
module.exports = {
  solidity: {
    version: "0.8.28",
    settings: {
      optimizer: {
        enabled: true,
        runs: 9_999_999
      }
    },
  },
  networks: {
    chain0: {
      chainId: 2018,
      url: "http://127.0.0.1:8545",
      accounts: {
        mnemonic: mnemonic,
        path: "m/44'/60'/0'/0",
        initialIndex: 0,
        count: 10
      }
    },
    chain1: {
      chainId: 3018,
      url: "http://127.0.0.1:8645",
      accounts: {
        mnemonic: mnemonic,
        path: "m/44'/60'/0'/0",
        initialIndex: 0,
        count: 10
      }
    }
  }
}
