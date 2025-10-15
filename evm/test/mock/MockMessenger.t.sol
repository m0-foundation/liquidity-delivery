// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.8.26;

import { IMessenger, IOrderBook } from "../../src/interfaces/IMessenger.sol";

contract MockMessenger is IMessenger {
    event FillReportSent(uint32 destinationChainId, IOrderBook.FillReport report);

    address public orderBook;

    function setOrderBook(address orderBook_) external {
        orderBook = orderBook_;
    }
    
    function sendFillReport(
        uint32 destinationChainId,
        IOrderBook.FillReport calldata report
    ) external override {
        emit FillReportSent(destinationChainId, report);
    }

    function receiveFillReport(
        IOrderBook.FillReport calldata report
    ) external {
        IOrderBook(orderBook).reportFill(report);
    }
}