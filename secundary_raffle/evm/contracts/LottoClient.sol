// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/access/AccessControl.sol";

import "./Config.sol";
import "./Raffle.sol";
import "./RewardManager.sol";
import "./PhatRollupAnchor.sol";

contract LottoClient is Config, Raffle, Ownable, AccessControl, PhatRollupAnchor {

	event ParticipantRegistered(uint indexed _raffleId, address indexed _participant, uint[] _numbers);

	bytes32 public constant LOTTO_MANAGER_ROLE = keccak256("LOTTO_MANAGER_ROLE");

	constructor(address _address)
		Ownable(_address)
	{
		_grantRole(DEFAULT_ADMIN_ROLE, _address);
		_grantRole(LOTTO_MANAGER_ROLE, _address);
	}


	function participate(uint[] memory _numbers) public {
		// check the status
		require(status == Status.Ongoing, "Incorrect Status");
		// check the numbers
		_checkNumbers(_numbers);

		// save the participation
		address _participant = msg.sender;
		emit ParticipantRegistered(raffleId, _participant, _numbers);

	}


	function setConfig(uint8 _nbNumbers, uint _minNumber, uint _maxNumber) public onlyRole(LOTTO_MANAGER_ROLE) {
		// check the status
		require(status == Status.NotStarted, "Incorrect Status");
		// save the config
		_setConfig(_nbNumbers, _minNumber, _maxNumber);
	}

	function startRaffle() public onlyRole(LOTTO_MANAGER_ROLE) {
		// check the config is set
		_ensureConfig();
		// start the raffle
		_startNewRaffle();
	}

	function completeRaffle() public onlyRole(LOTTO_MANAGER_ROLE) {
		// stop the raffle
		_stopCurrentRaffle();
		// request the draw numbers
		_pushMessage(abi.encode(raffleId, RequestType.DRAW_NUMBERS, nbNumbers, minNumber, maxNumber));
	}

	bytes public constant LAST_RAFFLE_DONE = "_lastRaffleDone";

	function _innerSetResults(uint _raffleId, uint8 _nbNumbers, uint _minNumber, uint _maxNumber, uint[] memory _results) internal {

		// check if the config used to select the numbers is correct
		_ensureSameConfig(_nbNumbers, _minNumber, _maxNumber);
		// check if the numbers are correct
		_checkNumbers(_results);
		// set the results
		_setResults(_raffleId, _results);
		// save in the kv store the last raffle id used for verification
		kvStore[LAST_RAFFLE_DONE] = abi.encode(_raffleId);
		// request to check the winners
		_pushMessage(abi.encode(_raffleId, RequestType.CHECK_WINNERS, _results));

	}

	function _innerSetWinners(uint _raffleId, uint[] memory _numbers, address[] memory _winners) internal {

		// check if the winners were selected based on the correct numbers
		_ensureSameResults(_raffleId, _numbers);
		// set the winners
		_setWinners(_raffleId, _winners);

		if (_winners.length == 0){
			// no winner => start new raffle
			_startNewRaffle();
		} else {
			// save the winners in the reward manager
			//_addWinners(_winners);
		}
	}

	function registerAttestor(address _attestor) public virtual {
		grantRole(PhatRollupAnchor.ATTESTOR_ROLE, _attestor);
	}

	enum RequestType {DRAW_NUMBERS, CHECK_WINNERS}

	/*
	struct LottoResponseMessage {
		// raffle id
		uint raffleId;
		// initial request type
		RequestType requestType;
		// initial request data :
		// RequestType == DRAW_NUMBERS : (uint8 nbNumbers, uint minNumber, uint maxNumber)
		// or
		// RequestType == CHECK_WINNERS : (uint[] numbers)
		bytes request;
		// response data :
		// RequestType == DRAW_NUMBERS :  (uint[] numbers)
		// or
		// RequestType == CHECK_WINNERS :  (address[] winners)
		bytes response;
	}
	*/


	function _onMessageReceived(bytes calldata _action) internal override {

		(uint _raffleId, RequestType _requestType, bytes memory _request, bytes memory _response) = abi.decode(_action, (uint, RequestType, bytes, bytes));

		require(_requestType == RequestType.DRAW_NUMBERS ||  _requestType == RequestType.CHECK_WINNERS, "cannot parse action");
		if (_requestType == RequestType.DRAW_NUMBERS){
			(uint8 _nbNumbers, uint _minNumber, uint _maxNumber) = abi.decode(_request, (uint8, uint , uint));
			(uint[] memory _numbers) = abi.decode(_response, (uint[]));
			_innerSetResults(_raffleId, _nbNumbers, _minNumber, _maxNumber, _numbers);
		} else if (_requestType == RequestType.CHECK_WINNERS){
			(uint[] memory _numbers) = abi.decode(_request, (uint[]));
			(address[] memory _winners) = abi.decode(_response, (address[]));
			_innerSetWinners(_raffleId, _numbers, _winners);
		}

	}


}