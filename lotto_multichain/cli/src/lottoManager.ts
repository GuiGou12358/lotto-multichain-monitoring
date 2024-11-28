import {ContractPromise} from "@polkadot/api-contract";
import {KeyringPair} from "@polkadot/keyring/types";
import {readFileSync} from "fs";
import {RaffleConfig, SmartContractConfig, WasmContractCallConfig} from "./config";
import {getApi, query, tx} from "./wasmContractHelper";
import {seed_wasm} from "./seed";
import {Keyring} from "@polkadot/api";
import {instantiateWithCode} from "./txHelper";


export class RaffleManager {

    private readonly config: SmartContractConfig;
    private contract: ContractPromise;
    private signer : KeyringPair;

    public constructor(config: SmartContractConfig){
        this.config = config;
    }

    public async instantiate() : Promise<void> {

        if (this.contract){
            return;
        }

        const api = await getApi((this.config.call as WasmContractCallConfig).wssRpc);

        const signer  = new Keyring({ type: 'sr25519' }).addFromUri(seed_wasm);
        const address = await instantiateWithCode(this.config, signer);
        this.config.address = address;
        console.log('new Raffle Manager instantiated: %s', address);

        await this.connect();
    }

    public async connect(){

        if (this.contract){
            return;
        }

        console.log('Connect to Raffle Manager %s', this.config.address);

        const api = await getApi((this.config.call as WasmContractCallConfig).wssRpc);
        const metadata = readFileSync(this.config.metadata);
        this.contract = new ContractPromise(api, metadata.toString(), this.config.address);
        this.signer = new Keyring({ type: 'sr25519' }).addFromUri(seed_wasm);
    }

    public async display() {

        console.log('Raffle Manager -  %s - %s', this.config.address, (this.config.call as WasmContractCallConfig).wssRpc);
        const status = await this.getStatus();
        const drawNumber = await this.getDrawNumber();
        console.log('Raffle Manager -  Draw Number: %s - status %s', drawNumber, status);
        const contractIds = await this.getRegistrationContracts();
        for (let id of contractIds){
            const regStatus = await this.getRegistrationContractStatus(id);
            console.log('Status for registration contract %s: %s', id, regStatus);
        }
    }

    public async getStatus() : Promise<String> {
        return await query(this.contract, 'raffleManager::getStatus');
    }

    public async getDrawNumber() : Promise<Number> {
        return await query(this.contract, 'raffleManager::getDrawNumber');
    }

    public async getRegistrationContracts() : Promise<Number[]> {
        return await query(this.contract, 'raffleManager::getRegistrationContracts');
    }

    public async setRegistrationContracts(contractIds: Number[]) : Promise<void> {
        console.log('Raffle Manager - Set the registration contracts');
        await tx(this.contract, this.signer, 'setRegistrationContracts', contractIds);
    }

    public async setConfig(config: RaffleConfig) : Promise<void> {
        console.log('Raffle Manager - Set the config');
        const params = {
            nbNumbers : config.nbNumbers,
            minNumber : config.minNumber,
            maxNumber : config.maxNumber,
        };
        await tx(this.contract, this.signer, 'setConfig', params);

        await tx(this.contract, this.signer, 'setNumberOfBlocksForParticipation', config.numberOfBlocksForParticipation);
    }

    public async registerAttestor(attestor: string) : Promise<void> {
        console.log('Raffle Manager - Register the attestor');
        const accountId = this.contract.api.registry.createType('AccountId', attestor);
        await tx(this.contract, this.signer, 'registerAttestor', accountId);
    }

    public async hasAttestorRole(attestor: string) : Promise<boolean> {
        const attestorRole = 2852625541;
        //const ROLE_GRANT_ATTESTOR = api.registry.createType('u32', 2852625541);
        //const accountId = this.api.registry.createType('AccountId', attestor);
        return await query(this.contract, 'accessControl::hasRole', attestorRole, attestor);
    }

    public async start() : Promise<void> {
        console.log('Raffle Manager - Start');
        await tx(this.contract, this.signer, 'start', 0);
    }

    public async getRegistrationContractStatus(contractId: Number) : Promise<Number[]> {
        return await query(this.contract, 'raffleManager::getRegistrationContractStatus', contractId);
    }

    public async hasPendingMessage() : Promise<Boolean> {
        return await query(this.contract, 'hasPendingMessage');
    }

    public async canCloseRegistrations() : Promise<Boolean> {
        return await query(this.contract, 'canCloseRegistrations');
    }
}



