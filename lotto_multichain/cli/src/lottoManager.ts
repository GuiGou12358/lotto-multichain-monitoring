import {WeightV2} from '@polkadot/types/interfaces';
import {api, alice, signer, lottoManagerSmartContract, signAndSend} from './smartContractHelper';




export async function displayRaffleManagerData() : Promise<void>{

    console.log('Raffle Manager Contract');
    await getRaffleManagerDrawNumber();
    await getRaffleManagerStatus();
    console.log('Participation Registration ...');
    const contractIds = await getRegistrationContracts();
    for (let id of contractIds){
        await getRegistrationContractStatus(id);
    }

}

export async function getRaffleManagerDrawNumber() : Promise<Number>{

    // maximum gas to be consumed for the call. if limit is too small the call will fail.
    const gasLimit: WeightV2 = api.registry.createType('WeightV2',
        {refTime: 30000000000, proofSize: 1000000}
    );
    // a limit to how much Balance to be used to pay for the storage created by the contract call
    // if null is passed, unlimited balance can be used
    const storageDepositLimit = null;

    const {result, output} = await lottoManagerSmartContract.query['raffleManager::getDrawNumber'](alice.address, {gasLimit, storageDepositLimit});

    if (result.isOk){
        const value : string = output?.toString() ?? '';
        const drawNumber = JSON.parse(value).ok as number;
        console.log('Current draw number: %s', drawNumber);
        return drawNumber;
    }
    return Promise.reject("ERROR when query getDrawNumber " + result.asErr);
}


export async function getRaffleManagerStatus() : Promise<String> {

    // maximum gas to be consumed for the call. if limit is too small the call will fail.
    const gasLimit: WeightV2 = api.registry.createType('WeightV2',
        {refTime: 30000000000, proofSize: 1000000}
    );
    // a limit to how much Balance to be used to pay for the storage created by the contract call
    // if null is passed, unlimited balance can be used
    const storageDepositLimit = null;

    const {result, output} = await lottoManagerSmartContract.query['raffleManager::getStatus'](alice.address, {gasLimit, storageDepositLimit});

    if (result.isOk){
        const value : string = output?.toString() ?? '';
        const status = JSON.parse(value).ok as string;
        console.log('Current status: %s', status);
        return status;
    }
    return Promise.reject("ERROR when query getCurrentStatus " + result.asErr);
}

export async function getRegistrationContracts() : Promise<Number[]>{

    // maximum gas to be consumed for the call. if limit is too small the call will fail.
    const gasLimit: WeightV2 = api.registry.createType('WeightV2',
        {refTime: 30000000000, proofSize: 1000000}
    );
    // a limit to how much Balance to be used to pay for the storage created by the contract call
    // if null is passed, unlimited balance can be used
    const storageDepositLimit = null;

    const {result, output} = await lottoManagerSmartContract.query['raffleManager::getRegistrationContracts'](alice.address, {gasLimit, storageDepositLimit});

    if (result.isOk){
        const value : string = output?.toString() ?? '';
        const contractIds = JSON.parse(value).ok as number[];
        console.log('Registration contract ids: %s', contractIds);
        return contractIds;
    }
    return Promise.reject("ERROR when query getRegistrationContracts " + result.asErr);
}

export async function getRegistrationContractStatus(contractId: Number) : Promise<String>{

    // maximum gas to be consumed for the call. if limit is too small the call will fail.
    const gasLimit: WeightV2 = api.registry.createType('WeightV2',
        {refTime: 30000000000, proofSize: 1000000}
    );
    // a limit to how much Balance to be used to pay for the storage created by the contract call
    // if null is passed, unlimited balance can be used
    const storageDepositLimit = null;

    const {result, output} = await lottoManagerSmartContract.query['raffleManager::getRegistrationContractStatus'](alice.address, {gasLimit, storageDepositLimit}, contractId);

    if (result.isOk){
        const value : string = output?.toString() ?? '';
        const status = JSON.parse(value).ok as string;
        console.log('Status for contract %s: %s', contractId, status);
        return status;
    }
    return Promise.reject("ERROR when query getRegistrationContractStatus " + result.asErr);
}

export async function hasPendingMessage() : Promise<Boolean>{

    // maximum gas to be consumed for the call. if limit is too small the call will fail.
    const gasLimit: WeightV2 = api.registry.createType('WeightV2',
        {refTime: 30000000000, proofSize: 1000000}
    );
    // a limit to how much Balance to be used to pay for the storage created by the contract call
    // if null is passed, unlimited balance can be used
    const storageDepositLimit = null;

    const {result, output} = await lottoManagerSmartContract.query['hasPendingMessage'](alice.address, {gasLimit, storageDepositLimit});

    if (result.isOk){
        const value : string = output?.toString() ?? '';
        return JSON.parse(value).ok as boolean;
    }
    return Promise.reject("ERROR when query hasPendingMessage " + result.asErr);
}



export async function completeRaffle() : Promise<void>{

    const gasLimit: WeightV2 = api.registry.createType('WeightV2',
        {refTime: 7000000000, proofSize: 100000}
    );
    // a limit to how much Balance to be used to pay for the storage created by the contract call
    // if null is passed, unlimited balance can be used
    const storageDepositLimit = null;

    console.log('Signer address: %s', signer.address);
    const {gasRequired, result, debugMessage } =
        await lottoManagerSmartContract.query.completeRaffle(
            signer.address,
            { storageDepositLimit, gasLimit}
        ) ;

    if (result.isOk){
        const tx = lottoManagerSmartContract.tx.completeRaffle({ storageDepositLimit, gasLimit : gasRequired });
        await signAndSend(tx);
    } else {
        console.log('ERROR when completeRaffle - debug message: %s', debugMessage);
        return Promise.reject("ERROR when completeRaffle " + result.asErr);
    }

}


