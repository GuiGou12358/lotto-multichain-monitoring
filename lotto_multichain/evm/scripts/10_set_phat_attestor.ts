import {ethers} from "hardhat";
import {contractAddress, phatAttestorAddress} from "./common";

async function main() {

  const [owner] = await ethers.getSigners();
  console.log("owner: %s", owner.address);
  console.log("phat attestor: %s", phatAttestorAddress);
  console.log("Contract's address: %s", contractAddress);

  const lottoInstance = await ethers.getContractAt("RaffleRegistration", contractAddress);

  await lottoInstance.connect(owner).registerAttestor(phatAttestorAddress);

  console.log("Attestor granted: %s", phatAttestorAddress);

}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});


