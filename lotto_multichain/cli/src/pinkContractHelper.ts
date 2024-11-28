import {PinkContractPromise, signCertificate} from '@phala/sdk'
import {KeyringPair} from "@polkadot/keyring/types";
import {signAndSend} from "./txHelper";
import {Keyring} from "@polkadot/api";

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


export async function tx(
  smartContract: PinkContractPromise,
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

    const certificate = await signCertificate({ pair: signer });

    const {result, output } =
      await smartContract.query[methodName](
        signer.address,
        {cert: certificate},
        ...params
      ) ;

    if (result.isOk){
        const value : string = output?.toString() ?? '';
        const res = JSON.parse(value).ok;
        if (res.err){
            console.log('Error when sending transaction - err : %s', res.err);
            return Promise.reject("Error when dry run the tx " + res.err);
        }

        const tx = smartContract.tx[methodName](
          { storageDepositLimit, gasLimit },
          ...params
        );
        await signAndSend(tx, signer);
    } else {
        console.log('Error when sending transaction - output : %s', output);
        return Promise.reject("Error when sending transaction " + result.asErr);
    }
}