import { loadFixture } from "@nomicfoundation/hardhat-network-helpers";
import { SignerWithAddress } from "@nomicfoundation/hardhat-ethers/signers";
import { expect } from "chai";
import { ethers } from "hardhat";
import {RaffleRegistration} from "../typechain-types";
import {Signer} from "ethers";

describe("RollupAnchor", function () {

  async function registerAttestor(contract: RaffleRegistration, owner : Signer, attestor : Signer){

    const ATTESTOR_ROLE = await contract.ATTESTOR_ROLE();

    // preconditions
    expect (await contract.hasRole(ATTESTOR_ROLE, attestor.getAddress())).to.equal(false);

    expect(await contract.connect(owner).registerAttestor(attestor)).not.to.be.reverted;

    // post condition
    expect (await contract.hasRole(ATTESTOR_ROLE, attestor.getAddress())).to.equal(true);
  }

  async function deployContractFixture(){
    const [owner, attestor, addr1, addr2] = await ethers.getSigners();

    // deploy the contract
    const contract = await ethers.deployContract("RaffleRegistration", [owner.address]);
    // register attestor
    await registerAttestor(contract, owner, attestor);

    return {contract, owner, attestor, addr1, addr2};
  }

  describe("Rollup", function () {
    it("Should not forward from random attestor", async function () {
      const { contract, owner } = await loadFixture(deployContractFixture);
      await expect(
          contract.connect(owner).rollupU256CondEq(
          // cond
          [], [],
          // updates
          [], [],
          // actions
          [],
        )
      ).to.be.revertedWithCustomError(contract, 'BadAttestor');
    });

    it("Should not allow invalid input arrays", async function () {
      const { contract, attestor } = await loadFixture(deployContractFixture);

      await expect(
          contract.connect(attestor).rollupU256CondEq(
          // cond
          ['0x01'], [],
          // updates
          [], [],
          // actions
          [],
        )
      ).to.be.revertedWithCustomError(contract, 'BadCondLen');

      await expect(
          contract.connect(attestor).rollupU256CondEq(
          // cond
          [], [],
          // updates
          ['0x'], [],
          // actions
          [],
        )
      ).to.be.revertedWithCustomError(contract, 'BadUpdateLen');
    });

    it("Should not allow incorrect action", async function () {
      const { contract, attestor } = await loadFixture(deployContractFixture);

      await expect(
        contract.connect(attestor).rollupU256CondEq(
          // cond
          [], [],
          // updates
          [], [],
          // actions
          ['0x09'],
        )
      ).to.be.revertedWithCustomError(contract, 'UnsupportedAction');
    });

    it("Should forward actions", async function () {
      const { contract, attestor } = await loadFixture(deployContractFixture);

      await expect(
        contract.connect(attestor).rollupU256CondEq(
          // cond
          ['0x01'],
          [encodeUint32(0)],
          // updates
          ['0x01'],
          [encodeUint32(1)],
          // actions
          [],
        )
      ).not.to.be.reverted;

      // check the storage
      expect(await contract.getStorage('0x01')).to.be.equals(encodeUint32(1));
    });
  });

  describe("OptimisticLock", function () {
    it("Should reject conflicting transaction", async function () {
      const { contract, attestor } = await loadFixture(deployContractFixture);
      // Rollup from v0 to v1
      await expect(
        contract.connect(attestor).rollupU256CondEq(
          // cond
          ['0x01'],
          [encodeUint32(0)],
          // updates
          ['0x01'],
          [encodeUint32(1)],
          // actions
          [],
        )
      ).not.to.be.reverted;
      expect(await contract.getStorage('0x01')).to.be.equals(encodeUint32(1));
      // Rollup to v1 again
      await expect(
        contract.connect(attestor).rollupU256CondEq(
          // cond
          ['0x01'],
          [encodeUint32(0)],
          // updates
          ['0x01'],
          [encodeUint32(1)],
          // actions
          [],
        )
      ).to.be
        .revertedWithCustomError(contract, 'CondNotMet')
        // We want to ensure 0x01 to match 0, but the value is 1.
        .withArgs('0x01', 0, 1);
    });
  });

  describe("Meta Transaction", function () {
    it("Process the request", async function () {
      const { contract, attestor, addr1 } = await loadFixture(deployContractFixture);
      // build the meta-transaction
      const [metaTxData1, metaTxSig1] = await metaTx([
        [], [], [], [],
        // Set the config (4 numbers between 1 and 50)
        ['0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000320000000000000000000000000000000000000000000000000000000000000021'],
      ], attestor, 0, await contract.getAddress());
      // Send meta-tx via addr1 on behalf of attestor
      const rollupTx = await contract.connect(addr1).metaTxRollupU256CondEq(metaTxData1, metaTxSig1);
      await expect(rollupTx).not.to.be.reverted;
      // check
      expect (await contract.nbNumbers()).to.equal(4);
      expect (await contract.minNumber()).to.equal(1);
      expect (await contract.maxNumber()).to.equal(50);

    })

    it("Should not be process the request", async function () {
      const { contract, owner, attestor, addr1 } = await loadFixture(deployContractFixture);
      // build the meta-transaction but not sign by the attestor
      const [metaTxData1, metaTxSig1] = await metaTx([
        [], [], [], [],
        // Set the config (4 numbers between 1 and 50)
        ['0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000320000000000000000000000000000000000000000000000000000000000000021'],
      ], addr1, 0, await contract.getAddress());
      // Send meta-tx
      const rollupTx = contract.connect(attestor).metaTxRollupU256CondEq(metaTxData1, metaTxSig1);
      await expect(rollupTx).to.be.revertedWithCustomError(contract, 'BadAttestor()');

    })

    it("Can propagate internal call error ( Open the registration before setting the config)", async function () {
      const { contract, attestor, addr1 } = await loadFixture(deployContractFixture);
      // build the meta-transaction
      const [metaTxData1, metaTxSig1] = await metaTx([
        [], [], [], [],
        // Open the registration
        ['0x00000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000b'],
      ], attestor, 0, await contract.getAddress());
      // Send meta-tx via addr1 on behalf of attestor
      const rollupTx = contract.connect(addr1).metaTxRollupU256CondEq(metaTxData1, metaTxSig1);
      await expect(rollupTx).to.be.revertedWith('Incorrect Status');
    })
  });
});


function abiEncode(type: string, value: any) {
  return ethers.AbiCoder.defaultAbiCoder().encode([type], [value]);
}
function encodeUint32(v: number) {
  return abiEncode('uint32', v);
}

interface MetaTxData {
  from: string;
  nonce: number;
  data: string;
};
type RollupParams = [string[], string[], string[], string[], string[]];

async function signMetaTx(signer: SignerWithAddress, contractAddress: string, value: MetaTxData) {
  // All properties on a domain are optional
  const domain = {
    name: 'PhatRollupMetaTxReceiver',
    version: '0.0.1',
    chainId: 31337,  // hardhat chain id
    verifyingContract: contractAddress
  };
  const types = {
    ForwardRequest: [
        { name: 'from', type: 'address' },
        { name: 'nonce', type: 'uint256' },
        { name: 'data', type: 'bytes' }
    ]
  };
  return await signer.signTypedData(domain, types, value);
}

async function metaTx(rollupParams: RollupParams, signer: SignerWithAddress, nonce: number, contractAddress: string): Promise<[MetaTxData, string]> {
  const data = ethers.AbiCoder.defaultAbiCoder().encode(
    ['bytes[]', 'bytes[]', 'bytes[]', 'bytes[]', 'bytes[]'],
    rollupParams,
  );
  const metaTxData = {
    from: signer.address,
    nonce,
    data,
  };
  const metaTxSig = await signMetaTx(signer, contractAddress, metaTxData);
  return [metaTxData, metaTxSig]
}
