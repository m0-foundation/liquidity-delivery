use alloy::sol_types::sol;

// Define Solidity events
sol! {
    #![sol(rpc, alloy_sol_types = alloy::sol_types)]

    #[derive(Debug)]
    event OrderOpened(
        bytes32 orderId,
        address indexed sender,
        address tokenIn,
        uint128 amountIn,
        uint32 indexed destChainId,
        bytes32 tokenOut,
        uint128 amountOut,
        bytes32 indexed solver
    );

    #[derive(Debug)]
    event OrderFilled(
        bytes32 indexed orderId,
        address indexed solver,
        uint128 amountInToRelease,
        uint128 amountOutFilled,
        bytes32 indexed messageId
    );

    #[derive(Debug)]
    event OrderCancelled(
        bytes32 indexed orderId,
        bytes32 indexed messageId
    );

    #[derive(Debug)]
    event RefundClaimed(
        bytes32 indexed orderId,
        address indexed sender,
        uint128 amountInRefunded
    );

    #[derive(Debug)]
    event OrderCompleted(
        bytes32 orderId
    );

    #[derive(Debug)]
    event FillReported(
        bytes32 indexed orderId,
        address indexed originRecipient,
        uint128 amountInToRelease,
        uint128 amountOutFilled
    );

    #[derive(Debug)]
    event CancelReported(
        bytes32 indexed orderId
    );
}
