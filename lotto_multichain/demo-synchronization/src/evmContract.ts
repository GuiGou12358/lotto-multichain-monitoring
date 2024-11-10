import {ethers} from "ethers";
import {Contract} from "ethers/lib.commonjs/contract/contract";
import RaffleRegistration from "./abi/RaffleRegistration.json";

export class ParticipationEvm {

  private readonly rpc: string;
  private readonly address: string;
  private contract: Contract;

  public constructor(rpc: string, address: string) {
    this.rpc = rpc;
    this.address = address;
  }

  public async init() {

    if (this.contract) {
      return;
    }

    const provider = new ethers.JsonRpcProvider(this.rpc);
    console.log('You are connected to %s - bloc : ', this.rpc, await provider.getBlockNumber());
    this.contract = new ethers.Contract(this.address, RaffleRegistration.abi, provider);
  }

  public async getStatus(): Promise<string> {
    const status = await this.contract.getStatus();
    console.log('Status for %s (%s): %s', this.address, this.rpc, status);
    return status.toString();
  }

  public async getDrawNumber(): Promise<string> {
    const drawNumber = await this.contract.getDrawNumber();
    console.log('Draw Number for %s (%s): %s', this.address, this.rpc, drawNumber);
    return drawNumber.toString();
  }
}
