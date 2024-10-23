import yargs from 'yargs/yargs';
import {displayConfiguration, initConfiguration} from './config';
import {initConnection as initPhatContractConnection} from './phatContractHelper';
import {checkGrants, checkLottoConfiguration} from './checks';
import {displayRaffleManagerData, hasPendingMessage,} from './lottoManager';
import {callPhatContract} from './lottoDraw';
import {initConnection as initSmartContractConnection} from "./smartContractHelper";
import {readSeed} from "./seed";

const argv = yargs(process.argv.slice(2)).options({
    dc: {alias: 'displayConfiguration', desc: 'Display the configuration (contract and http addresses)'},
    ch: {alias: 'checks', desc: 'Check if the grants and the configuration in the smart contracts have been set'},
    di: {alias: 'display', desc: 'Display information from indexer and smart contracts'},
    co:  {alias: 'synchronize', desc: 'Synchronize the status between smart contracts, draw the numbers, check winners'},
    net: {alias: 'network', choices:['testnet'], type:'string', desc: 'Specify the network', requiresArg: true},
    d: {alias: 'debug', desc: 'Debug mode: display more information'},
}).version('0.1').parseSync();


export function isDebug() : boolean{
    return argv.debug != undefined;
}

async function run() : Promise<void>{

    if (!argv.displayConfiguration && !argv.checks && !argv.display && !argv.synchronize
    ) {
        return Promise.reject('At least one option is required. Use --help for more information');
    }

    if (argv.net == undefined) {
        return Promise.reject('The network is mandatory');
    } else {
        initConfiguration(argv.net);
    }

    if (argv.displayConfiguration) {
        displayConfiguration();
    }

    readSeed(argv.net);
    await initSmartContractConnection();

    if (argv.display) {
        await displayRaffleManagerData();
    }

    if (argv.checks) {
        await checkGrants();
        await checkLottoConfiguration();
    }

    if (argv.synchronize) {
        await initPhatContractConnection();

        let nbErrors = 0;
        while (await hasPendingMessage()) {
            if (nbErrors > 10) {
                return Promise.reject("Stop the synchronization");
            }
            try {
                await callPhatContract();
                // wait 15 seconds and read again the status
                await new Promise(f => setTimeout(f, 15000));
                // display the data
                await displayRaffleManagerData();
                nbErrors = 0;
            } catch (e) {
                nbErrors +=1;
                // wait 10 seconds
                await new Promise(f => setTimeout(f, 10000));
            }
        }
    }

}

run().catch(console.error).finally(() => process.exit());


