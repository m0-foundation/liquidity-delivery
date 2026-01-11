// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.26;

import { Script } from "../../lib/forge-std/src/Script.sol";
import { console } from "../../lib/forge-std/src/console.sol";

import { OrderBook } from "../../src/OrderBook.sol";
import { MockERC20 } from "../../test/mock/MockERC20.t.sol";

/**
 * @title DeployLocal
 * @notice Deployment script for local Docker development environment
 * @dev Deploys OrderBook and mock ERC20 tokens, mints to test user
 *
 * Environment variables:
 *   CHAIN_ID - The chain ID for this deployment
 *   DEST_CHAIN_ID - The destination chain ID for cross-chain config
 *   SOLVER_ADDRESS - Address of the solver/admin
 *   USER_ADDRESS - Address of the test user to receive tokens
 */
contract DeployLocal is Script {
    // Anvil's default funded account (account 0)
    uint256 constant ANVIL_PRIVATE_KEY = 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80;

    function run() external {
        uint32 chainId = uint32(vm.envUint("CHAIN_ID"));
        uint32 destChainId = uint32(vm.envUint("DEST_CHAIN_ID"));
        address solverAddress = vm.envAddress("SOLVER_ADDRESS");
        address userAddress = vm.envAddress("USER_ADDRESS");

        vm.startBroadcast(ANVIL_PRIVATE_KEY);

        // Deployer address (Anvil account 0)
        address deployer = vm.addr(ANVIL_PRIVATE_KEY);

        // Deploy OrderBook (no messenger for local testing)
        OrderBook orderBook = new OrderBook(chainId, address(0));

        // Initialize with deployer as admin first so we can configure
        orderBook.initialize(deployer);

        // Configure destination chain
        orderBook.setDestinationConfig(destChainId, true, 10);

        // Grant admin role to solver
        orderBook.grantRole(orderBook.DEFAULT_ADMIN_ROLE(), solverAddress);

        // Renounce deployer's admin role (solver is now the only admin)
        orderBook.renounceRole(orderBook.DEFAULT_ADMIN_ROLE(), deployer);

        console.log("OrderBook deployed at:", address(orderBook));

        // Deploy mock tokens
        MockERC20 usdc = new MockERC20("USD Coin", "USDC", 6);
        MockERC20 usdt = new MockERC20("Tether USD", "USDT", 6);

        console.log("USDC deployed at:", address(usdc));
        console.log("USDT deployed at:", address(usdt));

        // Mint tokens to test user (1000 tokens each with 6 decimals)
        uint256 mintAmount = 1000 * 10 ** 6;
        usdc.mint(userAddress, mintAmount);
        usdt.mint(userAddress, mintAmount);

        // Also mint to solver for filling orders
        usdc.mint(solverAddress, mintAmount);
        usdt.mint(solverAddress, mintAmount);

        console.log("Minted", mintAmount, "USDC and USDT to user:", userAddress);
        console.log("Minted", mintAmount, "USDC and USDT to solver:", solverAddress);

        vm.stopBroadcast();
    }
}
