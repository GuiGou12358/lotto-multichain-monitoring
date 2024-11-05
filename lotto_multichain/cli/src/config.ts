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
        address = 'Z3idfVQaj7cV2sdM9Vw2mjJ1Vr5fFPm5RWECQBUQeEHV2L8';
        publicKey = '0x899a1b887428fc89f78a44c7212ac7ae29db06572ab721c3881b2e8ab9980227';
        metadata = './metadata/lotto_registration_contract.json';
        call = shibuyaConfig;
    }
}
const registrationContractMinato = new class implements RegistrationContractConfig {
    registrationContractId = 11;
    contractConfig = new class implements SmartContractConfig {
        address = '0x83121dDd37aa589C391b5e44bD8f94D978970bBA';
        publicKey = '0x83121dDd37aa589C391b5e44bD8f94D978970bBA';
        metadata = './abi/RaffleRegistration.json.json';
        call = minatoConfig;
    }
}

const registrationContractMoonbase = new class implements RegistrationContractConfig {
    registrationContractId = 12;
    contractConfig = new class implements SmartContractConfig {
        address = '0x22bcC50980B8C6bA38DB0E7077B4EB47dd24E351';
        publicKey = '0x22bcC50980B8C6bA38DB0E7077B4EB47dd24E351';
        metadata = './abi/RaffleRegistration.json.json';
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
        address = 'XFd4chL4urinkgMmAiTq38cmSa6QdbLJw1hxjghi6WcdgzY';
        publicKey = '0x3a35d5fd0234b7e95bdee84944628cf0fce976f41492766db87acfe272fef003';
        metadata = './metadata/lotto_registration_manager_contract.json';
        call = shibuyaConfig;
    };
    lottoDraw = new class implements  PhalaConfig {
        wssRpc = 'wss://poc6.phala.network/ws';
        address = '0xa59d4b80ad64e34e053575e5ae0f7664f3d0ba3c04e6ea52cb297219f3702a3c';
        metadata = "./metadata/lotto_draw_multichain.json";
    };
    lottoRegistrations = [registrationContractShibuya, registrationContractMinato, registrationContractMoonbase];
    indexer = "https://query.substrate.fi/lotto-subquery-shibuya";
}

export let config : Config;

export function initConfiguration(network: string) {
    console.log('Set config for %s', network);
    if (network == 'testnet'){
        config = new TestnetConfig();
    } else {
        throw new Error("No config for this Network");
    }
}

function displayRegistrationContractConfig(registrationContractConfig : RegistrationContractConfig){
    console.log('RegistrationContractConfig %s : %s %s ',
      registrationContractConfig.registrationContractId,
      registrationContractConfig.contractConfig.address,
      registrationContractConfig.contractConfig.call
    );
}

export function displayConfiguration(){
    console.log('Lotto Manager: %s %s', config.lottoManager.address, config.lottoManager.call);
    console.log('Lotto Draw: %s  { %s }', config.lottoDraw.address, config.lottoDraw.wssRpc);
    config.lottoRegistrations.forEach( (c) => displayRegistrationContractConfig(c));
}

