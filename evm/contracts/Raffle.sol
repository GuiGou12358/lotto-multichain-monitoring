// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract Raffle {

    enum Status { NotStarted, Ongoing, WaitingResults, WaitingWinners, Closed }

    uint public currentRaffleId;
    Status public status;
    mapping (uint => uint[]) public results;
    mapping (uint => address[]) public winners;

    event RaffleStarted(uint indexed _raffleId);
    event RaffleEnded(uint indexed _raffleId);
    event ResultReceived(uint indexed _raffleId, uint[] _numbers);
    event WinnersRevealed(uint indexed _raffleId, address[] _winners);

    function _startNewRaffle() internal returns (uint){

        // check the status
        require(status == Status.NotStarted || status == Status.Closed, "Incorrect Status");
        // update the storage
        currentRaffleId += 1;
        status = Status.Ongoing;
        // emit the event
        emit RaffleStarted(currentRaffleId);

        return currentRaffleId;

    }

    function _stopCurrentRaffle() internal {

        // check the status
        require(status == Status.Ongoing, "Incorrect Status");
        // update the storage
        status = Status.WaitingResults;
        // emit the event
        emit RaffleEnded(currentRaffleId);
    }

    function _setResults(uint _raffleId, uint[] memory _results) internal {

        // check the raffle number
        require(currentRaffleId == _raffleId, "Incorrect Raffle Id");
        // check the status
        require(status == Status.WaitingResults, "Incorrect Status");
        // save the results
        results[_raffleId] = _results;
        // update the status
        status = Status.WaitingWinners;
        // emit the event
        emit ResultReceived(_raffleId, _results);
    }

    function _setWinners(uint _raffleId, address[] memory _winners) internal {

        // check the raffle number
        require(currentRaffleId == _raffleId, "Incorrect Raffle Id");
        // check the status
        require(status == Status.WaitingWinners, "Incorrect Status");
        // save the results
        winners[_raffleId] = _winners;
        // update the status
        status = Status.Closed;
        // emit the event
        emit WinnersRevealed(_raffleId, _winners);
    }


	function _ensureSameResults(uint _raffleId, uint[] memory _numbers) internal view {
		require(results[_raffleId].length == _numbers.length, "Different results");

        for (uint i=0; i<_numbers.length; i++){
            require(results[_raffleId][i] == _numbers[i], "Different results");
        }
	}


}