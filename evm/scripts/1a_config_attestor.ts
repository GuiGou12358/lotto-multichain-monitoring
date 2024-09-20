import hre, {ethers} from "hardhat";
import {contractAddress} from "./common";

async function main() {

  console.log("Contract's address: %", contractAddress);

  const [owner] = await ethers.getSigners();
  const lottoInstance = await ethers.getContractAt("LottoClient", contractAddress);

  await lottoInstance.connect(owner).registerAttestor(owner.address);

  console.log("Attestor granted: %", owner.address);

}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});


