// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/utils/Address.sol";

contract RewardManager {

    uint public totalPendingRewards;
    mapping(address => uint) public pendingRewards;


    event PendingRewards(address indexed _winner, uint _rewards);
    event ClaimedRewards(address indexed _winner, uint _rewards);

    function _addWinners(address[] memory _winners) internal {

        // rewards by winner
        uint _rewards = (msg.value - totalPendingRewards) / _winners.length;

        for (uint i=0; i<_winners.length; i++){
            pendingRewards[_winners[i]] += _rewards;
            totalPendingRewards += _rewards;
            // emit the event
            emit PendingRewards(_winners[i], _rewards);
        }
    }

    function claim() external {
        address _address = msg.sender;
        _claimFrom(_address);
    }

    function claimFrom(address _address) external {
        _claimFrom(_address);
    }

    function _claimFrom(address _address) private {

        uint _rewards = pendingRewards[_address];
        require(_rewards > 0, "No reward");

        pendingRewards[_address] = 0;
        totalPendingRewards -= _rewards;

        // emit the event
        emit ClaimedRewards(_address, _rewards);

        // transfer the value
        Address.sendValue(payable(_address), _rewards);
    }

}