import {getClient, getContract, PinkContractPromise} from '@phala/sdk';
import {query, tx} from './pinkContractHelper';
import {
    ContractCallConfig,
    EvmContractCallConfig,
    PhalaConfig,
    RegistrationContractConfig,
    SmartContractConfig,
    WasmContractCallConfig
} from "./config";
import {Keyring} from "@polkadot/api";
import {KeyringPair} from "@polkadot/keyring/types";
import {readFileSync} from "fs";
import {seed} from "./seed";


export class LottoDraw {

    private readonly config: PhalaConfig;
    private smartContract: PinkContractPromise;
    private signer : KeyringPair;

    public constructor(config: PhalaConfig){
        this.config = config;
    }

    public async init(){

        if (this.smartContract){
            return;
        }

        const client = await getClient({
            transport: this.config.wssRpc
        });

        const[chain, nodeName, nodeVersion] = await Promise.all([
            client.api.rpc.system.chain(),
            client.api.rpc.system.name(),
            client.api.rpc.system.version()
        ]);
        console.log('You are connected to chain %s using %s v%s', chain, nodeName, nodeVersion);

        const metadata = readFileSync(this.config.metadata, 'utf-8');
        this.smartContract = await getContract({
            client,
            contractId: this.config.address,
            abi: metadata,
            //provider,
          }
        );

        this.signer = new Keyring({ type: 'sr25519' }).addFromUri(seed);
    }

    public async synchronize() : Promise<void> {
        return await query(this.smartContract, 'answerRequest');
    }

    public async getAttestEcdsaAddressSubstrate() : Promise<string> {
        return await query(this.smartContract, 'getAttestEcdsaAddressSubstrate');
    }

    public async getAttestEcdsaAddressEvm() : Promise<string> {
        return await query(this.smartContract, 'getAttestEcdsaAddressEvm');
    }

    private getCallConfig(call: ContractCallConfig, publicKey: string, senderKey: string) : any {
        let config;
        if ((call as  WasmContractCallConfig).palletId !== undefined) {
            const callConfig = call as WasmContractCallConfig;
            config =
              {
                  wasm: {
                      rpc: callConfig.httpsRpc,
                      palletId: callConfig.palletId,
                      callId: callConfig.callId,
                      contractId: publicKey,
                      senderKey: senderKey,
                  }
              };
        } else {
            const callConfig = call as EvmContractCallConfig;
            config =
              {
                  evm: {
                      rpc: callConfig.rpc,
                      contractId: publicKey,
                      senderKey: senderKey,
                  }
              };
        }
        return config;
    }

    public async configIndexer(url: string) : Promise<void> {
        return await tx(this.smartContract, this.signer, 'configIndexer', url);
    }

    public async setRaffleManager(raffleManagerConfig: SmartContractConfig) : Promise<void> {

        const senderKey = "0xea31cc677ba1c0109cda39829e2f3c00d7ec36ea08b186d2ec906a2bb8849e3d";
        let config = this.getCallConfig(raffleManagerConfig.call, raffleManagerConfig.publicKey, senderKey);
        return await tx(this.smartContract, this.signer, 'setConfigRaffleManager', config);

/*
        await query(this.smartContract.query.setConfigRaffleManager, this.signer, config);
        await signAndSend2(this.client, this.smartContract.tx.setConfigRaffleManager, this.signer, config);

 */
/*

        const gasLimit: WeightV2 = this.client.api.registry.createType('WeightV2',
          {refTime: 30000000000, proofSize: 1000000}
        );
        // a limit to how much Balance to be used to pay for the storage created by the contract call
        // if null is passed, unlimited balance can be used
        const storageDepositLimit = null;

        const certificate = await signCertificate({ pair: this.signer });

        const {result, output } =
          await this.smartContract.query.setConfigRaffleManager(
            this.signer.address,
            {cert: certificate},
            config
          ) ;

        if (result.isOk){
            const value : string = output?.toString() ?? '';
            const res = JSON.parse(value).ok;
            if (res.err){
                console.log('query result: %s', value);
                return Promise.reject("Error when setConfigRaffleManager " + res.err);
            }

            const tx = this.smartContract.tx.setConfigRaffleManager(
              { storageDepositLimit, gasLimit },
              config
            );
            await signAndSend(tx, this.signer);
        } else {
            console.log('ERROR when setConfigRaffleManager : %s', output);
            return Promise.reject("ERROR when setConfigRaffleManager " + result.asErr);
        }

 */
    }

    public async setRaffleRegistration(raffleRegistrationConfig: RegistrationContractConfig) : Promise<void> {

        const registrationContractId = raffleRegistrationConfig.registrationContractId;
        const publicKey = raffleRegistrationConfig.contractConfig.publicKey;
        const senderKey = "0xea31cc677ba1c0109cda39829e2f3c00d7ec36ea08b186d2ec906a2bb8849e3d";
        let config = this.getCallConfig(raffleRegistrationConfig.contractConfig.call, publicKey, senderKey);
        return await tx(this.smartContract, this.signer, 'setConfigRaffleRegistrations', registrationContractId, config);

/*
        const gasLimit: WeightV2 = this.client.api.registry.createType('WeightV2',
          {refTime: 30000000000, proofSize: 1000000}
        );
        // a limit to how much Balance to be used to pay for the storage created by the contract call
        // if null is passed, unlimited balance can be used
        const storageDepositLimit = null;

        const certificate = await signCertificate({ pair: this.signer });

        const {result, output } =
          await this.smartContract.query.setConfigRaffleRegistrations(
            this.signer.address,
            {cert: certificate},
            registrationContractId, config
          ) ;

        if (result.isOk){
            const value : string = output?.toString() ?? '';
            const res = JSON.parse(value).ok;
            if (res.err){
                console.log('query result: %s', value);
                return Promise.reject("Error when setConfigRaffleRegistrations " + res.err);
            }

            const tx = this.smartContract.tx.setConfigRaffleRegistrations(
              { storageDepositLimit, gasLimit },
              registrationContractId, config
            );
            await signAndSend(tx, this.signer);
        } else {
            console.log('ERROR when setConfigRaffleRegistrations : %s', output);
            return Promise.reject("ERROR when setConfigRaffleRegistrations " + result.asErr);
        }

 */
    }


}

