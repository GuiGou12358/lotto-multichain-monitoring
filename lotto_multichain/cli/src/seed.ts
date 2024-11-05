import {readFileSync} from "fs";

export let seed_wasm : string;
export let seed_evm : string;

export function readSeed(network: string) {
    if (network == 'testnet'){
        seed_wasm = readFileSync('.secret-testnet-wasm').toString().trim();
        seed_evm = readFileSync('.secret-testnet-evm').toString().trim();
    } else {
        throw new Error("No config for this Network");
    }
}
