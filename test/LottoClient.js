//import { expect } from "chai";

const LottoClient = artifacts.require("LottoClient");

const web3 = require("web3");

// status
const STATUS_NOT_STARTED = 0;
const STATUS_ONGOING = 1;
const STATUS_WAITING_RESULTS = 2;
const STATUS_WAITING_WINNERS = 3;
const STATUS_CLOSED = 4;
// result type
const DRAW_NUMBERS = 0;
const CHECK_WINNERS = 1;

contract('Test raffle life cycle', (accounts) => {


  it('set the config', async () => {
    const lottoInstance = await LottoClient.deployed();
    await lottoInstance.setConfig(4, 1, 50, {from: accounts[0]});
  });

  it('start the raffle', async () => {
    const lottoInstance = await LottoClient.deployed();

    // check the status
    let status = await lottoInstance.status();
    assert.equal(status, STATUS_NOT_STARTED, "Incorrect status");

    // start the raffle
    await lottoInstance.startRaffle({from: accounts[0]});
    // check the status
    status = await lottoInstance.status();
    assert.equal(status, STATUS_ONGOING, "Incorrect status");
    // check the raffle id
    const raffleId = await lottoInstance.currentRaffleId();
    assert.equal(raffleId, 1, "Incorrect raffle id");

  });

  it('participate 1', async () => {
    const lottoInstance = await LottoClient.deployed();
    await lottoInstance.participate([1, 2, 3, 50]);
  });

  it('participate 2', async () => {
    const lottoInstance = await LottoClient.deployed();
    await lottoInstance.participate([1, 2, 3, 50], {from: accounts[1]});
    await lottoInstance.participate([10, 20, 30, 50], {from: accounts[1]});
    await lottoInstance.participate([50, 2, 6, 1], {from: accounts[1]});
  });

  it('participate 3', async () => {
    const lottoInstance = await LottoClient.deployed();
    await lottoInstance.participate([1, 20, 3, 50], {from: accounts[2]});
    await lottoInstance.participate([10, 2, 30, 50], {from: accounts[2]});
    await lottoInstance.participate([50, 2, 6, 1], {from: accounts[2]});
  });

  it('should not participate (Incorrect nb numbers) ', async () => {
    const lottoInstance = await LottoClient.deployed();

    try {
      await lottoInstance.participate([1, 2, 3, 4, 5])
      assert(false, "we should not be able to participate with Incorrect nb numbers");
    } catch (e) {
      //console.log(e)
    }
    //expect(await lottoInstance.participate([1, 2, 3, 4, 5])).to.be.revertedWithCustomError("Incorrect nb numbers");

  });

  it('should not participate (Incorrect min value) ', async () => {
    const lottoInstance = await LottoClient.deployed();

    try {
      await lottoInstance.participate([0, 2, 3, 4])
      assert(false, "we should not be able to participate with value = 0");
    } catch (e) {
      //console.log(e)
    }
    //expect(await lottoInstance.participate([1, 2, 3, 4, 5])).to.be.revertedWith("Incorrect nb numbers");

  });

  it('should not participate (Incorrect max value) ', async () => {
    const lottoInstance = await LottoClient.deployed();

    try {
      await lottoInstance.participate([51, 2, 3, 4])
      assert(false, "we should not be able to participate with value = 51");
    } catch (e) {
      //console.log(e)
    }
    //expect(await lottoInstance.participate([1, 2, 3, 4, 5])).to.be.revertedWith("Incorrect nb numbers");

  });

  it('Complete the raffle 1', async () => {
    const lottoInstance = await LottoClient.deployed();

    // check the status
    status = await lottoInstance.status();
    assert.equal(status, STATUS_ONGOING, "Incorrect status");

    // start the raffle
    await lottoInstance.completeRaffle({from: accounts[0]});
    status = await lottoInstance.status();

    // check the status
    status = await lottoInstance.status();
    assert.equal(status, STATUS_WAITING_RESULTS, "Incorrect status");
    // check the raffle id
    const raffleId = await lottoInstance.currentRaffleId();
    assert.equal(raffleId, 1, "Incorrect raffle id");
  });

  it('Grant Account 1 as attestor', async () => {
    const lottoInstance = await LottoClient.deployed();
    await lottoInstance.registerAttestor(accounts[1], {from: accounts[0]});
  });

  it('Attestor submits result 1', async () => {
    const lottoInstance = await LottoClient.deployed();
    const raffleId = 1;
    const request = web3.eth.abi.encodeParameters(['uint8', 'uint', 'uint'], [4, 1, 50]);
    const response = web3.eth.abi.encodeParameters(['uint[]'], [[33, 47, 5, 6]]);
    const action = web3.eth.abi.encodeParameters(['uint', 'uint8', 'bytes', 'bytes'], [raffleId, DRAW_NUMBERS, request, response]);
    const reply = '0x0' + action.substring(2);
    console.log(reply)
    //await debug(lottoInstance.rollupU256CondEq([], [], [], [], [action], {from: accounts[1]}));
    await lottoInstance.rollupU256CondEq([], [], [], [], [reply], {from: accounts[1]});

    // check the status
    const status = await lottoInstance.status();
    assert.equal(status, STATUS_WAITING_WINNERS, "Incorrect status");
  });

  it('Attestor submits no winner', async () => {
    const lottoInstance = await LottoClient.deployed();

    // check the raffle id
    const raffleId = 1;
    assert.equal(raffleId, await lottoInstance.currentRaffleId(), "Incorrect raffle id");

    const request = web3.eth.abi.encodeParameters(['uint[]'], [[33, 47, 5, 6]]);
    const response = web3.eth.abi.encodeParameters(['address[]'], [[]]);
    const action = web3.eth.abi.encodeParameters(['uint', 'uint8', 'bytes', 'bytes'], [raffleId, CHECK_WINNERS, request, response]);
    const reply = '0x0' + action.substring(2);
    console.log(reply)
    await lottoInstance.rollupU256CondEq([], [], [], [], [reply], {from: accounts[1]});

    // check the status  => we go back to ongoing
    const status = await lottoInstance.status();
    assert.equal(status, STATUS_ONGOING, "Incorrect status");
    // check the raffle id
    assert.equal(2, await lottoInstance.currentRaffleId(), "Incorrect raffle id");

  });

  it('participate 4', async () => {
    const lottoInstance = await LottoClient.deployed();
    await lottoInstance.participate([1, 20, 3, 50], {from: accounts[2]});
    await lottoInstance.participate([10, 2, 30, 50], {from: accounts[2]});
    await lottoInstance.participate([50, 2, 6, 1], {from: accounts[2]});
  });


  it('Complete the raffle 2', async () => {
    const lottoInstance = await LottoClient.deployed();

    await lottoInstance.completeRaffle({from: accounts[0]});
    status = await lottoInstance.status();

    // check the status
    status = await lottoInstance.status();
    assert.equal(status, STATUS_WAITING_RESULTS, "Incorrect status");
  });

  it('Attestor submits result 2', async () => {
    const lottoInstance = await LottoClient.deployed();
    const raffleId = 2;
    const request = web3.eth.abi.encodeParameters(['uint8', 'uint', 'uint'], [4, 1, 50]);
    const response = web3.eth.abi.encodeParameters(['uint[]'], [[50, 2, 6, 1]]);
    const action = web3.eth.abi.encodeParameters(['uint', 'uint8', 'bytes', 'bytes'], [raffleId, DRAW_NUMBERS, request, response]);
    const reply = '0x0' + action.substring(2);
    await lottoInstance.rollupU256CondEq([], [], [], [], [reply], {from: accounts[1]});

    // check the status
    const status = await lottoInstance.status();
    assert.equal(status, STATUS_WAITING_WINNERS, "Incorrect status");
  });

  it('Attestor submits a winner', async () => {
    const lottoInstance = await LottoClient.deployed();

    const raffleId = 2;
    assert.equal(raffleId, await lottoInstance.currentRaffleId(), "Incorrect raffle id");

    const request = web3.eth.abi.encodeParameters(['uint[]'], [[50, 2, 6, 1]]);
    const response = web3.eth.abi.encodeParameters(['address[]'], [[accounts[2]]]);
    const action = web3.eth.abi.encodeParameters(['uint', 'uint8', 'bytes', 'bytes'], [raffleId, CHECK_WINNERS, request, response]);
    const reply = '0x0' + action.substring(2);
    await lottoInstance.rollupU256CondEq([], [], [], [], [reply], {from: accounts[1]});

    // check the status  => closed
    const status = await lottoInstance.status();
    assert.equal(status, STATUS_CLOSED, "Incorrect status");
    // check the raffle id
    assert.equal(raffleId, await lottoInstance.currentRaffleId(), "Incorrect raffle id");
  });


});

contract('Test authorization', (accounts) => {
  it('should not start the raffle (unauthorized)', async () => {
    const lottoInstance = await LottoClient.deployed();
    try {
      await lottoInstance.startRaffle({from: accounts[0]});
      assert(false, "we should not be able to start with unauthorized account");
    } catch (e) {
      //console.log(e)
    }
    //expect(await lottoInstance.startRaffle({from: accounts[1]})).to.be.revertedWithCustomError("Custom error (could not decode)");

  });
});

