import {getClient, getContract, OnChainRegistry, PinkContractPromise, signCertificate} from '@phala/sdk';
import {Keyring} from "@polkadot/api";
import LottoDrawMetadataWasm from "./metadata/lotto_draw_multichain.json";

const clients = new Map();

export async function reuseClient(rpc: string) : Promise<OnChainRegistry> {

    if (! clients.has(rpc)){

        const client = await getClient({
            transport: rpc
        });
        const[chain, nodeName, nodeVersion] = await Promise.all([
            client.api.rpc.system.chain(),
            client.api.rpc.system.name(),
            client.api.rpc.system.version()
        ]);
        console.log('You are connected to chain %s using %s v%s', chain, nodeName, nodeVersion);
        clients.set(rpc, client);
    }

    return clients.get(rpc);
}


export class LottoDraw {

    private readonly rpc: string;
    private readonly address: string;
    private contract: PinkContractPromise;

    public constructor(rpc: string, address: string) {
        this.rpc = rpc;
        this.address = address;
    }

    public async init(){

        if (this.contract){
            return;
        }

        const client = await reuseClient(this.rpc);

        this.contract = await getContract({
            client,
            contractId: this.address,
            abi: LottoDrawMetadataWasm,
            //provider,
          }
        );

    }

    public async synchronizeContracts() : Promise<void> {
        console.log('Raffle - Synchronise');
        return await query(this.contract, 'answerRequest');
    }

    public async closeRegistrations() : Promise<void> {
        console.log('Raffle - Close registrations');
        return await query(this.contract, 'closeRegistrations');
    }

}

export async function query(
  smartContract: PinkContractPromise,
  methodName: string,
  ...params: any[]
) : Promise<any> {

    const alice = new Keyring({ type: 'sr25519' }).addFromUri("//Alice");
    const certificate = await signCertificate({ pair: alice })

    const {result, output} = await smartContract.query[methodName](alice.address, {cert: certificate}, ...params);

    if (result.isOk){
        const value : string = output?.toString() ?? '';
        console.log('Query result: %s', value);
        const res = JSON.parse(value);
        if (res.err){
            return Promise.reject("Error during query pink contract : " + res.err);
        }
        return res.ok;
    }
    return Promise.reject("Error during query pink contract : " + result.asErr);
}

