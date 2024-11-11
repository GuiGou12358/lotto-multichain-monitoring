import {ethers} from "hardhat";

export const abiCoder = ethers.AbiCoder.defaultAbiCoder();

// workflow status
export enum Status { NotStarted, Started, RegistrationsOpen, RegistrationsClosed, ResultsReceived }
// request type
export enum RequestType {SET_CONFIG_AND_START, OPEN_REGISTRATIONS, CLOSE_REGISTRATIONS, SET_RESULTS}

export function hex(str: string): string {
  return ethers.hexlify(ethers.toUtf8Bytes(str));
}

export const contractAddress = "0x29621E6F2b7DBf256Ff0028dc04986C5E14Db50c";
export const registrationContractId = 31;
export const drawNumber = 1;

export const phatAttestorAddress = "0x01e38f9e010ea0ad5808531f2722e2985f79a7c3";



