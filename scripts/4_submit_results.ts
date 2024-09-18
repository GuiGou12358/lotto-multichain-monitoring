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

  const raffleId = await lottoInstance.currentRaffleId();
  const request = abiCoder.encode(['uint8', 'uint', 'uint'], [4, 1, 50]);
  const response = abiCoder.encode(['uint[]'], [[33, 47, 5, 6]]);
  const action = abiCoder.encode(['uint', 'uint8', 'bytes', 'bytes'], [raffleId, DRAW_NUMBERS, request, response]);
  const reply = '0x00' + action.substring(2);
  await lottoInstance.connect(owner).rollupU256CondEq([], [], [], [], [reply]);

  console.log("Results sent");

}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});


