import {ethers} from "hardhat";
import {contractAddress} from "./common";

async function main() {

  const [owner, attestor] = await ethers.getSigners();
  console.log("owner: %s", owner.address);
  console.log("attestor: %s", attestor.address);
  console.log("Contract's address: %s", contractAddress);

  const lottoInstance = await ethers.getContractAt("RaffleRegistration", contractAddress);

  await lottoInstance.connect(owner).registerAttestor(attestor.address);

  console.log("Attestor granted: %s", attestor.address);

}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});


