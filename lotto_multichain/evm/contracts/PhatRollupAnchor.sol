// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "./MetaTransaction.sol";

// Uncomment this line to use console.log
// import "hardhat/console.sol";

/// Adds the Offchain Rollup functionalities to your contract
///
/// Phat Offchain Rollup Anchor implements the rollup functions to allow your contract to
/// integrate. It implements the basic kv-store, the rollup transaction handling, and also allow
/// you to interact with the Phat Contract in a request-response style.
///
/// ## Solidity Usage
///
/// ```solidity
/// contract ConsumerContract is PhatRollupAnchor {
///     constructor(address attestor) {
///         _grantRole(PhatRollupAnchor.ATTESTOR_ROLE, attestor);
///     }
///     function _onMessageReceived(bytes calldata action) internal override {
///         emit MsgReceived(action);
///     }
/// }
/// ```
///
/// Inherit this abstract contract in your consumer contract. To allow the Phat Contract to connect
/// to your consumer contract properly, you will need to specify `attestor`, an address generated
/// and controlled by the Phat Contract as its credential.
///
/// Add a attestor by `_grantRole()` as above. The attestors are controlled by OpenZeppelin's
/// `AccessControl` library. It allows to add and remove members to the role. You should have at
/// least one `attestor` to receive response from Phat Contract.
///
/// Then you should implement `_onMessageReceived()` to receive response. The parameter `action` is
/// the raw data provided by the Phat Contract. Usually it's encoded meaningful data in some
/// predefined schema (e.g. `abi.encode()`).
///
///
abstract contract PhatRollupAnchor is ReentrancyGuard, MetaTxReceiver, AccessControl {

    // Only submission from attestor is allowed.
    bytes32 public constant ATTESTOR_ROLE = keccak256("ATTESTOR_ROLE");

    event MetaTxDecoded();
    event MessageProcessedTo(uint256);

    error BadAttestor();
    error BadCondLen(uint kenLen, uint valueLen);
    error BadUpdateLen(uint kenLen, uint valueLen);
    error CondNotMet(bytes cond, uint32 expected, uint32 actual);
    error UnsupportedAction(uint8 actionId);
    error Internal_toUint32Strict_outOfBounds(bytes data);

    uint8 constant ACTION_REPLY = 0;

    mapping (bytes => bytes) kvStore;

    /// Triggers a rollup transaction with `eq` condition check on uint256 values
    ///
    /// - actions: Starts with one byte to define the action type and followed by the parameter of
    ///     the actions. Supported actions: ACTION_REPLY
    ///
    /// Note that calling from `address(this)` is allowed to make parameters a calldata. Don't
    /// abuse it.
    function rollupU256CondEq(
        bytes[] calldata condKeys,
        bytes[] calldata condValues,
        bytes[] calldata updateKeys,
        bytes[] calldata updateValues,
        bytes[] calldata actions
    ) public returns (bool) {
        // Allow meta tx to call itself
        if (msg.sender != address(this) && !hasRole(ATTESTOR_ROLE, msg.sender)) {
            revert BadAttestor();
        }
        return _rollupU256CondEqInternal(condKeys, condValues, updateKeys, updateValues, actions);
    }

    /// Triggers a rollup transaction similar to `rollupU256CondEq` but with meta-tx.
    ///
    /// Note to error handling. Most of the errors are propagated to the transaction error.
    /// However in case of out-of-gas, the error will not be propagated. It results in a bare
    /// "reverted" in etherscan. It's hard to debug, but you will find the gas is 100% used like
    /// [this tx](https://mumbai.polygonscan.com/tx/0x0abe643ada209ec31a0a6da4fab546b7071e1cf265f3b4681b9bede209c400c9).
    function metaTxRollupU256CondEq(
        ForwardRequest calldata req,
        bytes calldata signature
    ) public useMetaTx(req, signature) returns (bool) {
        if (!hasRole(ATTESTOR_ROLE, req.from)) {
            revert BadAttestor();
        }
        (
            bytes[] memory condKeys,
            bytes[] memory condValues,
            bytes[] memory updateKeys,
            bytes[] memory updateValues,
            bytes[] memory actions
        ) = abi.decode(req.data, (bytes[], bytes[], bytes[], bytes[], bytes[]));
        emit MetaTxDecoded();
        // Self-call to move memory bytes to calldata. Check "error handling" notes in docstring
        // to learn more.
        return this.rollupU256CondEq(condKeys, condValues, updateKeys, updateValues, actions);
    }

    function _rollupU256CondEqInternal(
        bytes[] calldata condKeys,
        bytes[] calldata condValues,
        bytes[] calldata updateKeys,
        bytes[] calldata updateValues,
        bytes[] calldata actions
    ) internal nonReentrant() returns (bool) {
        if (condKeys.length != condValues.length) {
            revert BadCondLen(condKeys.length, condValues.length);
        }
        if (updateKeys.length != updateValues.length) {
            revert BadUpdateLen(updateKeys.length, updateValues.length);
        }

        // check cond
        for (uint i = 0; i < condKeys.length; i++) {
            uint32 value = toUint32Strict(kvStore[condKeys[i]]);
            uint32 expected = toUint32Strict(condValues[i]);
            if (value != expected) {
                revert CondNotMet(condKeys[i], expected, value);
            }
        }

        // apply updates
        for (uint i = 0; i < updateKeys.length; i++) {
            kvStore[updateKeys[i]] = updateValues[i];
        }

        // apply actions
        for (uint i = 0; i < actions.length; i++) {
            handleAction(actions[i]);
        }

        return true;
    }

    function handleAction(bytes calldata action) private {
        uint8 actionType = uint8(action[0]);
        if (actionType == ACTION_REPLY) {
            _onMessageReceived(action[1:]);
        } else {
            revert UnsupportedAction(actionType);
        }
    }

    function getStorage(bytes memory key) public view returns(bytes memory) {
        return kvStore[key];
    }

    function toUint32Strict(bytes memory _bytes) public pure returns (uint32) {
        if (_bytes.length == 0) {
            return 0;
        }
        if (_bytes.length != 32) {
            revert Internal_toUint32Strict_outOfBounds(_bytes);
        }
        uint32 v = abi.decode(_bytes, (uint32));
        return v;
    }

    /// The handler to be called when a message is received from a Phat Contract
    ///
    /// Reverting in this function resulting the revert of the offchain rollup transaction.
    function _onMessageReceived(bytes calldata action) internal virtual;

}