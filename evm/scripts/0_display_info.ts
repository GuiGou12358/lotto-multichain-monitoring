import {ethers} from "hardhat";
import {hex, contractAddress} from "./common";

const abiCoder = ethers.AbiCoder.defaultAbiCoder();

async function main() {

  console.log("Contract's address: %s", contractAddress);

  const lottoInstance = await ethers.getContractAt("LottoClient", contractAddress);

  console.log("current status: %s", await lottoInstance.status());
  console.log("current raffle id: %s", await lottoInstance.currentRaffleId());

  const head = await lottoInstance.queueGetUint(hex("_head"));
  const tail = await lottoInstance.queueGetUint(hex("_tail"));
  console.log("Queue info - head: %s - tail: %s", head, tail);

  for (var i = head; i < tail; i++){
    const key = abiCoder.encode(['uint'], [i]);
    console.log("key for message %s: %s", i, key);
    console.log("message %s: %s", i, await lottoInstance.queueGetBytes(key));
  }
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});


