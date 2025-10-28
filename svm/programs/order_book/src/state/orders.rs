use anchor_lang::{prelude::*,solana_program::keccak};

#[repr(u8)]
#[derive(AnchorDeserialize, AnchorSerialize, InitSpace, Clone, PartialEq)]
pub enum OrderStatus {
    DoesNotExist,
    Created,
    CancelRequested,
    Completed
}

#[repr(u8)]
#[derive(AnchorDeserialize, AnchorSerialize, InitSpace, Clone, PartialEq)]
pub enum OrderType {
    Native,
    Foreign
}

#[constant]
pub const ORDER_SEED_PREFIX: &[u8] = b"order";

#[account]
#[derive(InitSpace)]
pub struct Order<T: AnchorDeserialize + AnchorSerialize + Space> {
    pub order_type: OrderType,
    pub bump: u8,
    pub data: T,
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone, InitSpace)]
pub struct NativeOrder {
    pub status: OrderStatus,
    pub version: u16,
    pub sender: Pubkey,
    pub nonce: u64,
    pub dest_chain_id: u32,
    pub fill_deadline: u32,
    pub cancel_requested_at: u32,
    pub token_in: Pubkey,
    pub token_out: [u8; 32], 
    pub amount_in: u128,
    pub amount_out: u128,
    pub recipient: [u8; 32], 
    pub solver: [u8; 32], 
    pub amount_in_released: u128,
    pub amount_out_filled: u128,
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone, InitSpace)]
pub struct ForeignOrder {
    pub amount_in_released: u128,
    pub amount_out_filled: u128,
}

// Note: this must match the EVM version exactly
// We derive the Order ID from the hash of this struct
#[derive(AnchorDeserialize, AnchorSerialize, Clone, Debug)]
pub struct OrderData {
    pub version: u16,
    pub sender: [u8; 32],
    pub nonce: u64,
    pub origin_chain_id: u32,
    pub dest_chain_id: u32,
    pub fill_deadline: u32,
    pub token_out: [u8; 32],
    pub amount_in: u128,
    pub amount_out: u128,
    pub recipient: [u8; 32],
    pub solver: [u8; 32],
}

pub fn compute_order_id(order_data: &OrderData) -> [u8; 32] {
    keccak::hashv(&[
        &order_data.version.to_le_bytes(),
        &order_data.sender,
        &order_data.nonce.to_le_bytes(),
        &order_data.origin_chain_id.to_le_bytes(),
        &order_data.dest_chain_id.to_le_bytes(),
        &order_data.fill_deadline.to_le_bytes(),
        &order_data.token_out,
        &order_data.amount_in.to_le_bytes(),
        &order_data.amount_out.to_le_bytes(),
        &order_data.recipient,
        &order_data.solver,
    ]).to_bytes()
}

impl OrderData {
    // TODO confirm hashes match the EVM implementation
    // I'm not positive whether the values should be converted to big-endian or little-endian byte order
    pub fn compute_order_id(&self) -> [u8; 32] {
        compute_order_id(&self)
    }
}