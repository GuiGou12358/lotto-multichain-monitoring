import {ethers} from "hardhat";
import {ParamType} from "ethers/src.ts/abi/fragments";
import {hexString} from "hardhat/internal/core/config/config-validation";


const abiCoder = ethers.AbiCoder.defaultAbiCoder();

async function main() {

  const contractAddress = "0x177b0b863b80Add7cC9824e9232a9a2dcbc7986a";
  console.log("Contract's address: %s", contractAddress);

  const lottoInstance = await ethers.getContractAt("LottoClient", contractAddress);

  console.log("current status: %s", await lottoInstance.status());
  console.log("current raffle id: %s", await lottoInstance.currentRaffleId());

  console.log("head: %s", await lottoInstance.queueGetUint(hex("_head")));
  console.log("tail: %s", await lottoInstance.queueGetUint(hex("_tail")));

  console.log("message 1: %s", await lottoInstance.queueGetBytes("0x0000000000000000000000000000000000000000000000000000000000000001"));
  console.log("message 2: %s", await lottoInstance.queueGetBytes("0x0000000000000000000000000000000000000000000000000000000000000002"));
  console.log("message 3: %s", await lottoInstance.queueGetBytes("0x0000000000000000000000000000000000000000000000000000000000000003"));
  console.log("message 4: %s", await lottoInstance.queueGetBytes("0x0000000000000000000000000000000000000000000000000000000000000004"));
  console.log("message 5: %s", await lottoInstance.queueGetBytes("0x0000000000000000000000000000000000000000000000000000000000000005"));

}

function hex(str: string): string {
  return ethers.hexlify(ethers.toUtf8Bytes(str));
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});


