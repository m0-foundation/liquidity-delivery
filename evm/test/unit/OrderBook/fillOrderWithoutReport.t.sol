// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.33;

import { TypeConverter } from "../../../lib/common/src/libs/TypeConverter.sol";
import { PausableUpgradeable } from "../../../lib/common/lib/openzeppelin-contracts-upgradeable/contracts/utils/PausableUpgradeable.sol";

import { OrderBookTestBase } from "./OrderBookTestBase.t.sol";
import { IOrderBook } from "../../../src/interfaces/IOrderBook.sol";

contract FillOrderWithoutReportTest is OrderBookTestBase {
    using TypeConverter for *;

    // Test cases
    // [X] given the contract is paused
    //    [X] it reverts with an EnforcedPause error
    // [X] given the order originated on the current chain (i.e. it is local)
    //    [X] it reverts with a SameChainOrder error
    // [X] given the order's origin chain is not enabled for batch reporting
    //    [X] it reverts with a BatchReportsUnsupported error
    // [X] given a valid cross-chain order
    //    [X] it transfers tokenOut from the solver to the recipient
    //    [X] it records the fill amounts in filledAmounts (accounting parity with fillOrder)
    //    [X] it accrues the pending report for (orderId, originRecipient)
    //    [X] it does NOT send a message via the portal
    //    [X] it emits FillReportDeferred and OrderFilled with a zero messageId
    // [X] given repeated deferred fills for the same (order, recipient)
    //    [X] the pending report accumulates
    //    [X] a completing fill sets the order status to Completed
    // [X] given deferred fills with distinct origin recipients
    //    [X] each (order, recipient) pending record is tracked separately

    function setUp() public override {
        super.setUp();

        // Enable deferred reporting toward the foreign origin chain
        vm.prank(admin);
        orderBook.setBatchReportSupported(DEST_CHAIN_ID, true);

        // Approve the orderbook to spend tokenOut for the solver
        vm.startPrank(users["solver"]);
        tokens["token-out-6D"].approve(address(orderBook), type(uint256).max);
        tokens["token-out-18D"].approve(address(orderBook), type(uint256).max);
        vm.stopPrank();
    }

    function _foreignOrderData(uint64 nonce_) internal view returns (IOrderBook.OrderData memory) {
        return
            IOrderBook.OrderData({
                version: 1,
                originChainId: DEST_CHAIN_ID, // Order was created on chain 2
                sender: users["alice"].toBytes32(),
                nonce: nonce_,
                destChainId: CHAIN_ID, // To be filled on this chain
                createdAt: uint64(block.timestamp),
                fillDeadline: uint64(block.timestamp + FILL_DURATION),
                amountIn: AMOUNT_IN,
                amountOut: AMOUNT_OUT,
                tokenIn: address(tokenIn).toBytes32(),
                tokenOut: address(tokenOut).toBytes32(),
                recipient: users["alice"].toBytes32(),
                solver: bytes32(0) // No designated solver
            });
    }

    function _fillWithoutReport(
        address solver_,
        IOrderBook.OrderData memory orderData_,
        uint128 amountOutToFill_,
        bytes32 originRecipient_
    ) internal returns (bytes32 orderId_) {
        orderId_ = orderBook.getOrderId(orderData_);
        vm.prank(solver_);
        orderBook.fillOrderWithoutReport(
            orderId_,
            orderData_,
            IOrderBook.FillParams({
                amountOutToFill: amountOutToFill_,
                originRecipient: originRecipient_,
                refundAddress: bytes32(0)
            })
        );
    }

    function test_whenPaused_reverts() public {
        IOrderBook.OrderData memory orderData = _foreignOrderData(0);
        bytes32 orderId = orderBook.getOrderId(orderData);

        vm.prank(pauser);
        orderBook.pause();

        vm.prank(users["solver"]);
        vm.expectRevert(abi.encodeWithSelector(PausableUpgradeable.EnforcedPause.selector));
        orderBook.fillOrderWithoutReport(
            orderId,
            orderData,
            IOrderBook.FillParams({
                amountOutToFill: AMOUNT_OUT,
                originRecipient: users["solver"].toBytes32(),
                refundAddress: bytes32(0)
            })
        );
    }

    function test_sameChainOrder_reverts() public {
        // Create a local order (origin chain == destination chain == this chain)
        params.destChainId = CHAIN_ID;
        bytes32 orderId = _placeOrder(users["alice"], params);
        IOrderBook.Order memory order = orderBook.getOrder(orderId);

        vm.prank(params.solver.toAddress());
        vm.expectRevert(abi.encodeWithSelector(IOrderBook.SameChainOrder.selector));
        orderBook.fillOrderWithoutReport(
            orderId,
            _getOrderDataFromOrder(orderId, order),
            IOrderBook.FillParams({
                amountOutToFill: order.amountOut,
                originRecipient: params.solver,
                refundAddress: bytes32(0)
            })
        );
    }

    function test_batchReportsUnsupported_reverts() public {
        // Disable deferred reporting toward the foreign origin chain
        vm.prank(admin);
        orderBook.setBatchReportSupported(DEST_CHAIN_ID, false);

        IOrderBook.OrderData memory orderData = _foreignOrderData(0);
        bytes32 orderId = orderBook.getOrderId(orderData);

        vm.prank(users["solver"]);
        vm.expectRevert(abi.encodeWithSelector(IOrderBook.BatchReportsUnsupported.selector));
        orderBook.fillOrderWithoutReport(
            orderId,
            orderData,
            IOrderBook.FillParams({
                amountOutToFill: AMOUNT_OUT,
                originRecipient: users["solver"].toBytes32(),
                refundAddress: bytes32(0)
            })
        );
    }

    function test_deferredFill_success() public {
        IOrderBook.OrderData memory orderData = _foreignOrderData(0);
        bytes32 orderId = orderBook.getOrderId(orderData);
        address solver = users["solver"];

        uint256 solverTokenOutBefore = tokenOut.balanceOf(solver);
        uint256 recipientTokenOutBefore = tokenOut.balanceOf(users["alice"]);

        // Full deferred fill
        vm.prank(solver);
        vm.expectEmit(true, false, false, true);
        emit IOrderBook.OrderCompleted(orderId);
        vm.expectEmit(true, true, true, true);
        emit IOrderBook.FillReportDeferred(
            orderId,
            solver,
            solver.toBytes32(),
            orderData.amountIn,
            orderData.amountOut
        );
        vm.expectEmit(true, true, true, true);
        emit IOrderBook.OrderFilled(orderId, solver, orderData.amountIn, orderData.amountOut, bytes32(0));
        orderBook.fillOrderWithoutReport(
            orderId,
            orderData,
            IOrderBook.FillParams({
                amountOutToFill: orderData.amountOut,
                originRecipient: solver.toBytes32(),
                refundAddress: bytes32(0)
            })
        );

        // tokenOut moved from solver to recipient
        assertEq(tokenOut.balanceOf(solver), solverTokenOutBefore - orderData.amountOut, "solver sent tokenOut");
        assertEq(
            tokenOut.balanceOf(users["alice"]),
            recipientTokenOutBefore + orderData.amountOut,
            "recipient received tokenOut"
        );

        // Fill accounting recorded at fill time (identical to fillOrder)
        IOrderBook.FilledAmounts memory filled = orderBook.getFilledAmounts(orderId);
        assertEq(filled.amountOutFilled, orderData.amountOut, "amountOutFilled recorded");
        assertEq(filled.amountInReleased, orderData.amountIn, "amountInReleased recorded");

        // Order completed on this chain
        assertEq(
            uint8(orderBook.getOrder(orderId).status),
            uint8(IOrderBook.OrderStatus.Completed),
            "order should be completed"
        );

        // Pending report accrued; no portal message sent
        IOrderBook.PendingFillReport memory pending = orderBook.getPendingFillReport(orderId, solver.toBytes32());
        assertEq(pending.amountInToRelease, orderData.amountIn, "pending amountInToRelease");
        assertEq(pending.amountOutFilled, orderData.amountOut, "pending amountOutFilled");
        assertFalse(portal.isFillReported(orderId), "no fill report should have been sent");
    }

    function test_accountingParityWithFillOrder() public {
        // Two identical foreign orders except for the nonce; one filled with a report, one deferred
        IOrderBook.OrderData memory reportedData = _foreignOrderData(0);
        IOrderBook.OrderData memory deferredData = _foreignOrderData(1);
        bytes32 reportedId = orderBook.getOrderId(reportedData);
        bytes32 deferredId = orderBook.getOrderId(deferredData);
        address solver = users["solver"];
        uint128 fillAmount = AMOUNT_OUT / 3;

        vm.prank(solver);
        orderBook.fillOrder(
            reportedId,
            reportedData,
            IOrderBook.FillParams({
                amountOutToFill: fillAmount,
                originRecipient: solver.toBytes32(),
                refundAddress: bytes32(0)
            })
        );
        _fillWithoutReport(solver, deferredData, fillAmount, solver.toBytes32());

        // Identical fill accounting and order status
        IOrderBook.FilledAmounts memory reportedFilled = orderBook.getFilledAmounts(reportedId);
        IOrderBook.FilledAmounts memory deferredFilled = orderBook.getFilledAmounts(deferredId);
        assertEq(reportedFilled.amountOutFilled, deferredFilled.amountOutFilled, "amountOutFilled parity");
        assertEq(reportedFilled.amountInReleased, deferredFilled.amountInReleased, "amountInReleased parity");
        assertEq(
            uint8(orderBook.getOrder(reportedId).status),
            uint8(orderBook.getOrder(deferredId).status),
            "status parity"
        );

        // The deferred amounts equal what the immediate path reported to the portal
        IOrderBook.PendingFillReport memory pending = orderBook.getPendingFillReport(deferredId, solver.toBytes32());
        (, uint128 reportedInToRelease, uint128 reportedOutFilled, , ) = portal.fillReports(reportedId);
        assertEq(pending.amountInToRelease, reportedInToRelease, "pending matches reported amountInToRelease");
        assertEq(pending.amountOutFilled, reportedOutFilled, "pending matches reported amountOutFilled");
    }

    function test_repeatedDeferredFills_accumulate() public {
        IOrderBook.OrderData memory orderData = _foreignOrderData(0);
        address solver = users["solver"];
        uint128 firstFill = AMOUNT_OUT / 3;
        uint128 expectedFirstIn = uint128((uint256(AMOUNT_IN) * firstFill) / AMOUNT_OUT);

        bytes32 orderId = _fillWithoutReport(solver, orderData, firstFill, solver.toBytes32());

        IOrderBook.PendingFillReport memory pending = orderBook.getPendingFillReport(orderId, solver.toBytes32());
        assertEq(pending.amountInToRelease, expectedFirstIn, "first fill pending amountInToRelease");
        assertEq(pending.amountOutFilled, firstFill, "first fill pending amountOutFilled");

        // Second deferred fill completes the order and accumulates into the same record
        _fillWithoutReport(solver, orderData, AMOUNT_OUT - firstFill, solver.toBytes32());

        pending = orderBook.getPendingFillReport(orderId, solver.toBytes32());
        assertEq(pending.amountInToRelease, AMOUNT_IN, "accumulated pending amountInToRelease");
        assertEq(pending.amountOutFilled, AMOUNT_OUT, "accumulated pending amountOutFilled");
        assertEq(
            uint8(orderBook.getOrder(orderId).status),
            uint8(IOrderBook.OrderStatus.Completed),
            "order should be completed after the second fill"
        );
    }

    function test_distinctRecipients_trackedSeparately() public {
        IOrderBook.OrderData memory orderData = _foreignOrderData(0);
        address solver = users["solver"];
        bytes32 recipientA = users["bob"].toBytes32();
        bytes32 recipientB = users["carol"].toBytes32();
        uint128 fillA = AMOUNT_OUT / 3;
        uint128 fillB = AMOUNT_OUT / 4;
        uint128 expectedInA = uint128((uint256(AMOUNT_IN) * fillA) / AMOUNT_OUT);
        uint128 expectedInB = uint128((uint256(AMOUNT_IN) * fillB) / AMOUNT_OUT);

        bytes32 orderId = _fillWithoutReport(solver, orderData, fillA, recipientA);
        _fillWithoutReport(solver, orderData, fillB, recipientB);

        IOrderBook.PendingFillReport memory pendingA = orderBook.getPendingFillReport(orderId, recipientA);
        IOrderBook.PendingFillReport memory pendingB = orderBook.getPendingFillReport(orderId, recipientB);
        assertEq(pendingA.amountInToRelease, expectedInA, "recipient A pending amountInToRelease");
        assertEq(pendingA.amountOutFilled, fillA, "recipient A pending amountOutFilled");
        assertEq(pendingB.amountInToRelease, expectedInB, "recipient B pending amountInToRelease");
        assertEq(pendingB.amountOutFilled, fillB, "recipient B pending amountOutFilled");
    }
}
