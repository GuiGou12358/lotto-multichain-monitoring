import {ethers} from "hardhat";
import {contractAddress} from "./common";

async function main() {

  console.log("Contract's address: %", contractAddress);

  const lottoInstance = await ethers.getContractAt("LottoClient", contractAddress);

  const raffleId = await lottoInstance.raffleId() ;
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


