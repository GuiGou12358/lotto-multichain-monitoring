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
        address = 'XWfadAkwxCUXAgWsGhC6o5ckLh62AYerLjhkpyHoZf7pfMr';
        publicKey = '0x45aefdc09e1d5d317d6605f4afd248111f0eb2cd172b017d19edf5608b8c7b1d';
        metadata = './metadata/lotto_registration_contract.json';
        call = shibuyaConfig;
    }
}
const registrationContractMinato = new class implements RegistrationContractConfig {
    registrationContractId = 11;
    contractConfig = new class implements SmartContractConfig {
        address = '0x45e6301df1d152C7c048EECAFA658E88fD5A5897';
        publicKey = '0x45e6301df1d152C7c048EECAFA658E88fD5A5897';
        metadata = './abi/RaffleRegistration.json';
        call = minatoConfig;
    }
}

const registrationContractMoonbase = new class implements RegistrationContractConfig {
    registrationContractId = 12;
    contractConfig = new class implements SmartContractConfig {
        address = '0xACDCb69F2C7C1F56C693244F6F5c004A56D3e7E3';
        publicKey = '0xACDCb69F2C7C1F56C693244F6F5c004A56D3e7E3';
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
        address = 'WxR8LgRfZj74duZuxEhLmFYXjmAB3DRzZuM2sd51An7NvfB';
        publicKey = '0x2d167c0b6595a331c6bb2854931ed8b4fbae5a269ae0096b13264408c798996a';
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