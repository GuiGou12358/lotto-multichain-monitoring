import {ethers} from "hardhat";
import {contractAddress, hex} from "./common";

const abiCoder = ethers.AbiCoder.defaultAbiCoder();

async function main() {

  console.log("Contract's address: %", contractAddress);

  const [owner] = await ethers.getSigners();
  const lottoInstance = await ethers.getContractAt("LottoClient", contractAddress);

  const head = await lottoInstance.queueGetUint(hex("_head"));
  const action = abiCoder.encode(['uint32'], [head + BigInt(1)]);
  const set_queue_head = '0x01' + action.substring(2);
  await lottoInstance.connect(owner).rollupU256CondEq([], [], [], [], [set_queue_head]);

  console.log("Queue Head set");

}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});


