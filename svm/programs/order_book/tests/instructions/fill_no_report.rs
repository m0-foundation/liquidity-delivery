use super::super::{OrderBookTest, CHAIN_ID, DEST_CHAIN_ID};
use anchor_lang::prelude::Clock;
use anchor_litesvm::{Signer, TestHelpers};
use anchor_spl::associated_token::get_associated_token_address;
use std::error::Error;

use order_book::error::OrderBookError;
use order_book::{OrderData, OrderStatus};

// Deferred fill tests
// fill_foreign_order_no_report
// [X] given the program is paused
//   [X] it reverts with a ProgramPaused error
// [X] given the origin chain of the order is the current chain (same-chain order)
//   [X] it reverts with an InvalidOriginChainId error
// [X] given a valid cross-chain order
//   [X] it transfers token_out from the solver to the recipient
//   [X] it records the fill amounts on the order (accounting parity with fill_foreign_order)
//   [X] it creates the pending fill report PDA with the fill amounts and identity fields
//   [X] it does NOT interact with the portal (no portal accounts in the instruction)
// [X] given repeated deferred fills for the same (order, origin_recipient)
//   [X] the pending amounts accumulate in the same PDA
//   [X] a completing fill sets the order status to Completed
// [X] given deferred fills with distinct origin recipients
//   [X] each (order, recipient) pending PDA is tracked separately

fn default_foreign_order_data(test: &OrderBookTest, sender: &str) -> OrderData {
    OrderData {
        version: order_book::VERSION,
        sender: test.get_user(sender).pubkey().to_bytes(),
        nonce: 0,
        origin_chain_id: DEST_CHAIN_ID, // Foreign order originates on another chain
        dest_chain_id: CHAIN_ID,        // Settles on current chain
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
fn fill_no_report_paused_reverts() -> Result<(), Box<dyn Error>> {
    let mut test = OrderBookTest::new()?;
    test.initialize()?;
    test.pause()?;

    let order_data = default_foreign_order_data(&test, "alice");
    let solver = test.get_user("solver");
    let fill_params = order_book::instructions::FillParams {
        amount_out_to_fill: 500_000,
        origin_recipient: solver.pubkey().to_bytes(),
    };

    let ix =
        test.create_fill_foreign_order_no_report_ix(&solver.pubkey(), &order_data, &fill_params)?;

    test.ctx
        .execute_instruction(ix, &[&solver])?
        .assert_anchor_error(&format!("{:?}", OrderBookError::ProgramPaused));

    Ok(())
}

#[test]
fn fill_no_report_same_chain_order_reverts() -> Result<(), Box<dyn Error>> {
    let mut test = OrderBookTest::new()?;
    test.initialize()?;

    let mut order_data = default_foreign_order_data(&test, "alice");
    order_data.origin_chain_id = CHAIN_ID; // same-chain order cannot defer its report
    let solver = test.get_user("solver");
    let fill_params = order_book::instructions::FillParams {
        amount_out_to_fill: 500_000,
        origin_recipient: solver.pubkey().to_bytes(),
    };

    let ix =
        test.create_fill_foreign_order_no_report_ix(&solver.pubkey(), &order_data, &fill_params)?;

    test.ctx
        .execute_instruction(ix, &[&solver])?
        .assert_anchor_error(&format!("{:?}", OrderBookError::InvalidOriginChainId));

    Ok(())
}

#[test]
fn fill_no_report_success() -> Result<(), Box<dyn Error>> {
    let mut test = OrderBookTest::new()?;
    test.initialize()?;

    let order_data = default_foreign_order_data(&test, "alice");
    let order_id = order_data.compute_order_id();
    let solver = test.get_user("solver");
    let amount_out_to_fill = 500_000u64;
    let expected_amount_in = 500_000u128; // pro-rata: 1:1 order

    let recipient_ata = test.get_ata("token-out-spl-6", "alice");
    let solver_ata = test.get_ata("token-out-spl-6", "solver");
    let recipient_balance_before = test.get_token_balance(&recipient_ata)?;
    let solver_balance_before = test.get_token_balance(&solver_ata)?;

    test.fill_foreign_order_no_report("solver", &order_data, amount_out_to_fill)?;

    // token_out moved from the solver to the recipient
    assert_eq!(
        test.get_token_balance(&recipient_ata)?,
        recipient_balance_before + amount_out_to_fill,
        "recipient should have received token_out"
    );
    assert_eq!(
        test.get_token_balance(&solver_ata)?,
        solver_balance_before - amount_out_to_fill,
        "solver should have sent token_out"
    );

    // Fill accounting recorded at fill time (identical to fill_foreign_order)
    let (_, order) = test.get_foreign_order_account(&order_id)?;
    assert_eq!(order.data.status, OrderStatus::Created);
    assert_eq!(order.data.amount_out_filled, amount_out_to_fill as u128);
    assert_eq!(order.data.amount_in_released, expected_amount_in);

    // Pending fill report created with the amounts and identity fields
    let (_, pending) =
        test.get_pending_fill_report_account(&order_id, &solver.pubkey().to_bytes())?;
    assert_eq!(pending.order_id, order_id);
    assert_eq!(pending.origin_chain_id, DEST_CHAIN_ID);
    assert_eq!(pending.token_in, order_data.token_in);
    assert_eq!(pending.origin_recipient, solver.pubkey().to_bytes());
    assert_eq!(pending.solver, solver.pubkey());
    assert_eq!(pending.payer, solver.pubkey());
    assert_eq!(pending.amount_out_filled, amount_out_to_fill as u128);
    assert_eq!(pending.amount_in_to_release, expected_amount_in);

    Ok(())
}

#[test]
fn fill_no_report_accumulates_and_completes() -> Result<(), Box<dyn Error>> {
    let mut test = OrderBookTest::new()?;
    test.initialize()?;

    let order_data = default_foreign_order_data(&test, "alice");
    let order_id = order_data.compute_order_id();
    let solver = test.get_user("solver");

    test.fill_foreign_order_no_report("solver", &order_data, 400_000)?;
    // Second deferred fill completes the order and accumulates in the same PDA
    test.fill_foreign_order_no_report("solver", &order_data, 600_000)?;

    let (_, order) = test.get_foreign_order_account(&order_id)?;
    assert_eq!(order.data.status, OrderStatus::Completed);
    assert_eq!(order.data.amount_out_filled, 1_000_000u128);
    assert_eq!(order.data.amount_in_released, 1_000_000u128);

    let (_, pending) =
        test.get_pending_fill_report_account(&order_id, &solver.pubkey().to_bytes())?;
    assert_eq!(pending.amount_out_filled, 1_000_000u128);
    assert_eq!(pending.amount_in_to_release, 1_000_000u128);

    Ok(())
}

#[test]
fn fill_no_report_distinct_recipients_tracked_separately() -> Result<(), Box<dyn Error>> {
    let mut test = OrderBookTest::new()?;
    test.initialize()?;

    let mut order_data = default_foreign_order_data(&test, "alice");
    order_data.solver = [0u8; 32]; // no designated solver so two fillers can fill
    let order_id = order_data.compute_order_id();
    let solver = test.get_user("solver");
    let bob = test.get_user("bob");
    let carol = test.get_user("carol");

    // Two deferred fills by the same solver paying out to different origin recipients
    for (recipient, amount) in [(&bob, 300_000u64), (&carol, 200_000u64)] {
        let fill_params = order_book::instructions::FillParams {
            amount_out_to_fill: amount,
            origin_recipient: recipient.pubkey().to_bytes(),
        };
        let ix = test.create_fill_foreign_order_no_report_ix(
            &solver.pubkey(),
            &order_data,
            &fill_params,
        )?;
        test.ctx
            .execute_instruction(ix, &[&solver])?
            .assert_success();
    }

    let (_, pending_bob) =
        test.get_pending_fill_report_account(&order_id, &bob.pubkey().to_bytes())?;
    let (_, pending_carol) =
        test.get_pending_fill_report_account(&order_id, &carol.pubkey().to_bytes())?;
    assert_eq!(pending_bob.amount_out_filled, 300_000u128);
    assert_eq!(pending_bob.origin_recipient, bob.pubkey().to_bytes());
    assert_eq!(pending_carol.amount_out_filled, 200_000u128);
    assert_eq!(pending_carol.origin_recipient, carol.pubkey().to_bytes());

    Ok(())
}

#[test]
fn fill_no_report_accounting_parity_with_fill_foreign_order() -> Result<(), Box<dyn Error>> {
    let mut test = OrderBookTest::new()?;
    test.initialize()?;

    // Two identical orders except for the nonce; one reported, one deferred
    let reported_data = default_foreign_order_data(&test, "alice");
    let mut deferred_data = default_foreign_order_data(&test, "alice");
    deferred_data.nonce = 1;

    let amount = 250_000u64;
    test.fill_foreign_order("solver", &reported_data, amount)?;
    test.fill_foreign_order_no_report("solver", &deferred_data, amount)?;

    let (_, reported_order) =
        test.get_foreign_order_account(&reported_data.compute_order_id())?;
    let (_, deferred_order) =
        test.get_foreign_order_account(&deferred_data.compute_order_id())?;

    assert_eq!(
        reported_order.data.amount_out_filled,
        deferred_order.data.amount_out_filled,
        "amount_out_filled parity"
    );
    assert_eq!(
        reported_order.data.amount_in_released,
        deferred_order.data.amount_in_released,
        "amount_in_released parity"
    );
    assert_eq!(
        reported_order.data.status, deferred_order.data.status,
        "status parity"
    );

    Ok(())
}
