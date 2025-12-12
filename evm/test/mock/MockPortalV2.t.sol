// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.8.26;

import { IPortalV2Like, IOrderBook } from "../../src/interfaces/IPortalV2Like.sol";

contract MockPortalV2 is IPortalV2Like {
    event FillReportSent(uint32 destinationChainId, IOrderBook.FillReport report);

    address public orderBook;

    mapping(bytes32 => IOrderBook.FillReport) public fillReports;

    function setOrderBook(address orderBook_) external {
        orderBook = orderBook_;
    }

    function sendFillReport(
        uint32 destinationChainId,
        IOrderBook.FillReport calldata report,
        bytes32 refundAddress,
        bytes calldata messageData
    ) external payable override returns (bytes32 messageId) {
        fillReports[report.orderId] = report;
        emit FillReportSent(destinationChainId, report);
    }

    function sendFillReport(
        uint32 destinationChainId,
        IOrderBook.FillReport calldata report,
        bytes32 refundAddress,
        address bridgeAdapter,
        bytes calldata bridgeAdapterArgs
    ) external payable override returns (bytes32 messageId) {
        fillReports[report.orderId] = report;
        emit FillReportSent(destinationChainId, report);
    }

    function receiveFillReport(IOrderBook.FillReport calldata report) external {
        IOrderBook(orderBook).reportFill(report);
    }

    function isFillReported(bytes32 orderId) external view returns (bool) {
        return fillReports[orderId].amountOutFilled != 0;
    }
}
