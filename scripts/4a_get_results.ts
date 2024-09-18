import hre, {ethers} from "hardhat";
import {expect} from "chai";

// result type
const DRAW_NUMBERS = 0;
const CHECK_WINNERS = 1;

const abiCoder = ethers.AbiCoder.defaultAbiCoder();

async function main() {

  const contractAddress = "0x177b0b863b80Add7cC9824e9232a9a2dcbc7986a";
  console.log("Contract's address: %", contractAddress);

  const lottoInstance = await ethers.getContractAt("LottoClient", contractAddress);

  const raffleId = await lottoInstance.currentRaffleId() ;
  const previousRaffle = raffleId - BigInt(1);
  console.log("previousRaffle: %", previousRaffle);
  console.log("Result: %s, %s, %s, %s",
      await lottoInstance.results(previousRaffle, 0),
      await lottoInstance.results(previousRaffle, 1),
      await lottoInstance.results(previousRaffle, 2),
      await lottoInstance.results(previousRaffle, 3)
  );
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});


