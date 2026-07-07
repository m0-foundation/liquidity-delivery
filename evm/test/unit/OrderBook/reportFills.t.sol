// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.33;

import { TypeConverter } from "../../../lib/common/src/libs/TypeConverter.sol";

import { OrderBookTestBase } from "./OrderBookTestBase.t.sol";
import { IOrderBook } from "../../../src/interfaces/IOrderBook.sol";
import { MockERC20 } from "../../mock/MockERC20.t.sol";

/// @dev tokenIn that can block transfers to a recipient (USDC-style blacklist),
///      used to poison a single report inside a batch
contract MockBlacklistERC20 is MockERC20 {
    mapping(address => bool) public blacklisted;

    constructor() MockERC20("blacklist-token", "BLT", 6) {}

    function setBlacklisted(address account_, bool isBlacklisted_) external {
        blacklisted[account_] = isBlacklisted_;
    }

    function transfer(address to, uint256 amount) public override returns (bool) {
        require(!blacklisted[to], "blacklisted");
        return super.transfer(to, amount);
    }
}

contract ReportFillsTest is OrderBookTestBase {
    using TypeConverter for *;

    // Test cases
    // [X] given the caller is not the portal
    //    [X] it reverts with a NotAuthorized error
    // [X] given processFillReport is called by an account other than the OrderBook itself
    //    [X] it reverts with a NotAuthorized error
    // [X] given a batch of N valid reports
    //    [X] each report is processed exactly like a singular reportFill (differential)
    //    [X] tokenIn is released to each report's originRecipient
    //    [X] a completing report marks the order Completed
    // [X] given one report in the batch fails to process (blacklisted recipient)
    //    [X] the failed report is parked and FillReportFailed is emitted
    //    [X] the sibling reports still land
    //    [X] retryFillReport succeeds once the failure is resolved and emits FillReportRetried
    //    [X] retrying the same report again reverts with ReportNotFailed
    // [X] given retryFillReport for a report that never failed
    //    [X] it reverts with a ReportNotFailed error
    // [X] given a fill was deferred, the order cancelled, and the refund processed (§6.3)
    //    [X] the late batch report still pays the solver exactly and totals conserve

    function _fillReport(
        bytes32 orderId_,
        uint128 amountInToRelease_,
        uint128 amountOutFilled_,
        address originRecipient_,
        address tokenIn_
    ) internal pure returns (IOrderBook.FillReport memory) {
        return
            IOrderBook.FillReport({
                orderId: orderId_,
                amountInToRelease: amountInToRelease_,
                amountOutFilled: amountOutFilled_,
                originRecipient: originRecipient_.toBytes32(),
                tokenIn: tokenIn_.toBytes32()
            });
    }

    function test_notPortal_reverts() public {
        IOrderBook.FillReport[] memory reports = new IOrderBook.FillReport[](1);
        reports[0] = _fillReport(bytes32("order"), 1, 1, users["solver"], address(tokenIn));

        vm.prank(users["bob"]);
        vm.expectRevert(abi.encodeWithSelector(IOrderBook.NotAuthorized.selector));
        orderBook.reportFills(DEST_CHAIN_ID, reports);
    }

    function test_processFillReport_notSelf_reverts() public {
        IOrderBook.FillReport memory report = _fillReport(bytes32("order"), 1, 1, users["solver"], address(tokenIn));

        // Even the portal cannot call the self-call wrapper directly
        vm.prank(address(portal));
        vm.expectRevert(abi.encodeWithSelector(IOrderBook.NotAuthorized.selector));
        orderBook.processFillReport(DEST_CHAIN_ID, report);
    }

    function test_batchProcessesLikeSingularReports() public {
        // Two identical orders (differing only by nonce); one reported singularly, one in a batch
        bytes32 orderIdSingular = _placeOrder(users["alice"], params);
        bytes32 orderIdBatch = _placeOrder(users["alice"], params);
        uint128 fillAmount = params.amountOut / 3;
        uint128 amountIn = uint128((uint256(params.amountIn) * fillAmount) / params.amountOut);

        uint256 solverBefore = tokenIn.balanceOf(users["solver"]);
        _reportFill(users["solver"], orderIdSingular, fillAmount, amountIn);
        uint256 singularDelta = tokenIn.balanceOf(users["solver"]) - solverBefore;

        IOrderBook.FillReport[] memory reports = new IOrderBook.FillReport[](1);
        reports[0] = _fillReport(orderIdBatch, amountIn, fillAmount, users["solver"], address(tokenIn));
        solverBefore = tokenIn.balanceOf(users["solver"]);
        vm.prank(address(portal));
        vm.expectEmit(true, true, false, true);
        emit IOrderBook.FillReported(orderIdBatch, users["solver"], amountIn, fillAmount);
        orderBook.reportFills(DEST_CHAIN_ID, reports);

        assertEq(tokenIn.balanceOf(users["solver"]) - solverBefore, singularDelta, "identical payout");

        IOrderBook.FilledAmounts memory filledSingular = orderBook.getFilledAmounts(orderIdSingular);
        IOrderBook.FilledAmounts memory filledBatch = orderBook.getFilledAmounts(orderIdBatch);
        assertEq(filledBatch.amountOutFilled, filledSingular.amountOutFilled, "amountOutFilled parity");
        assertEq(filledBatch.amountInReleased, filledSingular.amountInReleased, "amountInReleased parity");
        assertEq(
            uint8(orderBook.getOrder(orderIdBatch).status),
            uint8(orderBook.getOrder(orderIdSingular).status),
            "status parity"
        );
    }

    function test_multiReportBatch_success() public {
        // Three orders, reported together; the last is a completing report
        bytes32[] memory orderIds = new bytes32[](3);
        for (uint256 i; i < 3; ++i) {
            orderIds[i] = _placeOrder(users["alice"], params);
        }

        IOrderBook.FillReport[] memory reports = new IOrderBook.FillReport[](3);
        uint128 partialOut = params.amountOut / 2;
        uint128 partialIn = uint128((uint256(params.amountIn) * partialOut) / params.amountOut);
        reports[0] = _fillReport(orderIds[0], partialIn, partialOut, users["solver"], address(tokenIn));
        reports[1] = _fillReport(orderIds[1], partialIn, partialOut, users["bob"], address(tokenIn));
        reports[2] = _fillReport(orderIds[2], params.amountIn, params.amountOut, users["solver"], address(tokenIn));

        uint256 solverBefore = tokenIn.balanceOf(users["solver"]);
        uint256 bobBefore = tokenIn.balanceOf(users["bob"]);

        vm.prank(address(portal));
        orderBook.reportFills(DEST_CHAIN_ID, reports);

        assertEq(
            tokenIn.balanceOf(users["solver"]),
            solverBefore + partialIn + params.amountIn,
            "solver receives both releases"
        );
        assertEq(tokenIn.balanceOf(users["bob"]), bobBefore + partialIn, "bob receives his release");
        assertEq(
            uint8(orderBook.getOrder(orderIds[0]).status),
            uint8(IOrderBook.OrderStatus.Created),
            "partially filled order stays Created"
        );
        assertEq(
            uint8(orderBook.getOrder(orderIds[2]).status),
            uint8(IOrderBook.OrderStatus.Completed),
            "fully filled order is Completed"
        );
    }

    function test_poisonedEntry_parksOnlyItself_andIsRetriable() public {
        // Order 1 uses a token that can blacklist the recipient; order 2 is a normal order
        MockBlacklistERC20 blToken = new MockBlacklistERC20();
        blToken.mint(users["alice"], MINT_AMOUNT * 1e6);

        IOrderBook.OrderParams memory blParams = params;
        blParams.tokenIn = address(blToken);
        vm.startPrank(users["alice"]);
        blToken.approve(address(orderBook), uint256(blParams.amountIn));
        bytes32 poisonedId = orderBook.openOrder(blParams);
        vm.stopPrank();

        bytes32 healthyId = _placeOrder(users["alice"], params);

        // The token blacklists the solver between fill and report
        blToken.setBlacklisted(users["solver"], true);

        IOrderBook.FillReport[] memory reports = new IOrderBook.FillReport[](2);
        reports[0] = _fillReport(poisonedId, params.amountIn, params.amountOut, users["solver"], address(blToken));
        reports[1] = _fillReport(healthyId, params.amountIn, params.amountOut, users["solver"], address(tokenIn));

        bytes32 reportHash = keccak256(abi.encode(DEST_CHAIN_ID, reports[0]));
        uint256 solverTokenInBefore = tokenIn.balanceOf(users["solver"]);

        // The poisoned report is parked; the sibling lands
        vm.prank(address(portal));
        vm.expectEmit(true, true, false, true);
        emit IOrderBook.FillReportFailed(DEST_CHAIN_ID, poisonedId, reportHash);
        orderBook.reportFills(DEST_CHAIN_ID, reports);

        assertEq(
            tokenIn.balanceOf(users["solver"]),
            solverTokenInBefore + params.amountIn,
            "sibling report should have paid out"
        );
        assertEq(
            uint8(orderBook.getOrder(healthyId).status),
            uint8(IOrderBook.OrderStatus.Completed),
            "sibling order should be completed"
        );
        assertEq(
            uint8(orderBook.getOrder(poisonedId).status),
            uint8(IOrderBook.OrderStatus.Created),
            "poisoned order should be untouched"
        );
        assertEq(blToken.balanceOf(users["solver"]), 0, "poisoned payout should not have happened");

        // Retry fails while the blacklist is still in place, and the report stays parked...
        vm.expectRevert("blacklisted");
        orderBook.retryFillReport(DEST_CHAIN_ID, reports[0]);

        // ...then succeeds (permissionlessly) once resolved
        blToken.setBlacklisted(users["solver"], false);
        vm.prank(users["bob"]);
        vm.expectEmit(true, false, false, true);
        emit IOrderBook.FillReportRetried(poisonedId, reportHash);
        orderBook.retryFillReport(DEST_CHAIN_ID, reports[0]);

        assertEq(blToken.balanceOf(users["solver"]), params.amountIn, "retried report should pay out");
        assertEq(
            uint8(orderBook.getOrder(poisonedId).status),
            uint8(IOrderBook.OrderStatus.Completed),
            "poisoned order should complete after retry"
        );

        // Retrying the same report again reverts (hash was cleared)
        vm.expectRevert(abi.encodeWithSelector(IOrderBook.ReportNotFailed.selector));
        orderBook.retryFillReport(DEST_CHAIN_ID, reports[0]);
    }

    function test_retryFillReport_neverFailed_reverts() public {
        bytes32 orderId = _placeOrder(users["alice"], params);
        IOrderBook.FillReport memory report = _fillReport(
            orderId,
            params.amountIn,
            params.amountOut,
            users["solver"],
            address(tokenIn)
        );

        vm.expectRevert(abi.encodeWithSelector(IOrderBook.ReportNotFailed.selector));
        orderBook.retryFillReport(DEST_CHAIN_ID, report);
    }

    function test_lateReportAfterCancelAndRefund_paysExactly() public {
        // §6.3 scenario: fill deferred on the destination chain, order cancelled there,
        // refund processed here, then the batch report arrives long after
        bytes32 orderId = _placeOrder(users["alice"], params);

        // The destination-side cancel refunds amountIn - amountInReleased, which already
        // counts the deferred (unreported) fill
        uint128 fillOut = 33e6;
        uint128 fillIn = uint128((uint256(params.amountIn) * fillOut) / params.amountOut);
        uint128 refund = params.amountIn - fillIn;

        uint256 aliceBefore = tokenIn.balanceOf(users["alice"]);
        _reportCancel(orderId, users["alice"], address(tokenIn), refund);
        assertEq(tokenIn.balanceOf(users["alice"]), aliceBefore + refund, "sender refunded the remainder");
        assertEq(uint8(orderBook.getOrder(orderId).status), uint8(IOrderBook.OrderStatus.Cancelled), "order cancelled");

        // The late batch report pays the solver exactly the deferred amount
        IOrderBook.FillReport[] memory reports = new IOrderBook.FillReport[](1);
        reports[0] = _fillReport(orderId, fillIn, fillOut, users["solver"], address(tokenIn));

        uint256 solverBefore = tokenIn.balanceOf(users["solver"]);
        vm.prank(address(portal));
        orderBook.reportFills(DEST_CHAIN_ID, reports);

        assertEq(tokenIn.balanceOf(users["solver"]), solverBefore + fillIn, "solver paid the deferred amount");

        // Conservation: released + refunded == amountIn, escrow fully drained
        IOrderBook.FilledAmounts memory filled = orderBook.getFilledAmounts(orderId);
        assertEq(filled.amountInReleased + filled.amountInRefunded, params.amountIn, "totals conserve");
        assertEq(tokenIn.balanceOf(address(orderBook)), 0, "no tokenIn stranded in the order book");
    }
}
