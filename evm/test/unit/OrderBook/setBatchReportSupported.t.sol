// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.33;

import { IAccessControl } from "../../../lib/common/lib/openzeppelin-contracts-upgradeable/lib/openzeppelin-contracts/contracts/access/IAccessControl.sol";

import { OrderBookTestBase } from "./OrderBookTestBase.t.sol";
import { IOrderBook } from "../../../src/interfaces/IOrderBook.sol";

contract SetBatchReportSupportedTest is OrderBookTestBase {
    // Test cases
    // [X] given the caller does not have the DEFAULT_ADMIN_ROLE
    //    [X] it reverts with an AccessControlUnauthorizedAccount error
    // [X] given the origin chain ID is the current chain
    //    [X] it reverts with a SameChainOrder error
    // [X] given a valid origin chain
    //    [X] it enables support and emits BatchReportSupportUpdated
    //    [X] it disables support and emits BatchReportSupportUpdated
    //    [X] setting the same value again does not emit

    function test_notAdmin_reverts() public {
        vm.prank(users["bob"]);
        vm.expectRevert(
            abi.encodeWithSelector(
                IAccessControl.AccessControlUnauthorizedAccount.selector,
                users["bob"],
                bytes32(0) // DEFAULT_ADMIN_ROLE
            )
        );
        orderBook.setBatchReportSupported(DEST_CHAIN_ID, true);
    }

    function test_sameChain_reverts() public {
        vm.prank(admin);
        vm.expectRevert(abi.encodeWithSelector(IOrderBook.SameChainOrder.selector));
        orderBook.setBatchReportSupported(CHAIN_ID, true);
    }

    function test_setAndUnset_success() public {
        assertFalse(orderBook.isBatchReportSupported(DEST_CHAIN_ID), "unsupported by default");

        vm.prank(admin);
        vm.expectEmit(true, false, false, true);
        emit IOrderBook.BatchReportSupportUpdated(DEST_CHAIN_ID, true);
        orderBook.setBatchReportSupported(DEST_CHAIN_ID, true);
        assertTrue(orderBook.isBatchReportSupported(DEST_CHAIN_ID), "enabled");

        // Same value: no event, no revert
        vm.recordLogs();
        vm.prank(admin);
        orderBook.setBatchReportSupported(DEST_CHAIN_ID, true);
        assertEq(vm.getRecordedLogs().length, 0, "no event when value is unchanged");

        vm.prank(admin);
        vm.expectEmit(true, false, false, true);
        emit IOrderBook.BatchReportSupportUpdated(DEST_CHAIN_ID, false);
        orderBook.setBatchReportSupported(DEST_CHAIN_ID, false);
        assertFalse(orderBook.isBatchReportSupported(DEST_CHAIN_ID), "disabled");
    }
}
