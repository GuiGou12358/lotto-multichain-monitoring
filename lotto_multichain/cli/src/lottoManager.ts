import {ContractPromise} from "@polkadot/api-contract";
import {KeyringPair} from "@polkadot/keyring/types";
import {readFileSync} from "fs";
import {RaffleConfig, SmartContractConfig, WasmContractCallConfig} from "./config";
import {getApi, query, tx} from "./wasmContractHelper";
import {seed_wasm} from "./seed";
import {Keyring} from "@polkadot/api";


export class RaffleManager {

    private readonly config: SmartContractConfig;
    private smartContract: ContractPromise;
    private signer : KeyringPair;

    public constructor(config: SmartContractConfig){
        this.config = config;
    }

    public async init(){

        if (this.smartContract){
            return;
        }

        const api = await getApi((this.config.call as WasmContractCallConfig).wssRpc);
        const metadata = readFileSync(this.config.metadata);
        this.smartContract = new ContractPromise(api, metadata.toString(), this.config.address);
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
        return await query(this.smartContract, 'raffleManager::getStatus');
    }

    public async getDrawNumber() : Promise<Number> {
        return await query(this.smartContract, 'raffleManager::getDrawNumber');
    }

    public async getRegistrationContracts() : Promise<Number[]> {
        return await query(this.smartContract, 'raffleManager::getRegistrationContracts');
    }

    public async setRegistrationContracts(contractIds: Number[]) : Promise<Number[]> {
        console.log('Raffle Manager - Set the registration contracts');
        return await tx(this.smartContract, this.signer, 'setRegistrationContracts', contractIds);
    }

    public async setConfig(config: RaffleConfig) : Promise<Number[]> {
        console.log('Raffle Manager - Set the config');
        const params = {
            nbNumbers : config.nbNumbers,
            minNumber : config.minNumber,
            maxNumber : config.maxNumber,
        };
        return await tx(this.smartContract, this.signer, 'setConfig', params);
    }

    public async registerAttestor(attestor: string) : Promise<void> {
        console.log('Raffle Manager - Register the attestor');
        const accountId = this.smartContract.api.registry.createType('AccountId', attestor);
        return await tx(this.smartContract, this.signer, 'registerAttestor', accountId);
    }

    public async hasAttestorRole(attestor: string) : Promise<boolean> {
        const attestorRole = 2852625541;
        //const ROLE_GRANT_ATTESTOR = api.registry.createType('u32', 2852625541);
        //const accountId = this.api.registry.createType('AccountId', attestor);
        return await query(this.smartContract, 'accessControl::hasRole', attestorRole, attestor);
    }

    public async closeRegistrations() : Promise<void> {
        console.log('Raffle Manager - Close the registrations');
        return await tx(this.smartContract, this.signer, 'closeRegistrations');
    }


/*
    public async setRegistrationContracts(contractIds: Number[]) : Promise<Number[]> {

        // maximum gas to be consumed for the call. if limit is too small the call will fail.
        const gasLimit: WeightV2 = this.api.registry.createType('WeightV2',
          {refTime: 30000000000, proofSize: 1000000}
        );

        // a limit to how much Balance to be used to pay for the storage created by the contract call
        // if null is passed, unlimited balance can be used
        const storageDepositLimit = null;

        const {gasRequired, result, debugMessage } =
          await this.smartContract.query['setRegistrationContracts'](
            this.signer.address,
            { storageDepositLimit, gasLimit},
            contractIds
          ) ;

        if (result.isOk){
            const tx = this.smartContract.tx['setRegistrationContracts'](
              { storageDepositLimit, gasLimit : gasRequired },
              contractIds
            );
            await signAndSend(tx, this.signer);
        } else {
            console.log('ERROR when completeRaffle - debug message: %s', debugMessage);
            return Promise.reject("ERROR when completeRaffle " + result.asErr);
        }
    }
 */

    public async getRegistrationContractStatus(contractId: Number) : Promise<Number[]> {
        return await query(this.smartContract, 'raffleManager::getRegistrationContractStatus', contractId);
    }

    public async hasPendingMessage() : Promise<Boolean> {
        return await query(this.smartContract, 'hasPendingMessage');
    }

}


/*
async function getRegistrationContractStatus(contractId: Number) : Promise<String>{

    const {result, output} = await lottoManagerSmartContract.query['raffleManager::getRegistrationContractStatus'](alice.address, {gasLimit, storageDepositLimit}, contractId);

    if (result.isOk){
        const value : string = output?.toString() ?? '';
        return JSON.parse(value).ok as string;
    }
    return Promise.reject("ERROR when query getRegistrationContractStatus " + result.asErr);
}

 */

/*
export async function setConfig() : Promise<void>{

    console.log('Signer address: %s', signer.address);
    const {gasRequired, result, debugMessage } =
        await lottoManagerSmartContract.query.completeRaffle(
            signer.address,
            { storageDepositLimit, gasLimit}
        ) ;

    if (result.isOk){
        const tx = lottoManagerSmartContract.tx.completeRaffle({ storageDepositLimit, gasLimit : gasRequired });
        await signAndSend(tx, signer);
    } else {
        console.log('ERROR when completeRaffle - debug message: %s', debugMessage);
        return Promise.reject("ERROR when completeRaffle " + result.asErr);
    }

}
 */


