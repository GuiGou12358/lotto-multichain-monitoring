import hre, {ethers} from "hardhat";
import {expect} from "chai";

// result type
const DRAW_NUMBERS = 0;
const CHECK_WINNERS = 1;

const abiCoder = ethers.AbiCoder.defaultAbiCoder();

async function main() {

  const contractAddress = "0x177b0b863b80Add7cC9824e9232a9a2dcbc7986a";
  console.log("Contract's address: %", contractAddress);

  const [owner] = await ethers.getSigners();
  const lottoInstance = await ethers.getContractAt("LottoClient", contractAddress);

  const action = abiCoder.encode(['uint32'], [4]);
  const set_queue_head = '0x01' + action.substring(2);
  await lottoInstance.connect(owner).rollupU256CondEq([], [], [], [], [set_queue_head]);

  console.log("Queue Head set");

}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});


