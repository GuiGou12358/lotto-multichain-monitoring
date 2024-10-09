import {ethers} from "hardhat";

export const contractAddress = "0x177b0b863b80Add7cC9824e9232a9a2dcbc7986a";

// result type
export const DRAW_NUMBERS = 0;
export const CHECK_WINNERS = 1;

export function hex(str: string): string {
  return ethers.hexlify(ethers.toUtf8Bytes(str));
}

