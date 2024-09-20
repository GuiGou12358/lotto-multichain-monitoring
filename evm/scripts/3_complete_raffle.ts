import hre, {ethers} from "hardhat";
import {contractAddress} from "./common";

async function main() {

  console.log("Contract's address: %", contractAddress);

  const [owner] = await ethers.getSigners();
  const lottoInstance = await ethers.getContractAt("LottoClient", contractAddress);

  await lottoInstance.connect(owner).completeRaffle();
  console.log("Raffle Completed");

}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});


