import {ethers} from "hardhat";

export const abiCoder = ethers.AbiCoder.defaultAbiCoder();

// workflow status
export enum Status { NotStarted, Started, RegistrationsOpen, RegistrationsClosed, ResultsReceived }
// request type
export enum RequestType {SET_CONFIG_AND_START, OPEN_REGISTRATIONS, CLOSE_REGISTRATIONS, SET_RESULTS}

export function hex(str: string): string {
  return ethers.hexlify(ethers.toUtf8Bytes(str));
}

export const contractAddress = "0xF86C68498ea16364B88B84631293CE074CeaE64f";
export const registrationContractId = 31;
export const drawNumber = 1;


