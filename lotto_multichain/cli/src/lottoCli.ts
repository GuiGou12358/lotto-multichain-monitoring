import yargs from 'yargs/yargs';
import {config, displayConfiguration, initConfiguration} from './config';
import {readSeed} from "./seed";
import {RaffleManager} from './lottoManager';
import {LottoDraw} from './lottoDraw';
import {RaffleRegistration} from "./lottoRegistrationWasm";

const argv = yargs(process.argv.slice(2)).options({
    dc: {alias: 'displayConfiguration', desc: 'Display the configuration (contract and http addresses)'},
    ch: {alias: 'checks', desc: 'Check if the grants and the configuration in the smart contracts have been set'},
    di: {alias: 'display', desc: 'Display information from indexer and smart contracts'},
    config: {alias: 'configure', desc: 'Configure smart contracts'},
    sync:  {alias: 'synchronize', desc: 'Synchronize the status between smart contracts, draw the numbers, check winners'},
    net: {alias: 'network', choices:['testnet'], type:'string', desc: 'Specify the network', requiresArg: true},
    d: {alias: 'debug', desc: 'Debug mode: display more information'},
}).version('0.1').parseSync();


export function isDebug() : boolean{
    return argv.debug != undefined;
}

async function run() : Promise<void>{

    if (!argv.displayConfiguration && !argv.checks && !argv.display && !argv.configure && !argv.synchronize
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

    const raffleManager = new RaffleManager(config.lottoManager);
    await raffleManager.init();

    const lottoDraw = new LottoDraw(config.lottoDraw);
    await lottoDraw.init();

    // get the attestor addresses
    const attestEcdsaAddressSubstrate = await lottoDraw.getAttestEcdsaAddressSubstrate();
    console.error("Attestor ECDSA Address for substrate contract : %s", attestEcdsaAddressSubstrate);
    const attestEcdsaAddressEvm = await lottoDraw.getAttestEcdsaAddressEvm();
    console.error("Attestor ECDSA Address for evm contract : " + attestEcdsaAddressEvm);

    if (argv.configure) {

        // Raffle manager - set the config
        await raffleManager.setConfig(config.raffleConfig);

        // Raffle manager - set the registration contract ids
        const registrationContractIds : Number[] = [];
        for (const raffleRegistrationConfig of config.lottoRegistrations){
            registrationContractIds.push(raffleRegistrationConfig.registrationContractId);
        }
        await raffleManager.setRegistrationContracts(registrationContractIds);

        // Raffle manager - grant the attestor
        if (! await raffleManager.hasAttestorRole(attestEcdsaAddressSubstrate) ) {
            await raffleManager.registerAttestor(attestEcdsaAddressSubstrate);
        }

        // lotto draw - set the raffle manager
        await lottoDraw.setRaffleManager(config.lottoManager);
        // lotto draw - set indexer
        await lottoDraw.configIndexer(config.indexer);

        // lotto draw - set the registration contracts
        for (const raffleRegistrationConfig of config.lottoRegistrations){
            await lottoDraw.setRaffleRegistration(raffleRegistrationConfig);
        }

        // Raffle registrations - grant the attestor
        const raffleRegistration = new RaffleRegistration(config.lottoRegistrations[0]);
        await raffleRegistration.init();
        if (! await raffleRegistration.hasAttestorRole(attestEcdsaAddressSubstrate) ) {
            await raffleRegistration.registerAttestor(attestEcdsaAddressSubstrate);
        }

    }

    if (argv.checks) {

        // check the attestor role
        if (! await raffleManager.hasAttestorRole(attestEcdsaAddressSubstrate) ) {
            console.error("Attestor not granted in the raffle manager");
        }

        const raffleRegistration = new RaffleRegistration(config.lottoRegistrations[0]);
        await raffleRegistration.init();
        if (! await raffleRegistration.hasAttestorRole(attestEcdsaAddressSubstrate) ) {
            console.error("Attestor not granted in the registration contract");
        }

    }

    if (argv.display) {
        await raffleManager.display();

        const raffleRegistration = new RaffleRegistration(config.lottoRegistrations[0]);
        await raffleRegistration.init();
        await raffleRegistration.display();
        /*
        for (const raffleRegistrationConfig of config.lottoRegistrations){
            const raffleRegistration = new RaffleRegistration(raffleRegistrationConfig);
            await raffleRegistration.init();
            await raffleRegistration.display();
        }
         */
    }

    if (argv.synchronize) {
        let nbErrors = 0;
        while (await raffleManager.hasPendingMessage()) {
            if (nbErrors > 10) {
                return Promise.reject("Stop the synchronization");
            }
            try {
                await lottoDraw.synchronize();
                // wait 30 seconds and read again the status
                await new Promise(f => setTimeout(f, 10000));
                // display the data
                await raffleManager.display();
                nbErrors = 0;
            } catch (e) {
                nbErrors +=1;
                // wait 30 seconds
                await new Promise(f => setTimeout(f, 10000));
            }
        }
    }

}

run().catch(console.error).finally(() => process.exit());


