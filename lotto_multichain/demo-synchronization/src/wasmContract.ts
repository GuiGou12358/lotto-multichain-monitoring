import LottoRegistrationMetadata from "./metadata/lotto_registration_contract.json";
import LottoManagerMetadataWasm from "./metadata/lotto_registration_manager_contract.json";
import {ApiPromise, WsProvider} from '@polkadot/api';
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
    const status = await query(this.contract, 'raffleManager::getStatus');
    console.log('Status for manager %s (%s): %s', this.address, this.rpc, status);
    return status;
  }

  public async getDrawNumber(): Promise<string> {
    const drawNumber = await query(this.contract, 'raffleManager::getDrawNumber');
    console.log('Draw Number for manager %s (%s): %s', this.address, this.rpc, drawNumber);
    return drawNumber;
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

    const api = await ApiPromise.create({provider: new WsProvider(this.rpc)});
    const [chain, nodeName, nodeVersion] = await Promise.all([
      api.rpc.system.chain(),
      api.rpc.system.name(),
      api.rpc.system.version()
    ]);
    console.log('You are connected to chain %s using %s v%s', chain, nodeName, nodeVersion);

    this.contract = new ContractPromise(api, LottoRegistrationMetadata, this.address);
  }

  public async getStatus(): Promise<string> {
    const json = await query(this.contract, 'raffle::getStatus');
    console.log('Status for %s (%s): %s', this.address, this.rpc, json);
    return json.ok;
  }

  public async getDrawNumber(): Promise<string> {
    const json = await query(this.contract, 'raffle::getDrawNumber');
    console.log('Draw Number for %s (%s): %s', this.address, this.rpc, json);
    return json.ok;
  }
}
