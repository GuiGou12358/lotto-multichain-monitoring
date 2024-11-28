import {getClient, getContract, PinkContractPromise} from '@phala/sdk';
import {query, tx} from './pinkContractHelper';
import {
    ContractCallConfig,
    EvmContractCallConfig, isWasmContract,
    PhalaConfig,
    RegistrationContractConfig,
    SmartContractConfig,
    WasmContractCallConfig
} from "./config";
import {Keyring} from "@polkadot/api";
import {decodeAddress} from "@polkadot/util-crypto";
import {u8aToHex} from "@polkadot/util";
import {KeyringPair} from "@polkadot/keyring/types";
import {readFileSync} from "fs";
import {seed_wasm} from "./seed";

const METADATA_FILE = './metadata/lotto_draw_multichain.json';

export class LottoDraw {

    private readonly config: PhalaConfig;
    private smartContract: PinkContractPromise;
    private signer : KeyringPair;

    public constructor(config: PhalaConfig){
        this.config = config;
    }

    public async connect(){

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

        const metadata = readFileSync(METADATA_FILE, 'utf-8');
        this.smartContract = await getContract({
            client,
            contractId: this.config.address,
            abi: metadata,
            //provider,
          }
        );

        this.signer = new Keyring({ type: 'sr25519' }).addFromUri(seed_wasm);
    }

    public async synchronize() : Promise<void> {
        console.log('Raffle - Synchronise');
        return await query(this.smartContract, 'answerRequest');
    }

    public async closeRegistrations() : Promise<void> {
        console.log('Raffle - closeRegistrations');
        return await query(this.smartContract, 'closeRegistrations');
    }

    public async getAttestAddressEvm() : Promise<string> {
        return await query(this.smartContract, 'getAttestAddressEvm');
    }

    public async getAttestAddressSubstrate() : Promise<string> {
        return await query(this.smartContract, 'getAttestAddressSubstrate');
    }

    public async getAttestEcdsaAddressSubstrate() : Promise<string> {
        return await query(this.smartContract, 'getAttestEcdsaAddressSubstrate');
    }

    private getCallConfig(call: ContractCallConfig, contractId: string, senderKey: string) : any {
        let config;
        if (isWasmContract(call)) {
            const callConfig = call as WasmContractCallConfig;
            config =
              {
                  wasm: {
                      rpc: callConfig.httpsRpc,
                      palletId: callConfig.palletId,
                      callId: callConfig.callId,
                      contractId: u8aToHex(decodeAddress(contractId)),
                      senderKey: senderKey,
                  }
              };
        } else {
            const callConfig = call as EvmContractCallConfig;
            config =
              {
                  evm: {
                      rpc: callConfig.rpc,
                      contractId: contractId,
                      senderKey: senderKey,
                  }
              };
        }
        return config;
    }

    public async configIndexer(url: string) : Promise<void> {
        console.log('Communicator - Set the indexer');
        return await tx(this.smartContract, this.signer, 'configIndexer', url);
    }

    public async setRaffleManager(raffleManagerConfig: SmartContractConfig, senderKey: string) : Promise<void> {
        console.log('Communicator - Set the raffle manager');
        const config = this.getCallConfig(raffleManagerConfig.call, raffleManagerConfig.address, senderKey);
        return await tx(this.smartContract, this.signer, 'setConfigRaffleManager', config);
    }

    public async setRaffleRegistration(raffleRegistrationConfig: RegistrationContractConfig, senderKey: string) : Promise<void> {
        const registrationContractId = raffleRegistrationConfig.registrationContractId;
        console.log('Communicator - Set the raffle registration %s', registrationContractId);
        let config = this.getCallConfig(raffleRegistrationConfig.contractConfig.call, raffleRegistrationConfig.contractConfig.address, senderKey);
        return await tx(this.smartContract, this.signer, 'setConfigRaffleRegistrations', registrationContractId, config);
    }

}

