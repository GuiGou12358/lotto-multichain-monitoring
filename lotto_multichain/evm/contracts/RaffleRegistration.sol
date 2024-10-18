// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/access/AccessControl.sol";

import "./Config.sol";
import "./PhatRollupAnchor.sol";

contract RaffleRegistration is Config, Ownable, AccessControl, PhatRollupAnchor {

	// workflow status
	enum Status { NotStarted, Started, RegistrationsOpen, RegistrationsClosed, ResultsReceived }

	// Event emitted when the workflow starts
	event Started(uint indexed registrationContractId);

	// Event emitted when the registrations are open
	event RegistrationsOpen(uint indexed registrationContractId, uint indexed draw_number);

	// Event emitted when the registrations are closed
	event RegistrationsClosed(uint indexed registrationContractId, uint indexed draw_number);

	// Event emitted when the results are received
	event ResultsReceived(uint indexed registrationContractId, uint indexed draw_number, uint[] numbers, address[] winners);

	// Event emitted when the participation is registered
	event ParticipantRegistered(uint indexed registrationContractId, uint indexed draw_number, address indexed participant, uint[] numbers);

	// registration contract id, must be unique in all similar contracts deployed on different chains
	uint public registrationContractId;

	constructor(address _address)
		Ownable(_address)
	{
		_grantRole(DEFAULT_ADMIN_ROLE, _address);
	}

	function _start(uint _registrationContractId) private {
		// check the status
		require(getStatus() == Status.NotStarted, "Incorrect Status");
		// save data
		_setStatus(Status.Started);
		registrationContractId = _registrationContractId;
		// emit the event
		emit Started(registrationContractId);
	}

	function _open_registrations(uint _draw_number) private {
		// check the status
		Status status = getStatus();
		require(status == Status.Started || status == Status.ResultsReceived, "Incorrect Status");
		// save the data
		_setDrawNumber(_draw_number);
		_setStatus(Status.RegistrationsOpen);
		// emit the event
		emit RegistrationsOpen(registrationContractId, _draw_number);
	}

	function _close_registrations(uint _draw_number) private {
		// check the status
		require(getStatus() == Status.RegistrationsOpen, "Incorrect Status");
		// check the draw number
		require(getDrawNumber() == _draw_number, "Incorrect Draw Number");
		// save the data
		_setStatus(Status.RegistrationsClosed);
		// emit the event
		emit RegistrationsClosed(registrationContractId, _draw_number);
	}

	function _saveResults(uint _draw_number, uint[] memory _numbers, address[] memory _winners) private {
		// check the status
		require(getStatus() == Status.RegistrationsClosed, "Incorrect Status");
		// check the draw number
		require(getDrawNumber() == _draw_number, "Incorrect Draw Number");
		// save the results
		//results[_draw_number] = _numbers;
		//winners[_draw_number] = _winners;
		// update the status
		_setStatus(Status.ResultsReceived);
		// emit the event
		emit ResultsReceived(registrationContractId, _draw_number, _numbers, _winners);
	}

	// return true if the users can participate (ie register their numbers)
	function can_participate() public view returns (bool){
		return getStatus() == Status.RegistrationsOpen;
	}

	// participate, ie  register their numbers
	function participate(uint[] memory _numbers) external {
		// check is the user can participate
		require(can_participate(), "Incorrect Status");
		// check if the numbers are correct
		_checkNumbers(_numbers);

		// save the participation with an event
		address _participant = msg.sender;
		emit ParticipantRegistered(registrationContractId, getDrawNumber(), _participant, _numbers);
	}

	bytes public constant STATUS = "_status";

	// return the workflow status
	function getStatus() public view returns (Status){
		// get the status in the kv store
		return abi.decode(kvStore[STATUS], (Status));
	}

	function _setStatus(Status _status) private {
		// save the status in the kv store
		kvStore[STATUS] = abi.encode(_status);
	}

	bytes public constant DRAW_NUMBER = "_drawNumber";

	// return the draw number
	function getDrawNumber() public view returns (uint){
		// get the draw number in the kv store
		return abi.decode(kvStore[DRAW_NUMBER], (uint));
	}

	function _setDrawNumber(uint _drawNumber) private {
		// save the draw number in the kv store
		kvStore[DRAW_NUMBER] = abi.encode(_drawNumber);
	}

	// register a new attestor
	function registerAttestor(address _attestor) public virtual onlyRole(DEFAULT_ADMIN_ROLE){
		grantRole(PhatRollupAnchor.ATTESTOR_ROLE, _attestor);
	}

	enum RequestType {SET_CONFIG_AND_START, OPEN_REGISTRATIONS, CLOSE_REGISTRATIONS, SET_RESULTS}

	function _onMessageReceived(bytes calldata _action) internal override {

		(RequestType _requestType, bytes memory _request) = abi.decode(_action, (RequestType, bytes));

		require(
			_requestType == RequestType.SET_CONFIG_AND_START
		||  _requestType == RequestType.OPEN_REGISTRATIONS
		||  _requestType == RequestType.CLOSE_REGISTRATIONS
		||  _requestType == RequestType.SET_RESULTS,
		"cannot parse action");

		if (_requestType == RequestType.SET_CONFIG_AND_START){
			(uint8 _nbNumbers, uint _minNumber, uint _maxNumber, uint _registrationContractId) = abi.decode(_request, (uint8, uint, uint, uint));
			// save the config
			_setConfig(_nbNumbers, _minNumber, _maxNumber);
			// start the workflow
			_start(_registrationContractId);
		} else if (_requestType == RequestType.OPEN_REGISTRATIONS){
			(uint _draw_number) = abi.decode(_request, (uint));
			// open the registrations
			_open_registrations(_draw_number);
		} else if (_requestType == RequestType.CLOSE_REGISTRATIONS){
			(uint _draw_number) = abi.decode(_request, (uint));
			// close the registrations
			_close_registrations(_draw_number);
		} else if (_requestType == RequestType.SET_RESULTS){
			(uint _draw_number, uint[] memory _numbers, address[] memory _winners) = abi.decode(_request, (uint, uint[], address[]));
			// check if the numbers satisfies the config
			_checkNumbers(_numbers);
			// set the results
			_saveResults(_draw_number, _numbers, _winners);
		}

	}

}