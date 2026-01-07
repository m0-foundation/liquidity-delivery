use crate::{
    error::OrderBookError,
    state::{NativeOrder, Order, OrderStatus, ORDER_SEED_PREFIX},
};
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    close_account, CloseAccount, Mint, TokenAccount, TokenInterface,
};

/// Close the order's token account and return SOL rent to the original payer
///
/// This instruction:
/// - Closes the order_token_in_ata (which should have 0 token balance)
/// - Returns the ~0.002 SOL rent to the payer who originally created the order
/// - Can be called by anyone (permissionless) after the order is finalized
///
/// Note: This only reclaims SOL rent. SPL tokens are already returned via
/// the cancel/fill instructions.
#[derive(Accounts)]
#[instruction(order_id: [u8; 32])]
pub struct CloseOrderTokenAccount<'info> {
    /// The original payer who paid SOL rent for the ATA
    /// SOL rent will be refunded to this account
    /// CHECK: Validated against order.data.payer in the validation function
    #[account(mut)]
    pub payer: UncheckedAccount<'info>,

    /// The order account - must be in Completed or Cancelled status
    #[account(
        seeds = [ORDER_SEED_PREFIX, &order_id],
        bump = order.bump,
        constraint = order.data.payer == payer.key() @ OrderBookError::InvalidPayer,
    )]
    pub order: Account<'info, Order::<NativeOrder>>,

    /// The token mint for validation
    #[account(
        address = order.data.token_in @ OrderBookError::InvalidTokenMint,
        mint::token_program = token_in_program
    )]
    pub token_in_mint: InterfaceAccount<'info, Mint>,

    /// The order's ATA to close - must have 0 token balance
    #[account(
        mut,
        associated_token::mint = token_in_mint,
        associated_token::authority = order,
        associated_token::token_program = token_in_program,
    )]
    pub order_token_in_ata: InterfaceAccount<'info, TokenAccount>,

    /// SPL Token or Token-2022 program
    pub token_in_program: Interface<'info, TokenInterface>,
}

impl CloseOrderTokenAccount<'_> {
    fn validate(&self) -> Result<()> {
        let order = &self.order.data;

        // Verify order is in a finalized state (Completed or Cancelled)
        require!(
            order.status == OrderStatus::Completed || order.status == OrderStatus::Cancelled,
            OrderBookError::OrderNotFinalized
        );

        // Verify token account is empty
        require_eq!(
            self.order_token_in_ata.amount,
            0,
            OrderBookError::TokenAccountNotEmpty
        );

        Ok(())
    }

    #[access_control(ctx.accounts.validate())]
    pub fn handler(ctx: Context<Self>, order_id: [u8; 32]) -> Result<()> {
        // Build PDA signer seeds
        let order_seeds: &[&[&[u8]]] = &[&[
            ORDER_SEED_PREFIX,
            order_id.as_ref(),
            &[ctx.accounts.order.bump],
        ]];

        // Close the ATA and return SOL rent to payer
        close_account(CpiContext::new_with_signer(
            ctx.accounts.token_in_program.to_account_info(),
            CloseAccount {
                account: ctx.accounts.order_token_in_ata.to_account_info(),
                destination: ctx.accounts.payer.to_account_info(),
                authority: ctx.accounts.order.to_account_info(),
            },
            order_seeds,
        ))?;

        Ok(())
    }
}
