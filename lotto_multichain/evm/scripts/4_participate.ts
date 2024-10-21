import {ethers} from "hardhat";
import {contractAddress} from "./common";

async function main() {

  console.log("Contract's address: %s", contractAddress);

  const [owner, _attestor, user1, user2] = await ethers.getSigners();
  console.log("user 1: %s", user1.address);
  console.log("user 2: %s", user2.address);

  const lottoInstance = await ethers.getContractAt("RaffleRegistration", contractAddress);

  await lottoInstance.connect(owner).participate([2, 17, 31, 45]);
  await lottoInstance.connect(owner).participate([15, 30, 28, 49]);

  console.log("Participation done");
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});


