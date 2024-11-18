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
    httpsRpc = 'https://rpc.shibuya.astar.network';
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
        address = 'ZkMmwcAsCuFPB13kfXH6aQiJYAfnQYC2qMiK5h1mGKsMX86';
        publicKey = '0xa898f50fd7f434af9a1a0a5ffaa10b499938abff3e334f29fe845850dceb056e';
        metadata = './metadata/lotto_registration_contract.json';
        call = shibuyaConfig;
    }
}
const registrationContractMinato = new class implements RegistrationContractConfig {
    registrationContractId = 11;
    contractConfig = new class implements SmartContractConfig {
        address = '0xA8AE9c3F7bc784Ccd1E6013c59A233600C6dE90A';
        publicKey = '0xA8AE9c3F7bc784Ccd1E6013c59A233600C6dE90A';
        metadata = './abi/RaffleRegistration.json';
        call = minatoConfig;
    }
}

const registrationContractMoonbase = new class implements RegistrationContractConfig {
    registrationContractId = 12;
    contractConfig = new class implements SmartContractConfig {
        address = '0x991926D5ca21EF2938B5BAffbf4EC24fB55e205e';
        publicKey = '0x991926D5ca21EF2938B5BAffbf4EC24fB55e205e';
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
        address = 'YuCwFXie1QX7pPyvL8dHYGCX6gpCPB4aWchzb2bSYSgdrvu';
        publicKey = '0x831c2d11adb7334113300493e5d02067ade55748d6e865e851a9f53a9444b2b3';
        metadata = './metadata/lotto_registration_manager_contract.json';
        call = shibuyaConfig;
    };
    lottoDraw = new class implements  PhalaConfig {
        wssRpc = 'wss://poc6.phala.network/ws';
        address = '0x49badf682da735bee55e1098414edbd993f3500893b35cadd2d610bc961b0d33';
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