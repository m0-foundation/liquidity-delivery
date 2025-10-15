// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.26;

import { Test } from "../../lib/forge-std/src/Test.sol";

import { OrderBook, IOrderBook } from "../../src/OrderBook.sol";
import { MockMessenger } from "../mock/MockMessenger.t.sol";
import { MockERC20 } from "../mock/MockERC20.t.sol";

abstract contract UnitTestBase is Test {
    OrderBook internal orderBook;
    MockMessenger internal messenger;

    uint32 internal constant CHAIN_ID = 1;
    uint32 internal constant DEST_CHAIN_ID = 2;
    uint256 internal constant TOKEN_COUNT = 3;
    uint256 internal constant USER_COUNT = 3;
    uint256 internal constant MINT_AMOUNT = 100e6;
    uint128 internal constant AMOUNT_IN = 10e6;
    uint128 internal constant AMOUNT_OUT = 999e4;
    uint40 internal constant FILL_DURATION = 1 hours;

    mapping(uint256 => MockERC20) internal tokens;
    mapping(uint256 => address) internal users;

    IOrderBook.OnchainOrderParams internal params;

    function setUp() public virtual {
        messenger = new MockMessenger();
        orderBook = new OrderBook(CHAIN_ID, address(messenger));

        messenger.setOrderBook(address(orderBook));

        orderBook.setDestinationConfig(DEST_CHAIN_ID, true, uint40(10 minutes));

        // Deploy mock tokens
        for (uint256 i = 0; i < TOKEN_COUNT; i++) {
            // TODO test with different decimals
            tokens[i] = new MockERC20(string.concat("Token ", vm.toString(i + 1)), string.concat("T", vm.toString(i + 1)), 6);
        }

        // Create users
        for (uint256 i = 0; i < USER_COUNT; i++) {
            users[i] = address(uint160(uint256(keccak256(abi.encodePacked("user", i + 1)))));
        }

        // Deal eth and tokens to users
        for (uint256 i = 0; i < USER_COUNT; i++) {
            vm.deal(users[i], 1 ether);

            for (uint256 j = 0; j < TOKEN_COUNT; j++) {
                tokens[j].mint(users[i], MINT_AMOUNT);
            }
        }

        // Setup the standard order params used in tests
        params = IOrderBook.OnchainOrderParams({
            tokenIn: address(tokens[0]),
            destChainId: DEST_CHAIN_ID,
            tokenOut: bytes32(uint256(uint160(address(tokens[1])))),
            amountIn: AMOUNT_IN,
            amountOut: AMOUNT_OUT,
            recipient: bytes32(uint256(uint160(users[0]))),
            fillDeadline: uint40(block.timestamp) + FILL_DURATION,
            solver: bytes32(uint256(uint160(users[2])))
        });
    }
}