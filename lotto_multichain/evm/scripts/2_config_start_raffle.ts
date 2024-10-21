import {ethers} from "hardhat";
import {abiCoder, contractAddress, registrationContractId, RequestType} from "./common";

async function main() {

  console.log("Contract's address: %s", contractAddress);

  const [_owner, attestor] = await ethers.getSigners();

  const lottoInstance = await ethers.getContractAt("RaffleRegistration", contractAddress);

  const request_bytes = abiCoder.encode(
    ['uint8', 'uint', 'uint', 'uint'],
    [4, 1, 50, registrationContractId]
  );
  const action = abiCoder.encode(
    ['uint8', 'bytes'],
    [RequestType.SET_CONFIG_AND_START, request_bytes]
  );
  const reply = '0x00' + action.substring(2);

  await lottoInstance.connect(attestor).rollupU256CondEq([], [], [], [], [reply]);
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});


