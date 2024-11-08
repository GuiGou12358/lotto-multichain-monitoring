import {ApiPromise, Keyring, WsProvider} from '@polkadot/api';
import {KeyringPair} from "@polkadot/keyring/types";
import {ContractPromise} from "@polkadot/api-contract";
import {signAndSend} from "./txHelper";


const apis = new Map();

export async function getApi(rpc: string) : Promise<ApiPromise> {

    if (! apis.has(rpc)){

        const api = await ApiPromise.create({ provider: new WsProvider(rpc)});
        const[chain, nodeName, nodeVersion] = await Promise.all([
            api.rpc.system.chain(),
            api.rpc.system.name(),
            api.rpc.system.version()
        ]);
        console.log('You are connected to chain %s using %s v%s', chain, nodeName, nodeVersion);
        apis.set(rpc, api);
    }

    return apis.get(rpc);
}

export async function query(
  smartContract: ContractPromise,
  methodName: string,
  ...params: any[]
) : Promise<any> {

    const alice = new Keyring({ type: 'sr25519' }).addFromUri("//Alice");

    // maximum gas to be consumed for the call. if limit is too small the call will fail.
    const gasLimit = smartContract.api.registry.createType('WeightV2',
      {refTime: 30000000000, proofSize: 1000000}
    );

    // a limit to how much Balance to be used to pay for the storage created by the contract call
    // if null is passed, unlimited balance can be used
    const storageDepositLimit = null;

    const {result, output}  = await smartContract.query[methodName](
      alice.address,
      {gasLimit, storageDepositLimit},
      ...params
    );

    if (result.isOk){
        const value : string = output?.toString() ?? '';
        return JSON.parse(value).ok;
    }
    return Promise.reject("ERROR when query " + result.asErr);
}


export async function tx(
  smartContract: ContractPromise,
  signer : KeyringPair,
  methodName: string,
  ...params: any[]
) : Promise<any> {

    // maximum gas to be consumed for the call. if limit is too small the call will fail.
    const gasLimit = smartContract.api.registry.createType('WeightV2',
      {refTime: 30000000000, proofSize: 1000000}
    );

    // a limit to how much Balance to be used to pay for the storage created by the contract call
    // if null is passed, unlimited balance can be used
    const storageDepositLimit = null;

    const {gasRequired, result, debugMessage } =
      await smartContract.query[methodName](
        signer.address,
        { storageDepositLimit, gasLimit},
        ...params
      ) ;

    if (result.isOk){
        const tx = smartContract.tx[methodName](
          { storageDepositLimit, gasLimit : gasRequired },
          ...params
        );
        await signAndSend(tx, signer);
    } else {
        console.log('Error when sending transaction - debugMessage : %s', debugMessage);
        return Promise.reject("Error when sending transaction " + result.asErr);
    }
}
