import {ethers, JsonRpcProvider,} from "ethers";


const providers = new Map();

export async function getProvider(rpc: string) : Promise<JsonRpcProvider> {

    if (! providers.has(rpc)){

        const provider = new ethers.JsonRpcProvider(rpc);
        console.log('You are connected to chain %s', rpc);
        providers.set(rpc, provider);
    }

    return providers.get(rpc);
}
