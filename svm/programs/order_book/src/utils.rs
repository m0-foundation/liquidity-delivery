// external dependencies
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};
use anchor_spl::token_interface::{close_account, CloseAccount};

use crate::state::ORDER_SEED_PREFIX;

pub fn transfer_tokens_from_program<'info>(
    from: &InterfaceAccount<'info, TokenAccount>,
    to: &InterfaceAccount<'info, TokenAccount>,
    amount: u64,
    mint: &InterfaceAccount<'info, Mint>,
    authority: &AccountInfo<'info>,
    authority_seeds: &[&[&[u8]]],
    token_program: &Interface<'info, TokenInterface>,
) -> Result<()> {
    // Build the arguments for the transfer instruction
    let transfer_options = TransferChecked {
        from: from.to_account_info(),
        to: to.to_account_info(),
        mint: mint.to_account_info(),
        authority: authority.clone(),
    };
    let cpi_context = CpiContext::new_with_signer(
        token_program.to_account_info(),
        transfer_options,
        authority_seeds,
    );

    // Call the transfer instruction
    transfer_checked(cpi_context, amount, mint.decimals)?;

    Ok(())
}

pub fn transfer_tokens<'info>(
    from: &InterfaceAccount<'info, TokenAccount>,
    to: &InterfaceAccount<'info, TokenAccount>,
    amount: u64,
    mint: &InterfaceAccount<'info, Mint>,
    authority: &AccountInfo<'info>,
    token_program: &Interface<'info, TokenInterface>,
) -> Result<()> {
    // Build the arguments for the transfer instruction
    let transfer_options = TransferChecked {
        from: from.to_account_info(),
        to: to.to_account_info(),
        mint: mint.to_account_info(),
        authority: authority.clone(),
    };
    let cpi_context = CpiContext::new(token_program.to_account_info(), transfer_options);

    // Call the transfer instruction
    transfer_checked(cpi_context, amount, mint.decimals)?;

    Ok(())
}

/// Close a program-owned token account (e.g. the order PDA's ATA) and send rent (lamports)
/// to `destination`.
///
/// - `token_program` can be SPL Token or Token-2022 (via token_interface).
/// - `authority` must be the PDA that owns the token account.
/// - `signer_seeds` must match the `authority` PDA seeds.
pub fn close_order_token_account<'info>(
    token_program: &Interface<'info, TokenInterface>,
    account_to_close: &InterfaceAccount<'info, TokenAccount>,
    destination: &AccountInfo<'info>,
    authority: &AccountInfo<'info>,
    order_id: [u8; 32],
    order_bump: u8,
) -> Result<()> {
    let order_signer: &[&[&[u8]]] = &[&[
            ORDER_SEED_PREFIX,
            order_id.as_ref(),
            &[order_bump]
        ]];

    close_account(CpiContext::new_with_signer(
        token_program.to_account_info(),
        CloseAccount {
            account: account_to_close.to_account_info(),
            destination: destination.clone(),
            authority: authority.clone(),
        },
        order_signer,
    ))
}
