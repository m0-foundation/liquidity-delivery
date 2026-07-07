use crate::{
    error::OrderBookError,
    state::{OrderBookGlobal, PendingFillReport, GLOBAL_SEED, PENDING_FILL_SEED_PREFIX},
};
use anchor_lang::prelude::*;

use crate::portal::{
    constants::{AUTHORITY_SEED as PORTAL_AUTHORITY_SEED, GLOBAL_SEED as PORTAL_GLOBAL_SEED},
    cpi::{accounts::SendFillReports, send_fill_reports},
    program::Portal,
    types::FillReportPayload,
    ID as PORTAL_ID,
};

// Sends a single Portal message reporting all pending (deferred) fills provided in
// remaining_accounts, amortizing the bridge fee across fills.
// Permissionless: draining a pending record can only pay out to the origin_recipient
// fixed at fill time, so anyone can flush a solver's pending reports.

// Events
#[event]
pub struct PendingFillReported {
    pub order_id: [u8; 32],
    pub origin_recipient: [u8; 32],
    pub amount_in_to_release: u128,
    pub amount_out_filled: u128,
}

#[event_cpi]
#[derive(Accounts)]
#[instruction(origin_chain_id: u32)]
pub struct ReportPendingFills<'info> {
    // Pays the bridge fee; permissionless
    #[account(mut)]
    pub signer: Signer<'info>,

    // Deliberately no paused check: flushing pending reports while paused
    // is part of the upgrade runbook
    #[account(
        mut,
        seeds = [GLOBAL_SEED],
        bump = global_account.bump,
    )]
    pub global_account: Account<'info, OrderBookGlobal>,

    pub portal_program: Program<'info, Portal>,

    /// CHECK: We validate the account seeds here
    /// The data is not used in this instruction
    /// We pass it into the CPI to the portal program
    #[account(
        mut,
        seeds = [PORTAL_GLOBAL_SEED],
        seeds::program = PORTAL_ID,
        bump,
    )]
    pub portal_global: UncheckedAccount<'info>,

    /// CHECK: We validate the seeds here
    /// The account holds no data and is used as a signer
    /// in the CPI to the portal program
    #[account(
        seeds = [PORTAL_AUTHORITY_SEED],
        seeds::program = PORTAL_ID,
        bump
    )]
    pub portal_authority: UncheckedAccount<'info>,

    /// CHECK: Bridge adapter program
    /// This is validated in the portal CPI
    pub bridge_adapter: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
    // remaining_accounts: [pending_pda_0, payer_0, pending_pda_1, payer_1, ...]
    // both accounts of each pair must be writable (the pending account is closed
    // and its rent lamports are returned to the payer)
}

impl<'info> ReportPendingFills<'info> {
    pub fn handler(
        ctx: Context<'_, '_, 'info, 'info, Self>,
        origin_chain_id: u32,
    ) -> Result<()> {
        let remaining = ctx.remaining_accounts;
        require!(
            !remaining.is_empty() && remaining.len() % 2 == 0,
            OrderBookError::InvalidPendingFillAccounts
        );

        let mut reports: Vec<FillReportPayload> = Vec::with_capacity(remaining.len() / 2);
        let mut to_close: Vec<(Account<'info, PendingFillReport>, &AccountInfo<'info>)> =
            Vec::with_capacity(remaining.len() / 2);

        for pair in remaining.chunks(2) {
            let pending_info = &pair[0];
            let payer_info = &pair[1];

            // Checks owner and discriminator (rejects forged and duplicate accounts)
            let pending = Account::<PendingFillReport>::try_from(pending_info)?;

            // Re-derive the PDA from the stored identity fields to defend against forged accounts
            let expected_key = Pubkey::create_program_address(
                &[
                    PENDING_FILL_SEED_PREFIX,
                    &pending.order_id,
                    &pending.origin_recipient,
                    &[pending.bump],
                ],
                &crate::ID,
            )
            .map_err(|_| OrderBookError::InvalidPendingFillAccounts)?;
            require_keys_eq!(
                pending_info.key(),
                expected_key,
                OrderBookError::InvalidPendingFillAccounts
            );

            // Duplicate pending entries in one call would double-report; the second
            // occurrence is caught here because its key equals an already-collected one
            require!(
                !to_close.iter().any(|(p, _)| p.key() == pending_info.key()),
                OrderBookError::InvalidPendingFillAccounts
            );

            // One message goes to one origin chain
            require!(
                pending.origin_chain_id == origin_chain_id,
                OrderBookError::InvalidOriginChainId
            );

            // Rent is returned to whoever funded the pending record
            require_keys_eq!(
                payer_info.key(),
                pending.payer,
                OrderBookError::InvalidPayer
            );

            reports.push(FillReportPayload {
                order_id: pending.order_id,
                token_in: pending.token_in,
                amount_in_to_release: pending.amount_in_to_release,
                amount_out_filled: pending.amount_out_filled,
                origin_recipient: pending.origin_recipient,
            });

            to_close.push((pending, payer_info));
        }

        // Send all reports to the origin chain in a single portal message
        send_fill_reports(
            CpiContext::new_with_signer(
                ctx.accounts.portal_program.to_account_info(),
                SendFillReports {
                    sender: ctx.accounts.signer.to_account_info(),
                    order_book_global: ctx.accounts.global_account.to_account_info(),
                    portal_global: ctx.accounts.portal_global.to_account_info(),
                    portal_authority: ctx.accounts.portal_authority.to_account_info(),
                    bridge_adapter: ctx.accounts.bridge_adapter.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                },
                &[&[GLOBAL_SEED, &[ctx.accounts.global_account.bump]]],
            ),
            reports.clone(),
            origin_chain_id,
        )?;

        // Close the pending accounts, returning rent to the fill-time payer.
        // Closing the account is the atomic "mark as reported": the PDA's existence
        // is the unreported flag. This must happen AFTER the portal CPI: direct
        // lamport moves involving an account that is later passed into a CPI (e.g.
        // a solver flushing their own reports is both the CPI sender and the rent
        // recipient) fail the runtime's CPI-boundary lamport balance check.
        for (pending, payer_info) in to_close {
            pending.close(payer_info.to_account_info())?;
        }

        for report in reports {
            emit_cpi!(PendingFillReported {
                order_id: report.order_id,
                origin_recipient: report.origin_recipient,
                amount_in_to_release: report.amount_in_to_release,
                amount_out_filled: report.amount_out_filled,
            });
        }

        Ok(())
    }
}
