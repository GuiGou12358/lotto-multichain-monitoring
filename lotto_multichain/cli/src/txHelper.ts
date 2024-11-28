import {SubmittableExtrinsic} from '@polkadot/api/types';
import type {ISubmittableResult} from '@polkadot/types/types';
import {KeyringPair} from "@polkadot/keyring/types";
import {setTimeout} from "timers/promises";
import {CodePromise} from "@polkadot/api-contract";

export async function instantiateWithCode(
  code : CodePromise,
  signer : KeyringPair,
) : Promise<string> {

    // maximum gas to be consumed for the call. if limit is too small the call will fail.
    const gasLimit = code.api.registry.createType('WeightV2',
      {refTime: 50000000000, proofSize: 1000000}
    );
    // a limit to how much Balance to be used to pay for the storage created by the contract call
    // if null is passed, unlimited balance can be used
    const storageDepositLimit = null;

    const extrinsic = code.tx.new({gasLimit, storageDepositLimit});
    return await signAndSend(extrinsic, signer);

}


export async function signAndSend(
    extrinsic: SubmittableExtrinsic<'promise', ISubmittableResult>,
    signer : KeyringPair,
) : Promise<string> {

    let extrinsicResult : ExtrinsicResult = {success: false, failed: false, finalized: false };

    const unsub = await extrinsic.signAndSend(
        signer,
        (result) => {
            if (readResult(result, extrinsicResult)) {
                unsub();
            }
        }
    );

    do {
        // wait 10 seconds
        await setTimeout(10000);
        // until the transaction has been finalized (or failed)
    } while (!extrinsicResult.failed && !extrinsicResult.finalized);

    if (extrinsicResult.failed){
        return Promise.reject("ERROR: Extrinsic failed");
    }

    return extrinsicResult.result;
}

export type ExtrinsicResult = {
    success: boolean;
    failed: boolean;
    finalized: boolean;
    result?: string;
}


function readResult(result: ISubmittableResult, extrinsicResult: ExtrinsicResult) : boolean {

    console.log('Transaction status:', result.status.type);

    if (result.status.isInBlock || result.status.isFinalized) {
        console.log('Transaction hash ', result.txHash.toHex());
        extrinsicResult.finalized = result.status.isFinalized;

        result.events.forEach(({ phase, event} ) => {
            let data = event.data;
            let method = event.method;
            let section = event.section;
            console.log(' %s : %s.%s:: %s', phase, section, method, data);

            if (section == 'system' && method == 'ExtrinsicSuccess'){
                extrinsicResult.success = true;
                return true;
            } else if (section == 'system' && method == 'ExtrinsicFailed'){
                extrinsicResult.failed = true;
                console.log(' %s : %s.%s:: %s', phase, section, method, data);
                return true;
            } else if (section == 'contracts' && method == 'Instantiated'){
                const [_owner, contract] = data;
                extrinsicResult.result = contract.toString();
            }
        });
    } else if (result.isError){
        console.log('Error');
        extrinsicResult.failed = true;
        return true;
    }
    return false;
}
