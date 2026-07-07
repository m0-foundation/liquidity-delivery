use anchor_lang::prelude::*;

#[constant]
pub const PENDING_FILL_SEED_PREFIX: &[u8] = b"pending_fill";

// Fill amounts recorded on this (destination) chain awaiting a batched report.
// The existence of this PDA is the "unreported" flag: report_pending_fills closes
// the account when the report is sent, returning rent to the payer.
// Repeated deferred fills of the same (order, origin_recipient) accumulate here.
#[account]
#[derive(InitSpace)]
pub struct PendingFillReport {
    pub bump: u8,
    // fill-time signer; memcmp filter for solver-side discovery
    pub solver: Pubkey,
    // rent returns here when the record is closed
    pub payer: Pubkey,
    pub order_id: [u8; 32],
    pub origin_chain_id: u32,
    pub token_in: [u8; 32],
    pub origin_recipient: [u8; 32],
    pub amount_in_to_release: u128,
    pub amount_out_filled: u128,
}
