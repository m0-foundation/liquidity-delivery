use super::super::{OrderBookTest, CHAIN_ID, DEST_CHAIN_ID};
use anchor_lang::prelude::Clock;
use anchor_litesvm::{Signer, TestHelpers};
use std::error::Error;

use order_book::error::OrderBookError;
use order_book::OrderData;

// Batch pending fill report tests
// report_pending_fills
// [X] given no remaining accounts are provided
//   [X] it reverts with an InvalidPendingFillAccounts error
// [X] given an odd number of remaining accounts
//   [X] it reverts with an InvalidPendingFillAccounts error
// [X] given a forged (non-PendingFillReport) account
//   [X] it reverts with an account deserialization error
// [X] given a payer account that does not match the pending record's payer
//   [X] it reverts with an InvalidPayer error
// [X] given a pending record whose origin chain differs from the batch origin chain
//   [X] it reverts with an InvalidOriginChainId error
// [X] given valid pending records
//   [X] it closes each pending PDA (the PDA's existence is the unreported flag)
//   [X] it returns the rent to the payer recorded at fill time
//   [X] it sends a single portal message (CPI succeeds against the mock portal)
//   [X] the caller can be anyone (permissionless flush)
// [X] given the program is paused
//   [X] report_pending_fills remains callable (upgrade-runbook flush)

fn default_foreign_order_data(test: &OrderBookTest, sender: &str, nonce: u64) -> OrderData {
    OrderData {
        version: order_book::VERSION,
        sender: test.get_user(sender).pubkey().to_bytes(),
        nonce,
        origin_chain_id: DEST_CHAIN_ID,
        dest_chain_id: CHAIN_ID,
        created_at: test.current_time(),
        fill_deadline: test.ctx.svm.get_sysvar::<Clock>().unix_timestamp as u64 + 86400,
        token_in: test.get_mint("token-in-spl-6").to_bytes(),
        token_out: test.get_mint("token-out-spl-6").to_bytes(),
        amount_in: 1_000_000,
        amount_out: 1_000_000,
        recipient: test.get_user(sender).pubkey().to_bytes(),
        solver: test.get_user("solver").pubkey().to_bytes(),
    }
}

#[test]
fn report_pending_fills_no_accounts_reverts() -> Result<(), Box<dyn Error>> {
    let mut test = OrderBookTest::new()?;
    test.initialize()?;

    let signer = test.get_user("solver");
    let ix = test.create_report_pending_fills_ix(&signer.pubkey(), DEST_CHAIN_ID, &[])?;

    test.ctx
        .execute_instruction(ix, &[&signer])?
        .assert_anchor_error(&format!("{:?}", OrderBookError::InvalidPendingFillAccounts));

    Ok(())
}

#[test]
fn report_pending_fills_odd_accounts_reverts() -> Result<(), Box<dyn Error>> {
    let mut test = OrderBookTest::new()?;
    test.initialize()?;

    let order_data = default_foreign_order_data(&test, "alice", 0);
    test.fill_foreign_order_no_report("solver", &order_data, 500_000)?;

    let solver = test.get_user("solver");
    let pending_pda = test.get_pending_fill_report_pda(
        &order_data.compute_order_id(),
        &solver.pubkey().to_bytes(),
    );

    // Build the instruction with a pending account but no payer account
    let mut ix = test.create_report_pending_fills_ix(&solver.pubkey(), DEST_CHAIN_ID, &[])?;
    ix.accounts
        .push(anchor_lang::solana_program::instruction::AccountMeta::new(
            pending_pda,
            false,
        ));

    test.ctx
        .execute_instruction(ix, &[&solver])?
        .assert_anchor_error(&format!("{:?}", OrderBookError::InvalidPendingFillAccounts));

    Ok(())
}

#[test]
fn report_pending_fills_forged_account_reverts() -> Result<(), Box<dyn Error>> {
    let mut test = OrderBookTest::new()?;
    test.initialize()?;

    let order_data = default_foreign_order_data(&test, "alice", 0);
    test.fill_foreign_order_no_report("solver", &order_data, 500_000)?;

    let solver = test.get_user("solver");
    // Pass the (foreign) order PDA where a PendingFillReport is expected
    let (forged_account, _) = test.get_foreign_order_account(&order_data.compute_order_id())?;

    let ix = test.create_report_pending_fills_ix(
        &solver.pubkey(),
        DEST_CHAIN_ID,
        &[(forged_account, solver.pubkey())],
    )?;

    test.ctx
        .execute_instruction(ix, &[&solver])?
        .assert_log_error("AccountDiscriminatorMismatch");

    Ok(())
}

#[test]
fn report_pending_fills_wrong_payer_reverts() -> Result<(), Box<dyn Error>> {
    let mut test = OrderBookTest::new()?;
    test.initialize()?;

    let order_data = default_foreign_order_data(&test, "alice", 0);
    test.fill_foreign_order_no_report("solver", &order_data, 500_000)?;

    let solver = test.get_user("solver");
    let bob = test.get_user("bob");
    let pending_pda = test.get_pending_fill_report_pda(
        &order_data.compute_order_id(),
        &solver.pubkey().to_bytes(),
    );

    // Bob tries to redirect the rent to himself
    let ix = test.create_report_pending_fills_ix(
        &bob.pubkey(),
        DEST_CHAIN_ID,
        &[(pending_pda, bob.pubkey())],
    )?;

    test.ctx
        .execute_instruction(ix, &[&bob])?
        .assert_anchor_error(&format!("{:?}", OrderBookError::InvalidPayer));

    Ok(())
}

#[test]
fn report_pending_fills_mixed_origin_chain_reverts() -> Result<(), Box<dyn Error>> {
    let mut test = OrderBookTest::new()?;
    test.initialize()?;

    let solver = test.get_user("solver");

    // Two pending records toward different origin chains
    let order_data_a = default_foreign_order_data(&test, "alice", 0);
    let mut order_data_b = default_foreign_order_data(&test, "alice", 1);
    order_data_b.origin_chain_id = 3;
    test.fill_foreign_order_no_report("solver", &order_data_a, 500_000)?;
    test.fill_foreign_order_no_report("solver", &order_data_b, 500_000)?;

    let pending_a = test.get_pending_fill_report_pda(
        &order_data_a.compute_order_id(),
        &solver.pubkey().to_bytes(),
    );
    let pending_b = test.get_pending_fill_report_pda(
        &order_data_b.compute_order_id(),
        &solver.pubkey().to_bytes(),
    );

    // One message goes to one origin chain; mixing must fail
    let ix = test.create_report_pending_fills_ix(
        &solver.pubkey(),
        DEST_CHAIN_ID,
        &[
            (pending_a, solver.pubkey()),
            (pending_b, solver.pubkey()),
        ],
    )?;

    test.ctx
        .execute_instruction(ix, &[&solver])?
        .assert_anchor_error(&format!("{:?}", OrderBookError::InvalidOriginChainId));

    Ok(())
}

#[test]
fn report_pending_fills_success_returns_rent_and_closes_pda() -> Result<(), Box<dyn Error>> {
    let mut test = OrderBookTest::new()?;
    test.initialize()?;

    let order_data = default_foreign_order_data(&test, "alice", 0);
    let order_id = order_data.compute_order_id();
    test.fill_foreign_order_no_report("solver", &order_data, 500_000)?;

    let solver = test.get_user("solver");
    let (pending_pda, _) =
        test.get_pending_fill_report_account(&order_id, &solver.pubkey().to_bytes())?;
    let pending_rent = test.ctx.svm.get_balance(&pending_pda).unwrap_or(0);
    assert!(pending_rent > 0, "pending PDA should hold rent");
    let solver_lamports_before = test.ctx.svm.get_balance(&solver.pubkey()).unwrap_or(0);

    // Bob flushes the solver's pending report (permissionless); rent still goes to the solver
    test.report_pending_fills("bob", DEST_CHAIN_ID, &[(pending_pda, solver.pubkey())])?;

    // Pending PDA is closed: its existence was the unreported flag
    assert!(
        test.get_pending_fill_report_account(&order_id, &solver.pubkey().to_bytes())
            .is_err(),
        "pending PDA should be closed after reporting"
    );

    // Rent returned to the payer recorded at fill time
    assert_eq!(
        test.ctx.svm.get_balance(&solver.pubkey()).unwrap_or(0),
        solver_lamports_before + pending_rent,
        "rent should be returned to the fill-time payer"
    );

    Ok(())
}

#[test]
fn report_pending_fills_multiple_orders_one_message() -> Result<(), Box<dyn Error>> {
    let mut test = OrderBookTest::new()?;
    test.initialize()?;

    let solver = test.get_user("solver");
    let mut pairs: Vec<(anchor_litesvm::Pubkey, anchor_litesvm::Pubkey)> = vec![];

    for nonce in 0..3u64 {
        let order_data = default_foreign_order_data(&test, "alice", nonce);
        test.fill_foreign_order_no_report("solver", &order_data, 500_000)?;
        pairs.push((
            test.get_pending_fill_report_pda(
                &order_data.compute_order_id(),
                &solver.pubkey().to_bytes(),
            ),
            solver.pubkey(),
        ));
    }

    // All three pending records drain in a single instruction (one portal message)
    test.report_pending_fills("solver", DEST_CHAIN_ID, &pairs)?;

    for (pending_pda, _) in pairs {
        assert_eq!(
            test.ctx.svm.get_balance(&pending_pda).unwrap_or(0),
            0,
            "all pending PDAs should be closed"
        );
    }

    Ok(())
}

#[test]
fn report_pending_fills_callable_while_paused() -> Result<(), Box<dyn Error>> {
    let mut test = OrderBookTest::new()?;
    test.initialize()?;

    let order_data = default_foreign_order_data(&test, "alice", 0);
    test.fill_foreign_order_no_report("solver", &order_data, 500_000)?;

    let solver = test.get_user("solver");
    let pending_pda = test.get_pending_fill_report_pda(
        &order_data.compute_order_id(),
        &solver.pubkey().to_bytes(),
    );

    // Flushing pending reports while paused is part of the upgrade runbook
    test.pause()?;
    test.report_pending_fills("solver", DEST_CHAIN_ID, &[(pending_pda, solver.pubkey())])?;

    Ok(())
}

#[test]
fn report_pending_fills_recreate_after_drain() -> Result<(), Box<dyn Error>> {
    let mut test = OrderBookTest::new()?;
    test.initialize()?;

    let order_data = default_foreign_order_data(&test, "alice", 0);
    let order_id = order_data.compute_order_id();
    let solver = test.get_user("solver");

    // Defer, drain, then defer again: the PDA is re-created cleanly
    test.fill_foreign_order_no_report("solver", &order_data, 300_000)?;
    let pending_pda =
        test.get_pending_fill_report_pda(&order_id, &solver.pubkey().to_bytes());
    test.report_pending_fills("solver", DEST_CHAIN_ID, &[(pending_pda, solver.pubkey())])?;

    test.fill_foreign_order_no_report("solver", &order_data, 200_000)?;

    let (_, pending) =
        test.get_pending_fill_report_account(&order_id, &solver.pubkey().to_bytes())?;
    assert_eq!(
        pending.amount_out_filled, 200_000u128,
        "re-created pending record should only hold the new fill"
    );

    Ok(())
}
