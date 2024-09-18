import {ethers} from "hardhat";

async function main() {

  const contractAddress = "0x177b0b863b80Add7cC9824e9232a9a2dcbc7986a";
  console.log("Contract's address: %", contractAddress);

  const lottoInstance = await ethers.getContractAt("LottoClient", contractAddress);

  console.log("lottoInstance: %", lottoInstance);
  console.log("current status: %", await lottoInstance.status.staticCall());
  console.log("current raffle id: %", await lottoInstance.currentRaffleId.staticCall());
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});


