task("send-packet", "Send a packet")
  .addParam("mockapp", "The address of the MockApp contract")
  .addParam("message", "The message to send")
  .addParam("port", "The source port to send the packet to")
  .addParam("channel", "The source channel to send the packet to")
  .addParam("timeoutheight", "The timeout height to send the packet to")
  .setAction(async (taskArgs, hre) => {
    const mockApp = await hre.ethers.getContractAt("IBCMockApp", taskArgs.mockapp);

    console.log("try to send a packet:", taskArgs.message, taskArgs.port, taskArgs.channel, [0, 0], taskArgs.timeoutheight);
    await mockApp.sendPacket(hre.ethers.toUtf8Bytes(taskArgs.message), taskArgs.port, taskArgs.channel, [0, taskArgs.timeoutheight], 0);
  });
