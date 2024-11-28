import {RegistrationContractConfig, WasmContractCallConfig} from "./config";
import {readFileSync} from "fs";
import {ContractPromise} from "@polkadot/api-contract";
import {getApi, query, tx} from "./wasmContractHelper";
import {Keyring} from "@polkadot/api";
import {KeyringPair} from "@polkadot/keyring/types";
import {seed_wasm} from "./seed";
import {instantiateWithCode} from "./txHelper";

export class RaffleRegistrationWasm {

    private readonly config: RegistrationContractConfig;
    private contract: ContractPromise;

    public constructor(config: RegistrationContractConfig){
        this.config = config;
    }

    private getSigner() : KeyringPair {
        return new Keyring({ type: 'sr25519' }).addFromUri(seed_wasm);
    }

    public async instantiate() : Promise<void> {

        if (this.contract){
            return;
        }

        const signer = this.getSigner();
        const address = await instantiateWithCode(this.config.contractConfig, signer);
        this.config.contractConfig.address = address;
        console.log('New WASM Raffle Registration instantiated: %s', address);

        await this.connect();
    }

    public async connect(){

        if (this.contract){
            return;
        }

        const api = await getApi((this.config.contractConfig.call as WasmContractCallConfig).wssRpc);
        const metadata = readFileSync(this.config.contractConfig.metadata);
        this.contract = new ContractPromise(api, metadata.toString(), this.config.contractConfig.address);
    }

    public async display() {
        console.log('Raffle Registration %s - %s - %s',
          this.config.registrationContractId,
          this.config.contractConfig.address,
          (this.config.contractConfig.call as WasmContractCallConfig).wssRpc
        );
        const status = await this.getStatus();
        const drawNumber = await this.getDrawNumber();
        const registrationContractId = await this.getRegistrationContractId();
        console.log('Registration contract %s - Draw Number: %s - status %s', registrationContractId, drawNumber, status);
    }

    public async getStatus() : Promise<String> {
        return await query(this.contract, 'raffle::getStatus');
    }

    public async getDrawNumber() : Promise<Number> {
        return await query(this.contract, 'raffle::getDrawNumber');
    }

    public async getRegistrationContractId() : Promise<Number> {
        return await query(this.contract, 'getRegistrationContractId');
    }

    public async registerAttestor(attestor: string) : Promise<void> {
        console.log('Raffle Registration %s - Register the attestor %s', this.config.registrationContractId, attestor);
        const accountId = this.contract.api.registry.createType('AccountId', attestor);
        const signer = this.getSigner();
        return await tx(this.contract, signer, 'registerAttestor', accountId);
    }

    public async hasAttestorRole(attestor: string) : Promise<boolean> {
        const attestorRole = 2852625541;
        //const ROLE_GRANT_ATTESTOR = api.registry.createType('u32', 2852625541);
        //const accountId = this.api.registry.createType('AccountId', attestor);
        return await query(this.contract, 'accessControl::hasRole', attestorRole, attestor);
    }

}
