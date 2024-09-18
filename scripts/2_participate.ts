import {ethers} from "hardhat";

async function main() {

  const contractAddress = "0x177b0b863b80Add7cC9824e9232a9a2dcbc7986a";
  console.log("Contract's address: %", contractAddress);

  const [owner, user1, user2] = await ethers.getSigners();
  console.log("user 1: %", user1);
  console.log("user 2: %", user2);

  const lottoInstance = await ethers.getContractAt("LottoClient", contractAddress);

  await lottoInstance.connect(owner).participate([2, 17, 31, 45]);
  await lottoInstance.connect(owner).participate([15, 30, 28, 49]);

  console.log("Participation done");
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});


