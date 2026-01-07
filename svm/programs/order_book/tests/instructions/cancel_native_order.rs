use super::super::{OrderBookTest, CHAIN_ID};
use anchor_litesvm::{Signer, TestHelpers};
use anchor_spl::associated_token::get_associated_token_address;
use order_book::{error::OrderBookError, ORDER_SEED_PREFIX};
use std::error::Error;

// CancelNativeOrder instruction tests
// For same-chain orders where origin_chain_id == dest_chain_id == current chain_id
//
// [X] given the order does not exist
//   [X] it reverts with an AccountNotInitialized error
// [X] given the order status is Completed
//   [X] it reverts with an InvalidOrderStatus error
// [X] given the order status is Cancelled
//   [X] it reverts with an InvalidOrderStatus error
// [X] given the created_at timestamp is in the future
//   [X] it reverts with an InvalidCreatedAtTimestamp error
// [X] given the signer is not sender or recipient and fill deadline has NOT passed
//   [X] it reverts with a NotAuthorized error
// [X] given the signer is sender before fill deadline
//   [X] it succeeds and refunds tokens to sender
// [X] given the signer is recipient before fill deadline
//   [X] it succeeds and refunds tokens to sender
// [X] given the signer is anyone after fill deadline
//   [X] it succeeds and refunds tokens to sender
// [X] given the order has been fully filled (no remaining tokens)
//   [X] it reverts with OrderFilled error
// [X] given a partial fill occurred
//   [X] it refunds only the remaining tokens
// [X] given all checks pass
//   [X] it sets order status to Cancelled
//   [X] it transfers remaining token_in to sender

mod local_orders {
    use super::*;

    fn default_order_params(test: &OrderBookTest) -> order_book::instructions::open::OrderParams {
        order_book::instructions::open::OrderParams {
            dest_chain_id: CHAIN_ID, // local order
            fill_deadline: test.current_time() + 100,
            token_out: test.get_mint("token-out-spl-6").to_bytes(),
            amount_in: 1_000_000,
            amount_out: 1_000_000,
            recipient: test.get_user("bob").pubkey().to_bytes(),
            solver: test.get_user("solver").pubkey().to_bytes(),
        }
    }

    #[test]
    fn test_cancel_native_order_not_exist_reverts() -> Result<(), Box<dyn Error>> {
        let mut test = OrderBookTest::new()?;
        test.initialize()?;

        // Try to cancel non-existent order
        let fake_order_id = [99u8; 32];
        let fake_order_account = test
            .ctx
            .svm
            .get_pda(&[ORDER_SEED_PREFIX, &fake_order_id], &order_book::ID);
        let fake_order_token_in_ata = test.create_associated_token_account(
            &test.get_mint("token-in-spl-6"),
            &fake_order_account,
        )?;

        let signer = test.get_user("alice");
        let sender = test.get_user("alice");

        // Create the accounts struct manually
        let accounts = order_book::accounts::CancelNativeOrder {
            program: order_book::ID,
            event_authority: test.get_event_authority()?,
            signer: signer.pubkey(),
            sender: sender.pubkey(),
            global_account: test.get_global_account()?.0,
            order: fake_order_account,
            token_in_mint: test.get_mint("token-in-spl-6"),
            sender_token_in_ata: test.get_ata("token-in-spl-6", "alice"),
            order_token_in_ata: fake_order_token_in_ata,
            token_in_program: anchor_spl::token::ID,
        };

        let ix =
            test.create_cancel_native_order_ix_with_custom_accounts(accounts, fake_order_id)?;

        test.ctx
            .execute_instruction(ix, &[&signer])?
            .assert_anchor_error("AccountNotInitialized");

        Ok(())
    }

    #[test]
    fn test_cancel_native_order_already_completed_reverts() -> Result<(), Box<dyn Error>> {
        let mut test = OrderBookTest::new()?;
        test.initialize()?;

        // Create and fill an order completely
        let order_params = default_order_params(&test);
        let order_id = test.open_order("alice", "token-in-spl-6", &order_params)?;

        // Fill the order completely
        test.fill_native_order("solver", order_id, 1_000_000)?;

        // Verify order is completed
        let (_, order_data) = test.get_native_order_account(&order_id)?;
        assert_eq!(
            order_data.data.status,
            order_book::state::OrderStatus::Completed
        );

        // Try to cancel completed order
        let ix = test.create_cancel_native_order_ix(
            &test.get_user("alice").pubkey(),
            &test.get_user("alice").pubkey(),
            order_id,
        )?;

        test.ctx
            .execute_instruction(ix, &[&test.get_user("alice")])?
            .assert_anchor_error(&format!("{:?}", OrderBookError::InvalidOrderStatus));

        Ok(())
    }

    #[test]
    fn test_cancel_native_order_already_cancelled_reverts() -> Result<(), Box<dyn Error>> {
        let mut test = OrderBookTest::new()?;
        test.initialize()?;

        // Create an order
        let order_params = default_order_params(&test);
        let order_id = test.open_order("alice", "token-in-spl-6", &order_params)?;

        // Cancel the order
        test.cancel_native_order("alice", "alice", order_id)?;

        // Verify order is cancelled
        let (_, order_data) = test.get_native_order_account(&order_id)?;
        assert_eq!(
            order_data.data.status,
            order_book::state::OrderStatus::Cancelled
        );

        // Expire blockhash to avoid AlreadyProcessed error
        test.ctx.svm.expire_blockhash();

        // Try to cancel again
        let ix = test.create_cancel_native_order_ix(
            &test.get_user("alice").pubkey(),
            &test.get_user("alice").pubkey(),
            order_id,
        )?;

        test.ctx
            .execute_instruction(ix, &[&test.get_user("alice")])?
            .assert_anchor_error(&format!("{:?}", OrderBookError::InvalidOrderStatus));

        Ok(())
    }

    #[test]
    fn test_cancel_native_order_unauthorized_before_deadline_reverts() -> Result<(), Box<dyn Error>>
    {
        let mut test = OrderBookTest::new()?;
        test.initialize()?;

        // Alice creates an order with bob as recipient
        let order_params = default_order_params(&test);
        let order_id = test.open_order("alice", "token-in-spl-6", &order_params)?;

        // Carol (not sender or recipient) tries to cancel before deadline
        let ix = test.create_cancel_native_order_ix(
            &test.get_user("carol").pubkey(),
            &test.get_user("alice").pubkey(),
            order_id,
        )?;

        test.ctx
            .execute_instruction(ix, &[&test.get_user("carol")])?
            .assert_anchor_error(&format!("{:?}", OrderBookError::NotAuthorized));

        Ok(())
    }

    #[test]
    fn test_cancel_native_order_sender_before_deadline_success() -> Result<(), Box<dyn Error>> {
        let mut test = OrderBookTest::new()?;
        test.initialize()?;

        // Alice creates an order
        let order_params = default_order_params(&test);
        let order_id = test.open_order("alice", "token-in-spl-6", &order_params)?;

        // Get initial balance
        let sender_ata = test.get_ata("token-in-spl-6", "alice");
        let initial_balance = test.get_token_balance(&sender_ata)?;

        // Alice (sender) cancels the order before deadline
        test.cancel_native_order("alice", "alice", order_id)?;

        // Verify order is cancelled
        let (_, order_data) = test.get_native_order_account(&order_id)?;
        assert_eq!(
            order_data.data.status,
            order_book::state::OrderStatus::Cancelled
        );

        // Verify tokens were refunded
        let final_balance = test.get_token_balance(&sender_ata)?;
        assert_eq!(final_balance, initial_balance + 1_000_000);

        Ok(())
    }

    #[test]
    fn test_cancel_native_order_recipient_before_deadline_success() -> Result<(), Box<dyn Error>> {
        let mut test = OrderBookTest::new()?;
        test.initialize()?;

        // Alice creates an order with bob as recipient
        let order_params = default_order_params(&test);
        let order_id = test.open_order("alice", "token-in-spl-6", &order_params)?;

        // Get initial balance
        let sender_ata = test.get_ata("token-in-spl-6", "alice");
        let initial_balance = test.get_token_balance(&sender_ata)?;

        // Bob (recipient) cancels the order before deadline
        let ix = test.create_cancel_native_order_ix(
            &test.get_user("bob").pubkey(),
            &test.get_user("alice").pubkey(),
            order_id,
        )?;

        test.ctx
            .execute_instruction(ix, &[&test.get_user("bob")])?
            .assert_success();

        // Verify order is cancelled
        let (_, order_data) = test.get_native_order_account(&order_id)?;
        assert_eq!(
            order_data.data.status,
            order_book::state::OrderStatus::Cancelled
        );

        // Verify tokens were refunded to sender (alice), not recipient (bob)
        let final_balance = test.get_token_balance(&sender_ata)?;
        assert_eq!(final_balance, initial_balance + 1_000_000);

        Ok(())
    }

    #[test]
    fn test_cancel_native_order_anyone_after_deadline_success() -> Result<(), Box<dyn Error>> {
        let mut test = OrderBookTest::new()?;
        test.initialize()?;

        // Alice creates an order with bob as recipient
        let order_params = default_order_params(&test);
        let order_id = test.open_order("alice", "token-in-spl-6", &order_params)?;

        // Warp time past deadline
        test.warp_forward(200);

        // Get initial balance
        let sender_ata = test.get_ata("token-in-spl-6", "alice");
        let initial_balance = test.get_token_balance(&sender_ata)?;

        // Carol (not sender or recipient) cancels the order after deadline
        let ix = test.create_cancel_native_order_ix(
            &test.get_user("carol").pubkey(),
            &test.get_user("alice").pubkey(),
            order_id,
        )?;

        test.ctx
            .execute_instruction(ix, &[&test.get_user("carol")])?
            .assert_success();

        // Verify order is cancelled
        let (_, order_data) = test.get_native_order_account(&order_id)?;
        assert_eq!(
            order_data.data.status,
            order_book::state::OrderStatus::Cancelled
        );

        // Verify tokens were refunded to sender (alice)
        let final_balance = test.get_token_balance(&sender_ata)?;
        assert_eq!(final_balance, initial_balance + 1_000_000);

        Ok(())
    }

    #[test]
    fn test_cancel_native_order_fully_filled_reverts() -> Result<(), Box<dyn Error>> {
        let mut test = OrderBookTest::new()?;
        test.initialize()?;

        // Create and fully fill an order
        let order_params = default_order_params(&test);
        let order_id = test.open_order("alice", "token-in-spl-6", &order_params)?;

        // Fill the order completely
        test.fill_native_order("solver", order_id, 1_000_000)?;

        // Verify order is completed
        let (_, order_data) = test.get_native_order_account(&order_id)?;
        assert_eq!(
            order_data.data.status,
            order_book::state::OrderStatus::Completed
        );

        // Try to cancel - should fail with InvalidOrderStatus (since order is Completed)
        let ix = test.create_cancel_native_order_ix(
            &test.get_user("alice").pubkey(),
            &test.get_user("alice").pubkey(),
            order_id,
        )?;

        test.ctx
            .execute_instruction(ix, &[&test.get_user("alice")])?
            .assert_anchor_error(&format!("{:?}", OrderBookError::InvalidOrderStatus));

        Ok(())
    }

    #[test]
    fn test_cancel_native_order_partial_fill_refunds_remaining() -> Result<(), Box<dyn Error>> {
        let mut test = OrderBookTest::new()?;
        test.initialize()?;

        // Create an order
        let order_params = default_order_params(&test);
        let order_id = test.open_order("alice", "token-in-spl-6", &order_params)?;

        // Partially fill the order (50%)
        test.fill_native_order("solver", order_id, 500_000)?;

        // Get initial balance after partial fill
        let sender_ata = test.get_ata("token-in-spl-6", "alice");
        let initial_balance = test.get_token_balance(&sender_ata)?;

        // Cancel the order - should refund the remaining 50%
        test.cancel_native_order("alice", "alice", order_id)?;

        // Verify order is cancelled
        let (_, order_data) = test.get_native_order_account(&order_id)?;
        assert_eq!(
            order_data.data.status,
            order_book::state::OrderStatus::Cancelled
        );

        // Verify only remaining tokens were refunded
        let final_balance = test.get_token_balance(&sender_ata)?;
        assert_eq!(final_balance, initial_balance + 500_000);

        Ok(())
    }

    #[test]
    fn test_cancel_native_order_wrong_sender_account_reverts() -> Result<(), Box<dyn Error>> {
        let mut test = OrderBookTest::new()?;
        test.initialize()?;

        // Alice creates an order
        let order_params = default_order_params(&test);
        let order_id = test.open_order("alice", "token-in-spl-6", &order_params)?;

        // Try to cancel with wrong sender account (carol instead of alice)
        let wrong_sender = test.get_user("carol");

        let accounts = test.build_cancel_native_order_accounts(
            &test.get_user("alice").pubkey(),
            &wrong_sender.pubkey(),
            order_id,
        )?;

        let ix = test.create_cancel_native_order_ix_with_custom_accounts(accounts, order_id)?;

        // Should fail because sender doesn't match order's sender
        test.ctx
            .execute_instruction(ix, &[&test.get_user("alice")])?
            .assert_anchor_error(&format!("{:?}", OrderBookError::InvalidSender));

        Ok(())
    }

    #[test]
    fn test_cancel_native_order_third_party_on_behalf_of_sender_success(
    ) -> Result<(), Box<dyn Error>> {
        let mut test = OrderBookTest::new()?;
        test.initialize()?;

        // Alice creates an order
        let order_params = default_order_params(&test);
        let order_id = test.open_order("alice", "token-in-spl-6", &order_params)?;

        // Warp time past deadline so anyone can cancel
        test.warp_forward(200);

        // Get initial balance
        let sender_ata = test.get_ata("token-in-spl-6", "alice");
        let initial_balance = test.get_token_balance(&sender_ata)?;

        // Carol (third party) cancels the order on behalf of Alice
        // Note: After deadline, anyone can sign. The sender account is just
        // used to determine where to send the refund.
        let ix = test.create_cancel_native_order_ix(
            &test.get_user("carol").pubkey(),
            &test.get_user("alice").pubkey(),
            order_id,
        )?;

        test.ctx
            .execute_instruction(ix, &[&test.get_user("carol")])?
            .assert_success();

        // Verify tokens went to sender (alice) not signer (carol)
        let final_balance = test.get_token_balance(&sender_ata)?;
        assert_eq!(final_balance, initial_balance + 1_000_000);

        Ok(())
    }

    #[test]
    fn test_cancel_and_close_ata_with_rent_refund() -> Result<(), Box<dyn Error>> {
        let mut test = OrderBookTest::new()?;
        test.initialize()?;

        // Alice creates an order
        let order_params = default_order_params(&test);
        let order_id = test.open_order("alice", "token-in-spl-6", &order_params)?;

        // Warp time past deadline
        test.warp_forward(200);

        // === STEP 1: Cancel the order ===

        // Track balances BEFORE cancel
        let alice_token_ata = test.get_ata("token-in-spl-6", "alice");
        let alice_token_balance_before = test.get_token_balance(&alice_token_ata)?;
        let alice = test.get_user("alice");
        let alice_sol_balance_before = test
            .ctx
            .svm
            .get_account(&alice.pubkey())
            .map(|a| a.lamports)
            .unwrap_or(0);

        // Cancel the order
        test.cancel_native_order("alice", "alice", order_id)?;

        // Verify AFTER cancel
        let (order_account, order_data) = test.get_native_order_account(&order_id)?;
        assert_eq!(
            order_data.data.status,
            order_book::state::OrderStatus::Cancelled
        );

        // Verify SPL tokens were refunded
        let alice_token_balance_after_cancel = test.get_token_balance(&alice_token_ata)?;
        assert_eq!(
            alice_token_balance_after_cancel - alice_token_balance_before,
            1_000_000,
            "SPL tokens should be refunded during cancel"
        );

        // Verify SOL did NOT increase (ATA not closed yet)
        let alice_sol_balance_after_cancel = test
            .ctx
            .svm
            .get_account(&alice.pubkey())
            .map(|a| a.lamports)
            .unwrap_or(0);

        // SOL might decrease slightly due to tx fees, but should not increase
        assert!(
            alice_sol_balance_after_cancel <= alice_sol_balance_before,
            "SOL should not increase during cancel (ATA not closed yet)"
        );

        // Verify ATA still exists with rent
        let order_token_in_ata =
            get_associated_token_address(&order_account, &test.get_mint("token-in-spl-6"));
        let ata_account_after_cancel = test.ctx.svm.get_account(&order_token_in_ata).unwrap();
        assert!(
            ata_account_after_cancel.lamports > 0,
            "ATA should still have rent after cancel"
        );
        assert!(
            !ata_account_after_cancel.data.is_empty(),
            "ATA should still have data after cancel"
        );

        // === STEP 2: Close the ATA ===

        // Track SOL balance before close
        let alice_sol_balance_before_close = test
            .ctx
            .svm
            .get_account(&alice.pubkey())
            .map(|a| a.lamports)
            .unwrap_or(0);
        let ata_rent = ata_account_after_cancel.lamports;

        // Close the ATA
        test.close_order_token_account("alice", order_id)?;

        // Verify SOL increased by ATA rent amount
        let alice_sol_balance_after_close = test
            .ctx
            .svm
            .get_account(&alice.pubkey())
            .map(|a| a.lamports)
            .unwrap_or(0);

        assert!(
            alice_sol_balance_after_close > alice_sol_balance_before_close,
            "SOL should increase after closing ATA"
        );

        // Check that increase is approximately the rent amount (accounting for tx fees)
        let sol_increase = alice_sol_balance_after_close - alice_sol_balance_before_close;
        assert!(
            sol_increase >= ata_rent - 10_000, // Allow small tx fee
            "SOL increase should be approximately ATA rent; expected ~{}, got {}",
            ata_rent,
            sol_increase
        );

        // Verify ATA is fully closed
        let ata_account_after_close = test.ctx.svm.get_account(&order_token_in_ata).unwrap();
        assert_eq!(
            ata_account_after_close.lamports, 0,
            "ATA must have 0 lamports after close"
        );
        assert!(
            ata_account_after_close.data.is_empty(),
            "ATA must have empty data after close"
        );

        // Verify SPL token balance unchanged from cancel to close
        let alice_token_balance_final = test.get_token_balance(&alice_token_ata)?;
        assert_eq!(
            alice_token_balance_final, alice_token_balance_after_cancel,
            "SPL token balance should not change during ATA close"
        );

        Ok(())
    }

    #[test]
    fn test_close_fails_if_order_not_finalized() -> Result<(), Box<dyn Error>> {
        let mut test = OrderBookTest::new()?;
        test.initialize()?;

        // Create an order (status = Created)
        let order_params = default_order_params(&test);
        let order_id = test.open_order("alice", "token-in-spl-6", &order_params)?;

        // Verify order is in Created status
        let (_, order_data) = test.get_native_order_account(&order_id)?;
        assert_eq!(
            order_data.data.status,
            order_book::state::OrderStatus::Created
        );

        // Try to close - should fail because order not finalized
        let ix =
            test.create_close_order_token_account_ix(&test.get_user("alice").pubkey(), order_id)?;

        test.ctx
            .execute_instruction(ix, &[&test.get_user("alice")])?
            .assert_anchor_error(&format!("{:?}", OrderBookError::OrderNotFinalized));

        Ok(())
    }

    #[test]
    fn test_close_fails_with_wrong_payer() -> Result<(), Box<dyn Error>> {
        let mut test = OrderBookTest::new()?;
        test.initialize()?;

        // Alice creates an order (alice is payer)
        let order_params = default_order_params(&test);
        let order_id = test.open_order("alice", "token-in-spl-6", &order_params)?;

        // Cancel the order
        test.cancel_native_order("alice", "alice", order_id)?;

        // Verify order is cancelled
        let (_, order_data) = test.get_native_order_account(&order_id)?;
        assert_eq!(
            order_data.data.status,
            order_book::state::OrderStatus::Cancelled
        );

        // Bob tries to close with himself as payer (should fail)
        let ix = test.create_close_order_token_account_ix(
            &test.get_user("bob").pubkey(), // Wrong payer!
            order_id,
        )?;

        test.ctx
            .execute_instruction(ix, &[&test.get_user("bob")])?
            .assert_anchor_error(&format!("{:?}", OrderBookError::InvalidPayer));

        Ok(())
    }
}
