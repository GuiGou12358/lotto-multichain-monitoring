
interface SmartContractConfig {
    readonly rpc: string;
    readonly address: string;
    readonly metadata: string;
}

interface Config {
    readonly lottoManager : SmartContractConfig;
    readonly lottoDraw : SmartContractConfig;
    readonly lottoPhatContractAttestorAddress : string;
}

class LottoManagerShibuyaConfig implements SmartContractConfig {
    rpc = 'wss://rpc.shibuya.astar.network';
    address = 'WHzpTa2BdDfj6kNfXXJt7AjoFRwCEN6tF3Renhi7xbrWPBN';
    metadata = './metadata/lotto_registration_manager_contract.json';
}

class LottoDrawPoc6Config implements SmartContractConfig {
    rpc = 'wss://poc6.phala.network/ws';
    address = '0x81a7816c71502c89a19b3588e1487e07a0b46acae3ebfc805dfe233cc82e5f26';
    metadata = "./metadata/lotto_draw_multichain.json";
}

class TestnetConfig implements Config {
    lottoManager = new LottoManagerShibuyaConfig();
    lottoDraw = new LottoDrawPoc6Config();
    lottoPhatContractAttestorAddress = 'YDiasEzv4mSVh5VJwXS94CHy9pRKNZfMJzdP5ChDP5RVKJk';
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

export function displayConfiguration(){
    console.log('Lotto Manager - RPC: %s', config.lottoManager.rpc);
    console.log('Lotto Manager - Address: %s', config.lottoManager.address);
    console.log('Lotto Draw - RPC: %s', config.lottoDraw.rpc);
    console.log('Lotto Draw - Address: %s', config.lottoDraw.address);
}

