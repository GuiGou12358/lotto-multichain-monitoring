import LottoRegistrationMetadata from "./metadata/lotto_registration_contract.json";
import LottoManagerMetadataWasm from "./metadata/lotto_registration_manager_contract.json";
import {ContractPromise} from "@polkadot/api-contract";
import {getApi, query} from "./wasmContractHelper";

export class RaffleManagerWasm {

  private readonly rpc: string;
  private readonly address: string;
  private contract: ContractPromise;

  public constructor(rpc: string, address: string) {
    this.rpc = rpc;
    this.address = address;
  }

  public async init() {

    if (this.contract) {
      return;
    }
    const api = await getApi(this.rpc);
    this.contract = new ContractPromise(api, LottoManagerMetadataWasm, this.address);
  }

  public async getStatus(): Promise<string> {
    const result = (await query(this.contract, 'raffleManager::getStatus')).ok;
    console.debug('Status for manager %s (%s): %s', this.address, this.rpc, result);
    return result;
  }

  public async getDrawNumber(): Promise<string> {
    const result = (await query(this.contract, 'raffleManager::getDrawNumber')).ok;
    console.debug('Draw Number for manager %s (%s): %s', this.address, this.rpc, result);
    return result;
  }

  public async canCloseRegistrations(): Promise<boolean> {
    const result = await query(this.contract, 'canCloseRegistrations');
    console.debug('Manager %s can close the registrations: %s', this.address, result);
    return result;
  }

  public async hasPendingMessage(): Promise<boolean> {
    const result = await query(this.contract, 'hasPendingMessage');
    console.debug('Manager %s has pending message : %s', this.address, result);
    return result;
  }

  public async getNextClosingRegistrations(): Promise<number> {
    return await query(this.contract, 'getNextClosingRegistrations');
  }

  public async getCurrentBlock(): Promise<number> {
    const result = await this.contract.api.rpc.chain.getBlock();
    return result.block.header.number.toNumber();
  }
}


export class RaffleRegistrationWasm {

  private readonly rpc: string;
  private readonly address: string;
  private contract: ContractPromise;

  public constructor(rpc: string, address: string) {
    this.rpc = rpc;
    this.address = address;
  }

  public async init() {

    if (this.contract) {
      return;
    }

    const api = await getApi(this.rpc);
    this.contract = new ContractPromise(api, LottoRegistrationMetadata, this.address);
  }

  public async getStatus(): Promise<string> {
    const result = await query(this.contract, 'raffle::getStatus');
    console.debug('Status for %s (%s): %s', this.address, this.rpc, result.ok);
    return result.ok;
  }

  public async getDrawNumber(): Promise<string> {
    const result = await query(this.contract, 'raffle::getDrawNumber');
    console.debug('Draw Number for %s (%s): %s', this.address, this.rpc, result.ok);
    return result.ok;
  }
}
