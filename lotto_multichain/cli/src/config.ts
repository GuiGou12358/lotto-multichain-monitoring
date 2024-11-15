export interface RaffleConfig {
    readonly nbNumbers: number;
    readonly minNumber: number;
    readonly maxNumber: number;
}

export interface WasmContractCallConfig {
    readonly wssRpc: string;
    readonly httpsRpc: string;
    readonly palletId: number;
    readonly callId: number;
}

export interface EvmContractCallConfig {
    readonly rpc: string;
}

export type ContractCallConfig = WasmContractCallConfig | EvmContractCallConfig;

export interface SmartContractConfig {
    readonly call: ContractCallConfig;
    readonly address: string;
    readonly publicKey: string;
    readonly metadata: string;
}

export interface PhalaConfig {
    readonly wssRpc: string;
    readonly address: string;
    readonly metadata: string;
}

export interface RegistrationContractConfig {
    readonly registrationContractId: number;
    readonly contractConfig: SmartContractConfig;
}

export interface Config {
    readonly raffleConfig : RaffleConfig;
    readonly lottoManager : SmartContractConfig;
    readonly lottoDraw : PhalaConfig;
    readonly lottoRegistrations : RegistrationContractConfig[];
    readonly indexer : string;
}

const shibuyaConfig = new class implements WasmContractCallConfig {
    wssRpc = 'wss://rpc.shibuya.astar.network';
    httpsRpc = 'https://shibuya.public.blastapi.io';
    palletId = 70;
    callId = 6;
}

const minatoConfig = new class implements EvmContractCallConfig {
    rpc = 'https://rpc.minato.soneium.org';
}

const moonbaseConfig = new class implements EvmContractCallConfig {
    rpc = 'https://rpc.api.moonbase.moonbeam.network';
}

const registrationContractShibuya = new class implements RegistrationContractConfig {
    registrationContractId = 10;
    contractConfig = new class implements SmartContractConfig {
        address = 'ZEDzKmBFeCPpjyfTt7aHVVBfM8Cu8i8psj2Z7TEsgnXTBLq';
        publicKey = '0x919d6225b5013f466cf381baf0ef745b1955c94ca4b2cb68b80879af7b28a8f7';
        metadata = './metadata/lotto_registration_contract.json';
        call = shibuyaConfig;
    }
}
const registrationContractMinato = new class implements RegistrationContractConfig {
    registrationContractId = 11;
    contractConfig = new class implements SmartContractConfig {
        address = '0xcB40e3F70c591A230Ce2E263C07682fDd8a784E9';
        publicKey = '0xcB40e3F70c591A230Ce2E263C07682fDd8a784E9';
        metadata = './abi/RaffleRegistration.json';
        call = minatoConfig;
    }
}

const registrationContractMoonbase = new class implements RegistrationContractConfig {
    registrationContractId = 12;
    contractConfig = new class implements SmartContractConfig {
        address = '0x100389C2bA0A9F22B2bEAa8bC976Ca00e63B3724';
        publicKey = '0x100389C2bA0A9F22B2bEAa8bC976Ca00e63B3724';
        metadata = './abi/RaffleRegistration.json';
        call = moonbaseConfig;
    }
}

class TestnetConfig implements Config {
    raffleConfig = new class implements RaffleConfig {
        nbNumbers = 4;
        minNumber = 1;
        maxNumber = 50;
    };
    lottoManager = new class implements SmartContractConfig {
        address = 'bLQBJHeqGUPS1zJgGeEG4yR9pGfom4Sr5j1QSnoGveH86Rz';
        publicKey = '0xeecb9f680c509533c525078772492dbb0449320958417dd00353e3f72ea9903b';
        metadata = './metadata/lotto_registration_manager_contract.json';
        call = shibuyaConfig;
    };
    lottoDraw = new class implements  PhalaConfig {
        wssRpc = 'wss://poc6.phala.network/ws';
        address = '0xc8950613bfd19463ca39d8508c30cbbf310091569de0edf91d429923adbf9929';
        metadata = "./metadata/lotto_draw_multichain.json";
    };
    lottoRegistrations = [registrationContractShibuya, registrationContractMinato, registrationContractMoonbase];
    indexer = "https://query.substrate.fi/lotto-subquery-shibuya";
}

export let config : Config;

export function initConfiguration(network: string) {
    if (network == 'testnet'){
        config = new TestnetConfig();
    } else {
        throw new Error("No config for this Network");
    }
    console.log('Use configuration for %s', network);
}

function displayRegistrationContractConfig(registrationContractConfig : RegistrationContractConfig){
    console.log('RegistrationContractConfig %s : %s %s ',
      registrationContractConfig.registrationContractId,
      registrationContractConfig.contractConfig.address,
      registrationContractConfig.contractConfig.call
    );
}

export function displayConfiguration(){
    console.log('Lotto Config: %s', config.raffleConfig);
    console.log('Lotto Manager: %s %s', config.lottoManager.address, config.lottoManager.call);
    console.log('Lotto Draw: %s  { %s }', config.lottoDraw.address, config.lottoDraw.wssRpc);
    config.lottoRegistrations.forEach( (c) => displayRegistrationContractConfig(c));
}

export function isWasmContract(config: ContractCallConfig) : boolean {
    return (config as WasmContractCallConfig).palletId !== undefined;
}