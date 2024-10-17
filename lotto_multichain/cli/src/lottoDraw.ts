import {lottoPhatContract} from './phatContractHelper';
import {Keyring} from "@polkadot/api";
import {signCertificate} from "@phala/sdk";


export async function callPhatContract() : Promise<void>{

    const alice = new Keyring({ type: 'sr25519' }).addFromUri("//Alice")
    const aliceCertificate = await signCertificate({ pair: alice })

    const {result, output} = await lottoPhatContract.query['answerRequest'](alice.address, {cert: aliceCertificate});

    if (result.isOk){
        const value : string = output?.toString() ?? '';
        console.log('Result: %s', value);
        const res = JSON.parse(value).ok;
        if (res.err){
            console.log('Error: %s', res.err);
            return Promise.reject("ERROR when start the raffle");
        }
        console.log('Tx for the raffle: %s', res.ok);
        return;
    }
    return Promise.reject("ERROR when start the raffle " + result.asErr);
}
