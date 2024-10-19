import {loadFixture} from "@nomicfoundation/hardhat-toolbox/network-helpers";
import {expect} from "chai";
import {ethers} from "hardhat";
import {RaffleRegistration} from "../typechain-types";
import {Signer} from "ethers";


// workflow status
enum Status { NotStarted, Started, RegistrationsOpen, RegistrationsClosed, ResultsReceived }
// request type
enum RequestType {SET_CONFIG_AND_START, OPEN_REGISTRATIONS, CLOSE_REGISTRATIONS, SET_RESULTS}

const abiCoder = ethers.AbiCoder.defaultAbiCoder();

describe('Test raffle life cycle', () => {

  async function registerAttestor(contract: RaffleRegistration, owner : Signer, attestor : Signer){

    const ATTESTOR_ROLE = await contract.ATTESTOR_ROLE.staticCall();

    // preconditions
    expect (await contract.hasRole(ATTESTOR_ROLE, attestor.getAddress())).to.equal(false);

    expect(await contract.connect(owner).registerAttestor(attestor)).not.to.be.reverted;

    // post condition
    expect (await contract.hasRole(ATTESTOR_ROLE, attestor.getAddress())).to.equal(true);
  }

  async function setConfigAndStart(
      contract: RaffleRegistration,
      attestor : Signer,
      nbNumber: number,
      min: number,
      max : number,
      registrationContractId: number
  ) {

    // preconditions
    expect (await contract.nbNumbers()).to.equal(0);
    expect (await contract.minNumber()).to.equal(0);
    expect (await contract.maxNumber()).to.equal(0);
    expect (await contract.registrationContractId()).to.equal(0);
    console.log(await contract.getStatus());
    expect (await contract.getStatus()).to.equal(Status.NotStarted);
    expect (await contract.getDrawNumber()).to.equal(0);
    expect (await contract.can_participate()).to.equal(false);

    const request_bytes = abiCoder.encode(
        ['uint8', 'uint', 'uint', 'uint'],
        [nbNumber, min, max, registrationContractId]
    );
    const action = abiCoder.encode(
        ['uint8', 'bytes'],
        [RequestType.SET_CONFIG_AND_START, request_bytes]
    );
    const reply = '0x00' + action.substring(2);

    console.log("setConfigAndStart");
    console.log("reply %s", reply);
    await expect(contract.connect(attestor).rollupU256CondEq([], [], [], [], [reply])).not.to.be.reverted;

    // check post conditions
    expect (await contract.nbNumbers()).to.equal(nbNumber);
    expect (await contract.minNumber()).to.equal(min);
    expect (await contract.maxNumber()).to.equal(max);
    expect (await contract.registrationContractId()).to.equal(registrationContractId);
    expect (await contract.getStatus()).to.equal(Status.Started);
    expect (await contract.getDrawNumber()).to.equal(0);
    expect (await contract.can_participate()).to.equal(false);

  }

  async function openRegistrations(
      contract: RaffleRegistration,
      attestor : Signer,
      drawNumber: number
  ) {


    expect (await contract.can_participate()).to.equal(false);

    //const action = abiCoder.encode(['uint', 'uint'], [RequestType.OPEN_REGISTRATIONS, drawNumber]);
    //const action = abiCoder.encode(['uint', 'uint'], [1, drawNumber]);
    //const reply = '0x00' + action.substring(2);

    const request_bytes = abiCoder.encode(
        ['uint'],
        [drawNumber]
    );
    const action = abiCoder.encode(
        ['uint', 'bytes'],
        [RequestType.OPEN_REGISTRATIONS, request_bytes]
    );
    const reply = '0x00' + action.substring(2);

    console.log("openRegistrations");
    console.log("reply %s", reply);

    await expect(contract.connect(attestor).rollupU256CondEq([], [], [], [], [reply])).not.to.be.reverted;

    // check post conditions
    expect (await contract.getStatus()).to.equal(Status.RegistrationsOpen);
    expect (await contract.getDrawNumber()).to.equal(drawNumber);
    expect (await contract.can_participate()).to.equal(true);

  }

  async function closeRegistrations(
      contract: RaffleRegistration,
      attestor : Signer,
      drawNumber: number
  ) {

    // preconditions
    expect (await contract.getStatus()).to.equal(Status.RegistrationsOpen);
    expect (await contract.getDrawNumber()).to.equal(drawNumber);
    expect (await contract.can_participate()).to.equal(true);

    const request_bytes = abiCoder.encode(
        ['uint'],
        [drawNumber]
    );
    const action = abiCoder.encode(
        ['uint', 'bytes'],
        [RequestType.CLOSE_REGISTRATIONS, request_bytes]
    );
    const reply = '0x00' + action.substring(2);

    console.log("closeRegistrations");
    console.log("reply %s", reply);

    await expect(contract.connect(attestor).rollupU256CondEq([], [], [], [], [reply])).not.to.be.reverted;

    // check post conditions
    expect (await contract.getStatus()).to.equal(Status.RegistrationsClosed);
    expect (await contract.getDrawNumber()).to.equal(drawNumber);
    expect (await contract.can_participate()).to.equal(false);

  }

  async function setResults(
      contract: RaffleRegistration,
      attestor : Signer,
      drawNumber: number,
      numbers: number[],
      winners: string[]
  ) {

    // preconditions
    expect (await contract.getStatus()).to.equal(Status.RegistrationsClosed);
    expect (await contract.getDrawNumber()).to.equal(drawNumber);
    expect (await contract.can_participate()).to.equal(false);

    /*
    const action = abiCoder.encode(
        ['uint8', 'uint', 'uint[]', 'address[]'],
        [RequestType.SET_RESULTS, drawNumber, numbers, winners]
    );
    const reply = '0x00' + action.substring(2);
     */


    const request_bytes = abiCoder.encode(
        ['uint', 'uint[]', 'address[]'],
        [drawNumber, numbers, winners]
    );
    const action = abiCoder.encode(
        ['uint', 'bytes'],
        [RequestType.SET_RESULTS, request_bytes]
    );
    const reply = '0x00' + action.substring(2);

    console.log("closeRegistrations");
    console.log("reply %s", reply);


    await expect(contract.connect(attestor).rollupU256CondEq([], [], [], [], [reply])).not.to.be.reverted;

    // check post conditions
    expect (await contract.getStatus()).to.equal(Status.ResultsReceived);
    expect (await contract.getDrawNumber()).to.equal(drawNumber);
    expect (await contract.can_participate()).to.equal(false);

  }

  async function deployContractFixture(){
    const [owner, attestor, addr1, addr2] = await ethers.getSigners();

    // deploy the contract
    const contract = await ethers.deployContract("RaffleRegistration", [owner.address]);
    // register attestor
    await registerAttestor(contract, owner, attestor);

    return {contract, owner, attestor, addr1, addr2};
  }



  it('configure and open the registrations', async () => {
    const {contract, attestor} = await loadFixture(deployContractFixture);

    // config and start the raffle
    await setConfigAndStart(contract, attestor, 4, 1, 50, 33);

    // open the registrations for the draw number 11
    await openRegistrations(contract, attestor, 11);

  });


  async function openRegistrationsFixture(){
    const {contract, owner, attestor, addr1, addr2} = await deployContractFixture();

    // config and start the raffle
    await setConfigAndStart(contract, attestor, 4, 1, 50, 33);

    // open the registrations for the draw number 11
    await openRegistrations(contract, attestor, 11);

    return {contract, owner, attestor, addr1, addr2};
  }


  it('participate', async () => {
    const {contract, owner, attestor, addr1, addr2} = await loadFixture(openRegistrationsFixture);
    await expect(contract.connect(owner).participate([1, 2, 3, 50])).not.to.be.reverted;
    await expect(contract.connect(attestor).participate([1, 2, 3, 50])).not.to.be.reverted;
    await expect(contract.connect(addr1).participate([1, 2, 3, 50])).not.to.be.reverted;
    await expect(contract.connect(addr2).participate([1, 2, 3, 50])).not.to.be.reverted;
    await expect(contract.connect(addr1).participate([10, 20, 30, 50])).not.to.be.reverted;
    await expect(contract.connect(addr2).participate([50, 2, 6, 1])).not.to.be.reverted;
  });

  it('should not be able to participate', async () => {
    const {contract, addr1} = await loadFixture(openRegistrationsFixture);
    await expect(contract.connect(addr1).participate([1, 2, 3, 4, 5])).to.be.revertedWith('Incorrect nb numbers');
    await expect(contract.connect(addr1).participate([1, 2, 3])).to.be.revertedWith('Incorrect nb numbers');
    await expect(contract.connect(addr1).participate([0, 2, 3, 5])).to.be.revertedWith('Number too low');
    await expect(contract.connect(addr1).participate([1, 2, 3, 51])).to.be.revertedWith('Number too high');
  });


  it('Close the registrations and send the results (no winner)', async () => {
    const {contract, attestor} = await loadFixture(openRegistrationsFixture);

    // close the registrations for the draw number 11
    await closeRegistrations(contract, attestor, 11);

    // send the results (no winner)
    await setResults(contract, attestor, 11, [33, 47, 5, 6], []);

  });

  it('Attestor submits 1 winner', async () => {
    const {contract, attestor, addr1} = await loadFixture(openRegistrationsFixture);

    // close the registrations for the draw number 11
    await closeRegistrations(contract, attestor, 11);

    // send the results (no winner)
    await setResults(contract, attestor, 11, [33, 47, 5, 6], [addr1.address]);

  });


  it('should not config and start the raffle (unauthorized)', async () => {
    const {contract, owner, addr1} = await loadFixture(deployContractFixture);
    await expect(contract.connect(owner).rollupU256CondEq([], [], [], [], [])).to.be.reverted;
    await expect(contract.connect(addr1).rollupU256CondEq([], [], [], [], [])).to.be.reverted;
  });

});
