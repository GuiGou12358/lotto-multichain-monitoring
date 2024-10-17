import {readFileSync} from "fs";

export let seed : string;

export function readSeed(network: string) {
    if (network == 'testnet'){
        seed = readFileSync('.secret-testnet').toString().trim();
    } else {
        throw new Error("No config for this Network");
    }
}
