const portMock = "mockapp";
const lcpClientType = "lcp-client";

function saveContractAddresses(addresses) {
  const path = require("path");
  const fs = require("fs");
  const envFile = path.join(__dirname, "..", hre.network.name + ".env.sh");
  let content = "";
  for (const [key, value] of Object.entries(addresses)) {
    content += `export ${key}=${value}\n`;
  }
  console.log("Writing contract addresses to", envFile);
  console.log(content);
  fs.writeFileSync(envFile, content);
}

async function deploy(deployer, contractName, args = []) {
  console.log({contractName});
  const factory = await hre.ethers.getContractFactory(contractName);
  const contract = await factory.connect(deployer).deploy(...args);
  await contract.waitForDeployment();
  return contract;
}

async function deployAndLink(deployer, contractName, libraries, args = []) {
  console.log({contractName});
  const factory = await hre.ethers.getContractFactory(contractName, {
    libraries: libraries
  });
  const contract = await factory.connect(deployer).deploy(...args);
  await contract.waitForDeployment();
  return contract;
}

async function deployIBC(deployer) {
  const logicNames = [
    "IBCClient",
    "IBCConnectionSelfStateNoValidation",
    "IBCChannelHandshake",
    "IBCChannelPacketSendRecv",
    "IBCChannelPacketTimeout",
    "IBCChannelUpgradeInitTryAck",
    "IBCChannelUpgradeConfirmOpenTimeoutCancel"
  ];
  const logics = [];
  for (const name of logicNames) {
    const logic = await deploy(deployer, name);
    logics.push(logic);
  }
  return deploy(deployer, "OwnableIBCHandler", logics.map(l => l.target));
}

async function main() {
  console.log("Deploying contracts...");
  // This is just a convenience check
  if (hre.network.name === "hardhat") {
    console.warn(
      "You are trying to deploy a contract to the Hardhat Network, which" +
        "gets automatically created and destroyed every time. Use the Hardhat" +
        " option '--network localhost'"
    );
  }
  if (hre.network.config.chainId === undefined) {
    throw new Error("chainId is not defined in hardhat.config.js");
  }

  const fs = require('fs');
  let rootCert;
  if (process.env.SGX_MODE === "SW") {
    console.log("RA simulation is enabled");
    rootCert = fs.readFileSync("./config/simulation_rootca.der");
  } else {
    console.log("RA simulation is disabled");
    rootCert = fs.readFileSync("./config/Intel_SGX_Attestation_RootCA.der");
  }

  // ethers is available in the global scope
  const [deployer] = await hre.ethers.getSigners();
  console.log(
    "Deploying the contracts with the account:",
    await deployer.getAddress()
  );
  console.log("Account balance:", (await hre.ethers.provider.getBalance(await deployer.getAddress())).toString());

  const ibcHandler = await deployIBC(deployer);
  const lcpProtoMarshaler = await deploy(deployer, "LCPProtoMarshaler");
  const avrValidator = await deploy(deployer, "AVRValidator");
  const lcpClient = await deployAndLink(deployer, "LCPClient", {
    LCPProtoMarshaler: lcpProtoMarshaler.target,
    AVRValidator: avrValidator.target
  }, [ibcHandler.target, true, rootCert]);
  await ibcHandler.registerClient(lcpClientType, lcpClient.target);

  const ibcMockApp = await deploy(deployer, "IBCMockApp", [ibcHandler.target]);
  console.log("IBCMockApp address:", ibcMockApp.target);

  await ibcHandler.bindPort(portMock, ibcMockApp.target);

  saveContractAddresses({
    IBC_HANDLER: ibcHandler.target,
    LCP_PROTO_MARSHALER: lcpProtoMarshaler.target,
    AVR_VALIDATOR: avrValidator.target,
    LCP_CLIENT: lcpClient.target,
    IBC_MOCKAPP: ibcMockApp.target
  });
}

if (require.main === module) {
  main()
    .then(() => process.exit(0))
    .catch((error) => {
      console.error(error);
      process.exit(1);
    });
}

exports.deployIBC = deployIBC;
