// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract Config {

    uint8 public nbNumbers;
    uint public minNumber;
    uint public maxNumber;

    event ConfigUpdated(uint8 nbNumbers, uint minNumber, uint maxNumber);

    function _setConfig(uint8 _nbNumbers, uint _minNumber, uint _maxNumber) internal {
        // check the provided config
        require(_nbNumbers > 0, "Nb Numbers must be greater than 0");
        require(_maxNumber > _minNumber, "max must be greater than min");
        // save the config
        nbNumbers = _nbNumbers;
        minNumber = _minNumber;
        maxNumber = _maxNumber;
        // emit the event
        emit ConfigUpdated(nbNumbers, minNumber, maxNumber);
    }

    function _ensureConfig() internal view {
        require(nbNumbers > 0, "Config not set");
    }

    function _ensureSameConfig(uint8 _nbNumbers, uint _minNumber, uint _maxNumber) internal view {
        _ensureConfig();
        require(nbNumbers == _nbNumbers, "Different nb numbers");
        require(minNumber == _minNumber, "Different min number");
        require(maxNumber == _maxNumber, "Different max number");
    }

    function _checkNumbers(uint[] memory _numbers) internal view {
        // check the config is set
        _ensureConfig();
        // check the nb numbers
        require(_numbers.length == nbNumbers, "Incorrect nb numbers");
        // check the min and max
        for (uint i=0; i<_numbers.length; i++){
            require(_numbers[i] >= minNumber, "Number too low");
            require(_numbers[i] <= maxNumber, "Number too high");
        }
    }

}