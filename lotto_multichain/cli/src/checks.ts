import {WeightV2} from '@polkadot/types/interfaces';
import {alice, api, lottoManagerSmartContract} from './smartContractHelper';
import {isDebug} from './lottoCli';
import {config} from './config';

export async function checkGrants() : Promise<void>{

    console.log('Check grants ... ');
  
    // maximum gas to be consumed for the call. if limit is too small the call will fail.
    const gasLimit: WeightV2 = api.registry.createType('WeightV2',
        {refTime: 6219235328, proofSize: 629760}
    );

    // a limit to how much Balance to be used to pay for the storage created by the contract call
    // if null is passed, unlimited balance can be used
    const storageDepositLimit = null;  

    const ROLE_GRANT_ATTESTOR = api.registry.createType('u32', 2852625541);

    const[hasRoleGrantAttestor] = await Promise.all([
        lottoManagerSmartContract.query['accessControl::hasRole'](
            alice.address, {gasLimit, storageDepositLimit},
            ROLE_GRANT_ATTESTOR, config.lottoPhatContractAttestorAddress
        ),
    ]);

    if (isDebug()){
        console.log('hasRoleGrantAttestor %s - %s', hasRoleGrantAttestor.result, hasRoleGrantAttestor.output);
    }
    
    if (hasRoleGrantAttestor.result.isOk){
        const value : string = hasRoleGrantAttestor.output?.toString() ?? '';
        const hasRole = JSON.parse(value).ok as Boolean;
        if (!hasRole){
            return Promise.reject("ERROR: the phat contract is not granted in the smart contract consumer");
        }
    } else {
        return Promise.reject("ERROR when query lottoSmartContract.accessControl::hasRole " + hasRoleGrantAttestor.result.asErr);
    }
    
    console.log('Check grants Ok');
}


export async function checkLottoConfiguration() : Promise<void>{

    console.log('Check Lotto Configuration ... ');

    // maximum gas to be consumed for the call. if limit is too small the call will fail.
    const gasLimit: WeightV2 = api.registry.createType('WeightV2',
        {refTime: 6219235328, proofSize: 629760}
    );

    // a limit to how much Balance to be used to pay for the storage created by the contract call
    // if null is passed, unlimited balance can be used
    const storageDepositLimit = null;

    const[
        getConfigOutcome,
    ] = await Promise.all([
        lottoManagerSmartContract.query['raffleConfig::getConfig'](
            alice.address, {gasLimit, storageDepositLimit}
        )
    ]);

    if (getConfigOutcome.result.isOk){
        const output : string = getConfigOutcome.output?.toString() ?? '';
        const config = JSON.parse(output).ok;
        console.log('getConfigOutcome: %s', config);
    } else {
        return Promise.reject('ERROR when query getConfig ' + getConfigOutcome.result.asErr);
    }

    console.log('Check configuration Ok');
}