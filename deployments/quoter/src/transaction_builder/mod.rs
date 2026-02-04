pub mod error;
pub mod evm;
pub mod order_id;
pub mod svm;

pub use error::TransactionBuilderError;
pub use evm::EvmTransactionBuilder;
pub use svm::SvmTransactionBuilder;

use crate::models::EvmTransaction;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvmTransactionResult {
    pub transaction: EvmTransaction,
    pub approval_transaction: Option<EvmTransaction>,
    /// Order ID is not predictable for EVM (uses block.timestamp on-chain)
    /// Frontend must extract actual order ID from transaction logs
    pub order_id: Option<String>,
    pub nonce: u64,
    pub contract_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResult {
    pub transaction: String,
    pub order_id: String,
    pub nonce: u64,
    pub contract_address: String,
}

#[derive(Debug, Clone)]
pub struct OpenOrderInput {
    pub sender_address: String,
    pub token_in: String,
    pub token_out: String,
    pub amount_in: u64,
    pub amount_out: u128,
    pub recipient: [u8; 32],
    pub solver: [u8; 32],
    pub dest_chain_id: u32,
    pub fill_deadline: u64,
}

/// Input data for building a cancel order transaction
#[derive(Debug, Clone)]
pub struct CancelOrderInput {
    pub order_id: [u8; 32],
    pub version: u16,
    pub sender: [u8; 32],
    pub nonce: u64,
    pub origin_chain_id: u32,
    pub dest_chain_id: u32,
    pub created_at: u64,
    pub fill_deadline: u64,
    pub token_in: [u8; 32],
    pub token_out: [u8; 32],
    pub amount_in: u128,
    pub amount_out: u128,
    pub recipient: [u8; 32],
    pub solver: [u8; 32],
    /// Caller address (for SVM transaction building)
    pub caller_address: String,
}
