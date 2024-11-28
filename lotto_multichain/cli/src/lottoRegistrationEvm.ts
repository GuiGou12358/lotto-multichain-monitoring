import {EvmContractCallConfig, RegistrationContractConfig} from "./config";
import {ethers, Wallet} from "ethers";
import {getProvider} from "./evmContractHelper";
import {Contract} from "ethers/lib.commonjs/contract/contract";
import {abi} from "../abi/RaffleRegistration.json";
import {seed_evm} from "./seed";
import {readFileSync} from "fs";

export class RaffleRegistrationEvm {

    private readonly config: RegistrationContractConfig;
    private contract: Contract;

    public constructor(config: RegistrationContractConfig){
        this.config = config;
    }

    public async instantiate() : Promise<void> {

        if (this.contract){
            return;
        }

        const bytecode = readFileSync("./abi/bytecode");

        const provider = await getProvider((this.config.contractConfig.call as EvmContractCallConfig).rpc);
        const signer = new ethers.Wallet(seed_evm, provider);

        const contractFactory = new ethers.ContractFactory(abi, bytecode, signer);
        const contract = await contractFactory.deploy(signer.address);
        await contract.deploymentTransaction().wait();

        const address = await contract.getAddress();
        this.config.contractConfig.address = address;
        console.log('New EVM Raffle Registration instantiated: %s', address);

        this.contract = new ethers.Contract(this.config.contractConfig.address, abi, provider);
    }

    public async connect(){

        if (this.contract){
            return;
        }
        const provider = await getProvider((this.config.contractConfig.call as EvmContractCallConfig).rpc);
        this.contract = new ethers.Contract(this.config.contractConfig.address, abi, provider);

    }

    public async display() {
        console.log('Raffle Registration %s - %s - %s',
          this.config.registrationContractId,
          this.config.contractConfig.address,
          (this.config.contractConfig.call as EvmContractCallConfig).rpc
        );
        const status = await this.getStatus();
        const drawNumber = await this.getDrawNumber();
        const registrationContractId = await this.getRegistrationContractId();
        console.log('Registration contract %s - Draw Number: %s - status %s', registrationContractId, drawNumber, status);
    }

    public async getStatus() : Promise<String> {
        return await this.contract.getStatus();
    }

    public async getDrawNumber() : Promise<Number> {
        return await this.contract.getDrawNumber();
    }

    public async getRegistrationContractId() : Promise<Number> {
        return await this.contract.registrationContractId();
    }

    public async registerAttestor(attestor: string) : Promise<void> {
        console.log('Raffle Registration %s - Register the attestor %s', this.config.registrationContractId, attestor);

        const provider = await getProvider((this.config.contractConfig.call as EvmContractCallConfig).rpc);
        const signer = new ethers.Wallet(seed_evm, provider);

        const tx = await this.contract.connect(signer).registerAttestor(attestor);
        await tx.wait();
    }

    public async hasAttestorRole(attestor: string) : Promise<boolean> {
        const attestorRole = ethers.keccak256(ethers.toUtf8Bytes("ATTESTOR_ROLE"));
        return await this.contract.hasRole(attestorRole, attestor);
    }
}