import {SubmittableExtrinsic} from '@polkadot/api/types';
import type {ISubmittableResult} from '@polkadot/types/types';
import {KeyringPair} from "@polkadot/keyring/types";
import {setTimeout} from "timers/promises";


export async function signAndSend(
    extrinsic: SubmittableExtrinsic<'promise', ISubmittableResult>,
    signer : KeyringPair,
) : Promise<void> {

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
}

export type ExtrinsicResult = {
    success: boolean;
    failed: boolean;
    finalized: boolean;
}

function readResult(result: ISubmittableResult, extrinsicResult: ExtrinsicResult) : boolean {

    console.log('Transaction status:', result.status.type);

    if (result.status.isInBlock || result.status.isFinalized) {
        console.log('Transaction hash ', result.txHash.toHex());
        extrinsicResult.finalized = result.status.isFinalized;

        //result.events.forEach(({ phase, event : {data, method, section}} ) => {
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
            }
        });
    } else if (result.isError){
        console.log('Error');
        extrinsicResult.failed = true;
        return true;
    }
    return false;
}
