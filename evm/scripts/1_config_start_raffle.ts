import hre, {ethers} from "hardhat";
import {contractAddress} from "./common";

async function main() {

  console.log("Contract's address: %", contractAddress);

  const [owner] = await ethers.getSigners();
  console.log("owner: %", owner);

  const lottoInstance = await ethers.getContractAt("LottoClient", contractAddress);

  await lottoInstance.connect(owner).setConfig(4, 1, 50);
  await lottoInstance.connect(owner).startRaffle();
  //await lottoInstance.connect(owner).registerAttestor(attestor.address);
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});


