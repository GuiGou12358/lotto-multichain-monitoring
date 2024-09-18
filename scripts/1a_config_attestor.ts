import hre, {ethers} from "hardhat";

async function main() {

  const contractAddress = "0x177b0b863b80Add7cC9824e9232a9a2dcbc7986a";
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


