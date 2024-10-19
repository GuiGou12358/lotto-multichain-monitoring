import {loadFixture} from "@nomicfoundation/hardhat-toolbox/network-helpers";
import {expect} from "chai";
import {ethers} from "hardhat";


// status
const STATUS_NOT_STARTED = 0;
const STATUS_ONGOING = 1;
const STATUS_WAITING_RESULTS = 2;
const STATUS_WAITING_WINNERS = 3;
const STATUS_CLOSED = 4;
// result type
const DRAW_NUMBERS = 0;
const CHECK_WINNERS = 1;

const abiCoder = ethers.AbiCoder.defaultAbiCoder();

describe('Test raffle life cycle', () => {

  async function deployContractFixture(){
    const [owner, attestor, addr1, addr2] = await ethers.getSigners();
    const lottoInstance = await ethers.deployContract("LottoClient", [owner.address]);
    return {lottoInstance, owner, attestor, addr1, addr2};
  }

  it('configure and start the raffle', async () => {
    const {lottoInstance, owner} = await loadFixture(deployContractFixture);

    expect (await lottoInstance.nbNumbers()).to.equal(0);
    expect (await lottoInstance.minNumber()).to.equal(0);
    expect (await lottoInstance.maxNumber()).to.equal(0);

    // set the raffle
    await expect(lottoInstance.connect(owner).setConfig(4, 1, 50)).not.to.be.reverted;

    expect (await lottoInstance.nbNumbers()).to.equal(4);
    expect (await lottoInstance.minNumber()).to.equal(1);
    expect (await lottoInstance.maxNumber()).to.equal(50);

    // check the status and the raffle id
    expect (await lottoInstance.status()).to.equal(STATUS_NOT_STARTED);
    expect (await lottoInstance.raffleId()).to.equal(0);

    // start the raffle
    await expect(lottoInstance.startRaffle()).not.to.be.reverted;

    // check the status and the raffle id
    expect (await lottoInstance.status()).to.equal(STATUS_ONGOING);
    expect (await lottoInstance.raffleId()).to.equal(1);

  });

  async function startRaffleFixture(){
    const {lottoInstance, owner, attestor, addr1, addr2} = await deployContractFixture();
    await lottoInstance.connect(owner).setConfig(4, 1, 50);
    await lottoInstance.connect(owner).startRaffle();
    await lottoInstance.connect(owner).registerAttestor(attestor.address);
    return {lottoInstance, owner, attestor, addr1, addr2};
  }

  it('participate', async () => {
    const {lottoInstance, owner, attestor, addr1, addr2} = await loadFixture(startRaffleFixture);
    await expect(lottoInstance.connect(owner).participate([1, 2, 3, 50])).not.to.be.reverted;
    await expect(lottoInstance.connect(attestor).participate([1, 2, 3, 50])).not.to.be.reverted;
    await expect(lottoInstance.connect(addr1).participate([1, 2, 3, 50])).not.to.be.reverted;
    await expect(lottoInstance.connect(addr1).participate([1, 2, 3, 50])).not.to.be.reverted;
    await expect(lottoInstance.connect(addr1).participate([10, 20, 30, 50])).not.to.be.reverted;
    await expect(lottoInstance.connect(addr2).participate([50, 2, 6, 1])).not.to.be.reverted;
  });

  it('should not be able to participate', async () => {
    const {lottoInstance} = await loadFixture(startRaffleFixture);
    await expect(lottoInstance.participate([1, 2, 3, 4, 5])).to.be.revertedWith('Incorrect nb numbers');
    await expect(lottoInstance.participate([1, 2, 3])).to.be.revertedWith('Incorrect nb numbers');
    await expect(lottoInstance.participate([0, 2, 3, 5])).to.be.revertedWith('Number too low');
    await expect(lottoInstance.participate([1, 2, 3, 51])).to.be.revertedWith('Number too high');
  });


  it('Complete the raffle, submit the results', async () => {
    const {lottoInstance, owner, attestor} = await loadFixture(startRaffleFixture);

    // check before
    expect(await lottoInstance.status()).to.equal(STATUS_ONGOING);

    // complete the raffle
    await expect(lottoInstance.connect(owner).completeRaffle()).not.to.be.reverted;

    // check after
    expect(await lottoInstance.status()).to.equal(STATUS_WAITING_RESULTS);
    expect(await lottoInstance.raffleId()).to.equal(1);

    // send the results
    const raffleId = 1;
    const request = abiCoder.encode(['uint8', 'uint', 'uint'], [4, 1, 50]);
    const response = abiCoder.encode(['uint[]'], [[33, 47, 5, 6]]);
    const action = abiCoder.encode(['uint', 'uint8', 'bytes', 'bytes'], [raffleId, DRAW_NUMBERS, request, response]);
    const reply = '0x00' + action.substring(2);
    await expect(lottoInstance.connect(attestor).rollupU256CondEq([], [], [], [], [reply])).not.to.be.reverted;

    // checks
    expect(await lottoInstance.status()).to.equal(STATUS_WAITING_WINNERS);
    expect(await lottoInstance.raffleId()).to.equal(1);

  });


  async function waitingWinnersFixture(){
    const {lottoInstance, owner, attestor, addr1, addr2} = await startRaffleFixture();
    // complete the raffle
    await lottoInstance.connect(owner).completeRaffle();
    // send the results
    const raffleId = 1;
    const request = abiCoder.encode(['uint8', 'uint', 'uint'], [4, 1, 50]);
    const response = abiCoder.encode(['uint[]'], [[33, 47, 5, 6]]);
    const action = abiCoder.encode(['uint', 'uint8', 'bytes', 'bytes'], [raffleId, DRAW_NUMBERS, request, response]);
    const reply = '0x00' + action.substring(2);
    await lottoInstance.connect(attestor).rollupU256CondEq([], [], [], [], [reply]);
    return {lottoInstance, owner, attestor, addr1, addr2};
  }


  it('Attestor submits no winner', async () => {
    const {lottoInstance, attestor} = await loadFixture(waitingWinnersFixture);

    // checks before
    expect(await lottoInstance.status()).to.equal(STATUS_WAITING_WINNERS);
    expect(await lottoInstance.raffleId()).to.equal(1);

    const raffleId = 1;
    const request = abiCoder.encode(['uint[]'], [[33, 47, 5, 6]]);
    const response = abiCoder.encode(['address[]'], [[]]);
    const action = abiCoder.encode(['uint', 'uint8', 'bytes', 'bytes'], [raffleId, CHECK_WINNERS, request, response]);
    const reply = '0x00' + action.substring(2);
    await expect(lottoInstance.connect(attestor).rollupU256CondEq([], [], [], [], [reply])).not.to.be.reverted;

    // checks
    expect(await lottoInstance.status()).to.equal(STATUS_ONGOING);
    expect(await lottoInstance.raffleId()).to.equal(2);
  });

  it('Attestor submits 1 winner', async () => {
    const {lottoInstance, attestor, addr1} = await loadFixture(waitingWinnersFixture);

    // checks before
    expect(await lottoInstance.status()).to.equal(STATUS_WAITING_WINNERS);
    expect(await lottoInstance.raffleId()).to.equal(1);

    const raffleId = 1;
    const request = abiCoder.encode(['uint[]'], [[33, 47, 5, 6]]);
    const response = abiCoder.encode(['address[]'], [[addr1.address]]);
    const action = abiCoder.encode(['uint', 'uint8', 'bytes', 'bytes'], [raffleId, CHECK_WINNERS, request, response]);
    const reply = '0x00' + action.substring(2);
    await expect(lottoInstance.connect(attestor).rollupU256CondEq([], [], [], [], [reply])).not.to.be.reverted;

    // checks
    expect(await lottoInstance.status()).to.equal(STATUS_CLOSED);
    expect(await lottoInstance.raffleId()).to.equal(1);
  });


  it('should not start the raffle (unauthorized)', async () => {
    const {lottoInstance, attestor, addr1} = await loadFixture(deployContractFixture);
    await expect(lottoInstance.connect(attestor).startRaffle()).to.be.reverted; //With('Custom error (could not decode)');
    await expect(lottoInstance.connect(addr1).startRaffle()).to.be.reverted; //With('Custom error (could not decode)');
  });

});
