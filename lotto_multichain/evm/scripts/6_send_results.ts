import {ethers} from "hardhat";
import {abiCoder, contractAddress, drawNumber, RequestType} from "./common";

async function main() {

  console.log("Contract's address: %s", contractAddress);

  const [_owner, attestor] = await ethers.getSigners();

  const lottoInstance = await ethers.getContractAt("RaffleRegistration", contractAddress);

  const request_bytes = abiCoder.encode(
    ['uint', 'uint[]', 'address[]'],
    [drawNumber, [33, 47, 5, 6], []]
  );
  const action = abiCoder.encode(
    ['uint8', 'bytes'],
    [RequestType.SET_RESULTS, request_bytes]
  );
  const reply = '0x00' + action.substring(2);

  await lottoInstance.connect(attestor).rollupU256CondEq([], [], [], [], [reply]);
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});


