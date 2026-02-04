use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteRequest {
    pub input_token: String,
    pub input_chain_id: u32,
    pub output_token: String,
    pub output_chain_id: u32,
    pub amount_in: u64,
    /// Sender address for transaction building (optional)
    #[serde(default)]
    pub sender_address: Option<String>,
    /// Recipient address on destination chain (defaults to sender if not provided)
    #[serde(default)]
    pub recipient: Option<String>,
}

/// EVM transaction parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvmTransaction {
    /// Target contract address (hex with 0x prefix)
    pub to: String,
    /// Transaction calldata (hex with 0x prefix)
    pub data: String,
    /// Transaction value in wei (hex with 0x prefix)
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteResponse {
    pub quote_id: String,
    pub fee_bps: u32,
    pub output_amount: u64,
    pub est_fill_time_seconds: u64,
    pub expires_at: String,
    pub rejected: bool,
    pub reject_reason: String,
    pub solver_address: String,
    pub requires_exclusivity: bool,
    /// Computed order ID (hex string) for redirect after order creation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_id: Option<String>,
    /// EVM transaction to open the order
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evm_transaction: Option<EvmTransaction>,
    /// EVM approval transaction (if token allowance is insufficient)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval_transaction: Option<EvmTransaction>,
    /// Serialized SVM transaction (base64)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub svm_transaction: Option<String>,
    /// Nonce used for the order
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<u64>,
    /// OrderBook contract address for the input chain
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orderbook_address: Option<String>,
}

/// Request to build a cancel order transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelRequest {
    /// Order ID to cancel (hex string with 0x prefix)
    pub order_id: String,
    /// Origin chain where the order was created
    pub origin_chain_id: u32,
    /// Destination chain where the order will be filled/cancelled
    pub dest_chain_id: u32,
    /// Order version
    pub version: u16,
    /// Order nonce
    pub nonce: u64,
    /// Timestamp when order was created
    pub created_at: u64,
    /// Fill deadline timestamp
    pub fill_deadline: u64,
    /// Sender address on origin chain
    pub sender: String,
    /// Recipient address on destination chain
    pub recipient: String,
    /// Input token address on origin chain
    pub token_in: String,
    /// Output token address on destination chain
    pub token_out: String,
    /// Amount of input token
    pub amount_in: u64,
    /// Amount of output token
    pub amount_out: u64,
    /// Solver address (or zero address for any solver)
    pub solver: String,
    /// Address of the caller (for building the transaction)
    pub caller_address: String,
}

/// Response with the cancel transaction to sign
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelResponse {
    /// Order ID being cancelled
    pub order_id: String,
    /// EVM transaction to cancel the order (if dest chain is EVM)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evm_transaction: Option<EvmTransaction>,
    /// Serialized SVM transaction (base64) to cancel the order (if dest chain is SVM)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub svm_transaction: Option<String>,
    /// OrderBook contract/program address on dest chain
    pub orderbook_address: String,
    /// Chain ID where the transaction should be submitted
    pub chain_id: u32,
}
