// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.33;

import { TypeConverter } from "../../../lib/common/src/libs/TypeConverter.sol";

import { OrderBookTestBase } from "./OrderBookTestBase.t.sol";
import { IOrderBook } from "../../../src/interfaces/IOrderBook.sol";

contract SendFillReportsTest is OrderBookTestBase {
    using TypeConverter for *;

    // Test cases
    // [X] given the input array lengths differ
    //    [X] it reverts with an ArrayLengthMismatch error
    // [X] given more than MAX_FILL_REPORTS_PER_BATCH entries
    //    [X] it reverts with a TooManyReports error
    // [X] given an order whose originChainId differs from the batch origin chain
    //    [X] it reverts with an InvalidReportSource error
    // [X] given an order whose destChainId is not this chain
    //    [X] it reverts with an InvalidDestinationChain error
    // [X] given none of the entries has a pending report
    //    [X] it reverts with a NoPendingReports error
    // [X] given a mix of pending and already-drained entries
    //    [X] drained entries are skipped and the rest are sent
    // [X] given multiple orders with pending reports
    //    [X] it drains all pending records and sends one portal message
    //    [X] it emits a PendingFillReported event per drained entry
    //    [X] it forwards msg.value and the refund address to the portal
    // [X] given the caller is not the solver
    //    [X] anyone can send pending reports (permissionless)
    // [X] given the contract is paused
    //    [X] sendFillReports remains callable (upgrade-runbook flush)
    // [X] given batch report support was disabled after deferral
    //    [X] existing pending records remain drainable

    uint128 internal FILL_AMOUNT = AMOUNT_OUT / 2;

    function setUp() public override {
        super.setUp();

        vm.prank(admin);
        orderBook.setBatchReportSupported(DEST_CHAIN_ID, true);

        vm.startPrank(users["solver"]);
        tokens["token-out-6D"].approve(address(orderBook), type(uint256).max);
        vm.stopPrank();
    }

    function _foreignOrderData(uint64 nonce_) internal view returns (IOrderBook.OrderData memory) {
        return
            IOrderBook.OrderData({
                version: 1,
                originChainId: DEST_CHAIN_ID,
                sender: users["alice"].toBytes32(),
                nonce: nonce_,
                destChainId: CHAIN_ID,
                createdAt: uint64(block.timestamp),
                fillDeadline: uint64(block.timestamp + FILL_DURATION),
                amountIn: AMOUNT_IN,
                amountOut: AMOUNT_OUT,
                tokenIn: address(tokenIn).toBytes32(),
                tokenOut: address(tokenOut).toBytes32(),
                recipient: users["alice"].toBytes32(),
                solver: bytes32(0)
            });
    }

    /// @dev Opens `count_` foreign orders and defers a fill for each; returns the order data array
    function _deferFills(uint256 count_) internal returns (IOrderBook.OrderData[] memory ordersData_) {
        ordersData_ = new IOrderBook.OrderData[](count_);
        for (uint256 i; i < count_; ++i) {
            ordersData_[i] = _foreignOrderData(uint64(i));
            bytes32 orderId_ = orderBook.getOrderId(ordersData_[i]);
            vm.prank(users["solver"]);
            orderBook.fillOrderWithoutReport(
                orderId_,
                ordersData_[i],
                IOrderBook.FillParams({
                    amountOutToFill: FILL_AMOUNT,
                    originRecipient: users["solver"].toBytes32(),
                    refundAddress: bytes32(0)
                })
            );
        }
    }

    function _recipients(uint256 count_) internal view returns (bytes32[] memory recipients_) {
        recipients_ = new bytes32[](count_);
        for (uint256 i; i < count_; ++i) {
            recipients_[i] = users["solver"].toBytes32();
        }
    }

    function test_arrayLengthMismatch_reverts() public {
        IOrderBook.OrderData[] memory ordersData = _deferFills(1);

        vm.expectRevert(abi.encodeWithSelector(IOrderBook.ArrayLengthMismatch.selector));
        orderBook.sendFillReports(DEST_CHAIN_ID, ordersData, _recipients(2), bytes32(0), address(0), new bytes(0));
    }

    function test_tooManyReports_reverts() public {
        uint256 tooMany = orderBook.MAX_FILL_REPORTS_PER_BATCH() + 1;
        IOrderBook.OrderData[] memory ordersData = new IOrderBook.OrderData[](tooMany);
        for (uint256 i; i < tooMany; ++i) {
            ordersData[i] = _foreignOrderData(uint64(i));
        }

        vm.expectRevert(abi.encodeWithSelector(IOrderBook.TooManyReports.selector));
        orderBook.sendFillReports(
            DEST_CHAIN_ID,
            ordersData,
            _recipients(tooMany),
            bytes32(0),
            address(0),
            new bytes(0)
        );
    }

    function test_mixedOriginChain_reverts() public {
        IOrderBook.OrderData[] memory ordersData = _deferFills(2);
        // Second order claims a different origin chain than the batch
        ordersData[1].originChainId = 3;

        vm.expectRevert(abi.encodeWithSelector(IOrderBook.InvalidReportSource.selector));
        orderBook.sendFillReports(DEST_CHAIN_ID, ordersData, _recipients(2), bytes32(0), address(0), new bytes(0));
    }

    function test_wrongDestinationChain_reverts() public {
        IOrderBook.OrderData[] memory ordersData = _deferFills(1);
        // Order not destined for this chain
        ordersData[0].destChainId = 3;

        vm.expectRevert(abi.encodeWithSelector(IOrderBook.InvalidDestinationChain.selector));
        orderBook.sendFillReports(DEST_CHAIN_ID, ordersData, _recipients(1), bytes32(0), address(0), new bytes(0));
    }

    function test_nothingPending_reverts() public {
        // Valid orders, but no deferred fills were recorded
        IOrderBook.OrderData[] memory ordersData = new IOrderBook.OrderData[](2);
        ordersData[0] = _foreignOrderData(0);
        ordersData[1] = _foreignOrderData(1);

        vm.expectRevert(abi.encodeWithSelector(IOrderBook.NoPendingReports.selector));
        orderBook.sendFillReports(DEST_CHAIN_ID, ordersData, _recipients(2), bytes32(0), address(0), new bytes(0));
    }

    function test_multiOrderBatch_success() public {
        IOrderBook.OrderData[] memory ordersData = _deferFills(3);
        bytes32 solverRecipient = users["solver"].toBytes32();
        uint128 expectedAmountIn = uint128((uint256(AMOUNT_IN) * FILL_AMOUNT) / AMOUNT_OUT);

        // One PendingFillReported per drained entry (messageId topic not checked; it is computed in-flight)
        for (uint256 i; i < 3; ++i) {
            vm.expectEmit(true, true, false, true);
            emit IOrderBook.PendingFillReported(
                orderBook.getOrderId(ordersData[i]),
                solverRecipient,
                expectedAmountIn,
                FILL_AMOUNT,
                bytes32(0)
            );
        }
        bytes32 messageId = orderBook.sendFillReports(
            DEST_CHAIN_ID,
            ordersData,
            _recipients(3),
            bytes32(0),
            address(0),
            new bytes(0)
        );
        assertTrue(messageId != bytes32(0), "messageId should be returned");

        // Single portal message carrying all three reports, to the origin chain
        assertEq(portal.getLastBatchLength(), 3, "portal should have received 3 reports in one batch");
        assertEq(portal.lastBatchDestinationChainId(), DEST_CHAIN_ID, "batch should target the origin chain");

        // Reports carry the exact pending amounts and hash-verified tokenIn
        for (uint256 i; i < 3; ++i) {
            bytes32 orderId = orderBook.getOrderId(ordersData[i]);
            (bytes32 rOrderId, uint128 rIn, uint128 rOut, bytes32 rRecipient, bytes32 rTokenIn) = portal.fillReports(
                orderId
            );
            assertEq(rOrderId, orderId, "report orderId");
            assertEq(rIn, expectedAmountIn, "report amountInToRelease");
            assertEq(rOut, FILL_AMOUNT, "report amountOutFilled");
            assertEq(rRecipient, solverRecipient, "report originRecipient");
            assertEq(rTokenIn, ordersData[i].tokenIn, "report tokenIn");

            // Pending record fully drained
            IOrderBook.PendingFillReport memory pending = orderBook.getPendingFillReport(orderId, solverRecipient);
            assertEq(pending.amountInToRelease, 0, "pending drained (in)");
            assertEq(pending.amountOutFilled, 0, "pending drained (out)");
        }
    }

    function test_aggregatedFills_produceSingleReport() public {
        // Two deferred fills of the same order aggregate into one pending record and one report
        IOrderBook.OrderData memory orderData = _foreignOrderData(0);
        bytes32 orderId = orderBook.getOrderId(orderData);
        bytes32 solverRecipient = users["solver"].toBytes32();

        for (uint256 i; i < 2; ++i) {
            vm.prank(users["solver"]);
            orderBook.fillOrderWithoutReport(
                orderId,
                orderData,
                IOrderBook.FillParams({
                    amountOutToFill: FILL_AMOUNT / 2,
                    originRecipient: solverRecipient,
                    refundAddress: bytes32(0)
                })
            );
        }

        IOrderBook.PendingFillReport memory pending = orderBook.getPendingFillReport(orderId, solverRecipient);

        IOrderBook.OrderData[] memory ordersData = new IOrderBook.OrderData[](1);
        ordersData[0] = orderData;
        orderBook.sendFillReports(DEST_CHAIN_ID, ordersData, _recipients(1), bytes32(0), address(0), new bytes(0));

        assertEq(portal.getLastBatchLength(), 1, "aggregated fills should produce a single report");
        (, uint128 rIn, uint128 rOut, , ) = portal.fillReports(orderId);
        assertEq(rIn, pending.amountInToRelease, "report carries the aggregated amountInToRelease");
        assertEq(rOut, pending.amountOutFilled, "report carries the aggregated amountOutFilled");
    }

    function test_drainedEntriesAreSkipped() public {
        IOrderBook.OrderData[] memory ordersData = _deferFills(2);

        // Drain the first entry ahead of the batch (e.g. a front-running keeper)
        IOrderBook.OrderData[] memory firstOnly = new IOrderBook.OrderData[](1);
        firstOnly[0] = ordersData[0];
        orderBook.sendFillReports(DEST_CHAIN_ID, firstOnly, _recipients(1), bytes32(0), address(0), new bytes(0));

        // The full batch still succeeds, sending only the remaining entry
        orderBook.sendFillReports(DEST_CHAIN_ID, ordersData, _recipients(2), bytes32(0), address(0), new bytes(0));
        assertEq(portal.getLastBatchLength(), 1, "already-drained entry should be skipped");
        (bytes32 rOrderId, , , , ) = portal.fillReports(orderBook.getOrderId(ordersData[1]));
        assertEq(rOrderId, orderBook.getOrderId(ordersData[1]), "remaining entry should be sent");
    }

    function test_permissionlessCaller_success() public {
        IOrderBook.OrderData[] memory ordersData = _deferFills(1);

        // Bob (not the solver) flushes the solver's pending report
        vm.prank(users["bob"]);
        orderBook.sendFillReports(DEST_CHAIN_ID, ordersData, _recipients(1), bytes32(0), address(0), new bytes(0));

        // The report still pays out to the recipient fixed at fill time
        (, , , bytes32 rRecipient, ) = portal.fillReports(orderBook.getOrderId(ordersData[0]));
        assertEq(rRecipient, users["solver"].toBytes32(), "payout recipient is fixed at fill time");
    }

    function test_callableWhilePaused() public {
        IOrderBook.OrderData[] memory ordersData = _deferFills(1);

        vm.prank(pauser);
        orderBook.pause();

        // Flushing pending reports while paused is part of the upgrade runbook
        orderBook.sendFillReports(DEST_CHAIN_ID, ordersData, _recipients(1), bytes32(0), address(0), new bytes(0));
        assertEq(portal.getLastBatchLength(), 1, "pending reports should be flushable while paused");
    }

    function test_drainableAfterSupportDisabled() public {
        IOrderBook.OrderData[] memory ordersData = _deferFills(1);

        // Admin disables the origin chain; the existing pending record must stay drainable
        vm.prank(admin);
        orderBook.setBatchReportSupported(DEST_CHAIN_ID, false);

        orderBook.sendFillReports(DEST_CHAIN_ID, ordersData, _recipients(1), bytes32(0), address(0), new bytes(0));
        assertEq(portal.getLastBatchLength(), 1, "pending reports should remain drainable after disable");
    }

    function test_msgValueAndRefundAddressForwarded() public {
        IOrderBook.OrderData[] memory ordersData = _deferFills(1);
        bytes32 refundAddress = users["bob"].toBytes32();

        vm.deal(address(this), 1 ether);
        orderBook.sendFillReports{ value: 0.1 ether }(
            DEST_CHAIN_ID,
            ordersData,
            _recipients(1),
            refundAddress,
            address(0),
            new bytes(0)
        );

        assertEq(portal.lastBatchValue(), 0.1 ether, "msg.value should be forwarded to the portal");
        assertEq(portal.lastBatchRefundAddress(), refundAddress, "refund address should be forwarded");
    }
}
