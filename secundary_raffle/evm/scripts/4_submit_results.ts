import {ethers} from "hardhat";
import {contractAddress, DRAW_NUMBERS, hex} from "./common";

const abiCoder = ethers.AbiCoder.defaultAbiCoder();

async function main() {

  console.log("Contract's address: %", contractAddress);

  const [owner] = await ethers.getSigners();
  const lottoInstance = await ethers.getContractAt("LottoClient", contractAddress);

  // reply
  const raffleId = await lottoInstance.raffleId();
  const request = abiCoder.encode(['uint8', 'uint', 'uint'], [4, 1, 50]);
  const response = abiCoder.encode(['uint[]'], [[43, 50, 2, 15]]);
  const action1 = abiCoder.encode(['uint', 'uint8', 'bytes', 'bytes'], [raffleId, DRAW_NUMBERS, request, response]);
  const reply = '0x00' + action1.substring(2);

  // remove the message in the queue
  const head = await lottoInstance.queueGetUint(hex("_head"));
  const action2 = abiCoder.encode(['uint32'], [head + BigInt(1)]);
  const removeMessageQueue = '0x01' + action2.substring(2);

  await lottoInstance.connect(owner).rollupU256CondEq([], [], [], [], [reply, removeMessageQueue]);

  console.log("Results sent");

}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});


