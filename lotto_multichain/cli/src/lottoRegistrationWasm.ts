import {RegistrationContractConfig, WasmContractCallConfig} from "./config";
import {readFileSync} from "fs";
import {ContractPromise} from "@polkadot/api-contract";
import {getApi, query, tx} from "./wasmContractHelper";
import {Keyring} from "@polkadot/api";
import {KeyringPair} from "@polkadot/keyring/types";
import {seed} from "./seed";

export class RaffleRegistration {

    private readonly config: RegistrationContractConfig;
    private smartContract: ContractPromise;
    private signer : KeyringPair;

    public constructor(config: RegistrationContractConfig){
        this.config = config;
    }

    public async init(){

        if (this.smartContract){
            return;
        }

        const api = await getApi((this.config.contractConfig.call as WasmContractCallConfig).wssRpc);
        const metadata = readFileSync(this.config.contractConfig.metadata);
        this.smartContract = new ContractPromise(api, metadata.toString(), this.config.contractConfig.address);
        this.signer = new Keyring({ type: 'sr25519' }).addFromUri(seed);
    }

    public async display() {
        const status = await this.getStatus();
        const drawNumber = await this.getDrawNumber();
        const registrationContractId = await this.getRegistrationContractId();
        console.log('Registration contract %s - Draw Number: %s - status %s', registrationContractId, drawNumber, status);
    }

    public async getStatus() : Promise<String> {
        return await query(this.smartContract, 'raffle::getStatus');
    }

    public async getDrawNumber() : Promise<Number> {
        return await query(this.smartContract, 'raffle::getDrawNumber');
    }

    public async getRegistrationContractId() : Promise<Number> {
        return await query(this.smartContract, 'getRegistrationContractId');
    }

    public async registerAttestor(attestor: string) : Promise<void> {
        const accountId = this.smartContract.api.registry.createType('AccountId', attestor);
        return await tx(this.smartContract, this.signer, 'registerAttestor', accountId);
    }

    public async hasAttestorRole(attestor: string) : Promise<boolean> {
        const attestorRole = 2852625541;
        //const ROLE_GRANT_ATTESTOR = api.registry.createType('u32', 2852625541);
        //const accountId = this.api.registry.createType('AccountId', attestor);
        return await query(this.smartContract, 'accessControl::hasRole', attestorRole, attestor);
    }

}





