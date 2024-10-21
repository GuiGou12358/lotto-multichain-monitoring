import {loadFixture} from "@nomicfoundation/hardhat-toolbox/network-helpers";
import {assert, expect} from "chai";
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

    console.log("openRegistrations");

    const request_bytes = abiCoder.encode(
        ['uint'],
        [drawNumber]
    );
    const action = abiCoder.encode(
        ['uint', 'bytes'],
        [RequestType.OPEN_REGISTRATIONS, request_bytes]
    );
    const reply = '0x00' + action.substring(2);
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

    const request_bytes = abiCoder.encode(
        ['uint', 'uint[]', 'address[]'],
        [drawNumber, numbers, winners]
    );
    const action = abiCoder.encode(
        ['uint', 'bytes'],
        [RequestType.SET_RESULTS, request_bytes]
    );
    const reply = '0x00' + action.substring(2);
    await expect(contract.connect(attestor).rollupU256CondEq([], [], [], [], [reply])).not.to.be.reverted;

    // check post conditions
    expect (await contract.getStatus()).to.equal(Status.ResultsReceived);
    expect (await contract.getDrawNumber()).to.equal(drawNumber);
    expect (await contract.can_participate()).to.equal(false);

    // check the storage for status
    expect ( await contract.getStorage("0x5f737461747573")).to.equal("0x0000000000000000000000000000000000000000000000000000000000000004");
    // check the storage for draw number
    expect ( await contract.getStorage("0x5f647261774e756d626572")).to.equal("0x000000000000000000000000000000000000000000000000000000000000000b")

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

  it('Attestor submits wrong results', async () => {
    const {contract, attestor, addr1} = await loadFixture(openRegistrationsFixture);

    // close the registrations for the draw number 11
    await closeRegistrations(contract, attestor, 11);

    // send the results : winning numbers are incorrect (too many numbers)
    await setResultsMustBeReverted(contract, attestor, 11, [33, 47, 5, 6, 40], [addr1.address]);
    // send the results : winning numbers are incorrect (not enough numbers)
    await setResultsMustBeReverted(contract, attestor, 11, [33, 47, 5], [addr1.address]);
    // send the results : winning numbers are incorrect (out of range)
    await setResultsMustBeReverted(contract, attestor, 11, [0, 47, 5, 8], [addr1.address]);
    // send the results : winning numbers are incorrect (out of range)
    await setResultsMustBeReverted(contract, attestor, 11, [1, 47, 51, 8], [addr1.address]);

  });

  async function setResultsMustBeReverted(
      contract: RaffleRegistration,
      attestor : Signer,
      drawNumber: number,
      numbers: number[],
      winners: string[]
  ) {

    const request_bytes = abiCoder.encode(
        ['uint', 'uint[]', 'address[]'],
        [drawNumber, numbers, winners]
    );
    const action = abiCoder.encode(
        ['uint', 'bytes'],
        [RequestType.SET_RESULTS, request_bytes]
    );
    const reply = '0x00' + action.substring(2);
    await expect(contract.connect(attestor).rollupU256CondEq([], [], [], [], [reply])).to.be.reverted;
  }

  it('should not config and start the raffle (unauthorized)', async () => {
    const {contract, owner, addr1} = await loadFixture(deployContractFixture);
    await expect(contract.connect(owner).rollupU256CondEq([], [], [], [], [])).to.be.reverted;
    await expect(contract.connect(addr1).rollupU256CondEq([], [], [], [], [])).to.be.reverted;
  });


  it('check hex - kv store', async () => {
    assert.equal(ethers.hexlify(ethers.toUtf8Bytes("_status")), "0x5f737461747573", "status key doesn't match");
    assert.equal(ethers.hexlify(ethers.toUtf8Bytes("_drawNumber")), "0x5f647261774e756d626572", "draw number key doesn't match");
  });

  it('check hex - config and start request', async () => {

    const request_bytes = abiCoder.encode(
      ['uint8', 'uint', 'uint', 'uint'],
      [4, 1, 50, 33]
    );
    const action = abiCoder.encode(
      ['uint8', 'bytes'],
      [RequestType.SET_CONFIG_AND_START, request_bytes]
    );
    const reply = '0x00' + action.substring(2);

    assert.equal(
      reply,
      "0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000320000000000000000000000000000000000000000000000000000000000000021",
      "reply doesn't match"
    );
  });

  it('check hex - open registration', async () => {

    const request_bytes = abiCoder.encode(
      ['uint'],
      [11]
    );
    const action = abiCoder.encode(
      ['uint', 'bytes'],
      [RequestType.OPEN_REGISTRATIONS, request_bytes]
    );
    const reply = '0x00' + action.substring(2);

    assert.equal(
      reply,
      "0x00000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000b",
      "reply doesn't match"
    );
  });

  it('check hex - close registration', async () => {

    const request_bytes = abiCoder.encode(
      ['uint'],
      [11]
    );
    const action = abiCoder.encode(
      ['uint', 'bytes'],
      [RequestType.CLOSE_REGISTRATIONS, request_bytes]
    );
    const reply = '0x00' + action.substring(2);

    assert.equal(
      reply,
      "0x00000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000b",
      "reply doesn't match"
    );
  });

  it('check hex - set results - no winner', async () => {

    const request_bytes = abiCoder.encode(
      ['uint', 'uint[]', 'address[]'],
      [11, [33, 47, 5, 6], []]
    );
    const action = abiCoder.encode(
      ['uint8', 'bytes'],
      [RequestType.SET_RESULTS, request_bytes]
    );
    const reply = '0x00' + action.substring(2);

    assert.equal(
      reply,
      "0x00000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000000000000000000000b0000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000021000000000000000000000000000000000000000000000000000000000000002f000000000000000000000000000000000000000000000000000000000000000500000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000000",
      "reply doesn't match"
    );
  });

  it('check hex - set results - 1 winner', async () => {

    const request_bytes = abiCoder.encode(
      ['uint', 'uint[]', 'address[]'],
      [11, [33, 47, 5, 6], ['0x3c44cdddb6a900fa2b585dd299e03d12fa4293bc']]
    );
    const action = abiCoder.encode(
      ['uint', 'bytes'],
      [RequestType.SET_RESULTS, request_bytes]
    );
    const reply = '0x00' + action.substring(2);

    assert.equal(
      reply,
      "0x00000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000140000000000000000000000000000000000000000000000000000000000000000b0000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000021000000000000000000000000000000000000000000000000000000000000002f0000000000000000000000000000000000000000000000000000000000000005000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000010000000000000000000000003c44cdddb6a900fa2b585dd299e03d12fa4293bc",
      "reply doesn't match"
    );
  });


});
