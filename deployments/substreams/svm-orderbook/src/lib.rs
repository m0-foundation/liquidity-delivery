mod idl;
#[allow(unused)]
mod pb;

use anchor_lang::AnchorDeserialize;
use anchor_lang::Discriminator;
use base64::prelude::*;
use pb::substreams::v1::program::AcceptAdminRoleInstruction;
use pb::substreams::v1::program::AddDestinationInstruction;
use pb::substreams::v1::program::CancelForeignOrderInstruction;
use pb::substreams::v1::program::CancelNativeOrderInstruction;
use pb::substreams::v1::program::CancelReport;
use pb::substreams::v1::program::CancelReportedEvent;
use pb::substreams::v1::program::ClearNewAdminInstruction;
use pb::substreams::v1::program::CloseOrderTokenAccountInstruction;
use pb::substreams::v1::program::Data;
use pb::substreams::v1::program::DestinationSupportUpdatedEvent;
use pb::substreams::v1::program::FillForeignOrderInstruction;
use pb::substreams::v1::program::FillNativeOrderInstruction;
use pb::substreams::v1::program::FillParams;
use pb::substreams::v1::program::FillReport;
use pb::substreams::v1::program::FillReportedEvent;
use pb::substreams::v1::program::ForeignOrder;
use pb::substreams::v1::program::IdlInstructionInstruction;
use pb::substreams::v1::program::InitializeInstruction;
use pb::substreams::v1::program::NativeOrder;
use pb::substreams::v1::program::OpenOrderInstruction;
use pb::substreams::v1::program::OrderCancelledEvent;
use pb::substreams::v1::program::OrderCompletedEvent;
use pb::substreams::v1::program::OrderData;
use pb::substreams::v1::program::OrderFilledEvent;
use pb::substreams::v1::program::OrderOpenedEvent;
use pb::substreams::v1::program::OrderParams;
use pb::substreams::v1::program::PauseInstruction;
use pb::substreams::v1::program::RefundClaimedEvent;
use pb::substreams::v1::program::RemoveDestinationInstruction;
use pb::substreams::v1::program::ReportOrderCancelInstruction;
use pb::substreams::v1::program::ReportOrderFillInstruction;
use pb::substreams::v1::program::SetNewAdminInstruction;
use pb::substreams::v1::program::SetPortalAuthorityInstruction;
use pb::substreams::v1::program::UnpauseInstruction;
use sologger_log_context::programs_selector::ProgramsSelector;
use sologger_log_context::sologger_log_context::LogContext;
use substreams_solana::pb::sf::solana::r#type::v1::Block;

const PROGRAM_ID: &str = "MzLoYnJ6sF6eeejs4vV95TNmXqS3W4cAtLGKkjT4ZrK";

#[substreams::handlers::map]
fn map_program_data(blk: Block) -> Data {
    let mut cancel_reported_event_list: Vec<CancelReportedEvent> = Vec::new();
    let mut destination_support_updated_event_list: Vec<DestinationSupportUpdatedEvent> =
        Vec::new();
    let mut fill_reported_event_list: Vec<FillReportedEvent> = Vec::new();
    let mut order_cancelled_event_list: Vec<OrderCancelledEvent> = Vec::new();
    let mut order_completed_event_list: Vec<OrderCompletedEvent> = Vec::new();
    let mut order_filled_event_list: Vec<OrderFilledEvent> = Vec::new();
    let mut order_opened_event_list: Vec<OrderOpenedEvent> = Vec::new();
    let mut refund_claimed_event_list: Vec<RefundClaimedEvent> = Vec::new();
    let mut accept_admin_role_instruction_list: Vec<AcceptAdminRoleInstruction> = Vec::new();
    let mut add_destination_instruction_list: Vec<AddDestinationInstruction> = Vec::new();
    let mut cancel_foreign_order_instruction_list: Vec<CancelForeignOrderInstruction> = Vec::new();
    let mut cancel_native_order_instruction_list: Vec<CancelNativeOrderInstruction> = Vec::new();
    let mut clear_new_admin_instruction_list: Vec<ClearNewAdminInstruction> = Vec::new();
    let mut close_order_token_account_instruction_list: Vec<CloseOrderTokenAccountInstruction> =
        Vec::new();
    let mut fill_foreign_order_instruction_list: Vec<FillForeignOrderInstruction> = Vec::new();
    let mut fill_native_order_instruction_list: Vec<FillNativeOrderInstruction> = Vec::new();
    let mut idl_instruction_instruction_list: Vec<IdlInstructionInstruction> = Vec::new();
    let mut initialize_instruction_list: Vec<InitializeInstruction> = Vec::new();
    let mut open_order_instruction_list: Vec<OpenOrderInstruction> = Vec::new();
    let mut pause_instruction_list: Vec<PauseInstruction> = Vec::new();
    let mut remove_destination_instruction_list: Vec<RemoveDestinationInstruction> = Vec::new();
    let mut report_order_cancel_instruction_list: Vec<ReportOrderCancelInstruction> = Vec::new();
    let mut report_order_fill_instruction_list: Vec<ReportOrderFillInstruction> = Vec::new();
    let mut set_new_admin_instruction_list: Vec<SetNewAdminInstruction> = Vec::new();
    let mut set_portal_authority_instruction_list: Vec<SetPortalAuthorityInstruction> = Vec::new();
    let mut unpause_instruction_list: Vec<UnpauseInstruction> = Vec::new();

    blk.transactions().for_each(|transaction| {
        let meta_wrapped = &transaction.meta;
        let meta = meta_wrapped.as_ref().unwrap();
        let programs_selector: ProgramsSelector = ProgramsSelector::new(&["*".to_string()]);
        let log_contexts = LogContext::parse_logs_basic(&meta.log_messages, &programs_selector);

        log_contexts
            .iter()
            .filter(|context| context.program_id == PROGRAM_ID)
            .for_each(|context| {
                context.data_logs.iter().for_each(|data| {
                    if let Ok(decoded) = BASE64_STANDARD.decode(data) {
                        let slice_u8: &mut &[u8] = &mut &decoded[..];
                        let slice_discriminator: [u8; 8] =
                            slice_u8[0..8].try_into().expect("error");
                        let static_discriminator_slice: &'static [u8] = Box::leak(Box::new(slice_discriminator));

                        match static_discriminator_slice {
                            idl::idl::program::events::CancelReported::DISCRIMINATOR => {
                                if let Ok(event) =
                                    idl::idl::program::events::CancelReported::deserialize(
                                        &mut &slice_u8[8..],
                                    )
                                {
                                    cancel_reported_event_list.push(CancelReportedEvent {
                                        trx_hash: transaction.id(),
                                        order_id: event.order_id.into_iter().map(|f| f as u64).collect(),
                                    });
                                }
                            }
                            idl::idl::program::events::DestinationSupportUpdated::DISCRIMINATOR => {
                                if let Ok(event) =
                                    idl::idl::program::events::DestinationSupportUpdated::deserialize(
                                        &mut &slice_u8[8..],
                                    )
                                {
                                    destination_support_updated_event_list.push(DestinationSupportUpdatedEvent {
                                        trx_hash: transaction.id(),
                                        dest_chain_id: event.dest_chain_id,
                                        is_supported: event.is_supported,
                                    });
                                }
                            }
                            idl::idl::program::events::FillReported::DISCRIMINATOR => {
                                if let Ok(event) =
                                    idl::idl::program::events::FillReported::deserialize(
                                        &mut &slice_u8[8..],
                                    )
                                {
                                    fill_reported_event_list.push(FillReportedEvent {
                                        trx_hash: transaction.id(),
                                        order_id: event.order_id.into_iter().map(|f| f as u64).collect(),
                                        amount_in_to_release: event.amount_in_to_release as u64,
                                        amount_out_filled: event.amount_out_filled as u64,
                                        origin_recipient: event.origin_recipient.into_iter().map(|f| f as u64).collect(),
                                    });
                                }
                            }
                            idl::idl::program::events::OrderCancelled::DISCRIMINATOR => {
                                if let Ok(event) =
                                    idl::idl::program::events::OrderCancelled::deserialize(
                                        &mut &slice_u8[8..],
                                    )
                                {
                                    order_cancelled_event_list.push(OrderCancelledEvent {
                                        trx_hash: transaction.id(),
                                        order_id: event.order_id.into_iter().map(|f| f as u64).collect(),
                                    });
                                }
                            }
                            idl::idl::program::events::OrderCompleted::DISCRIMINATOR => {
                                if let Ok(event) =
                                    idl::idl::program::events::OrderCompleted::deserialize(
                                        &mut &slice_u8[8..],
                                    )
                                {
                                    order_completed_event_list.push(OrderCompletedEvent {
                                        trx_hash: transaction.id(),
                                        order_id: event.order_id.into_iter().map(|f| f as u64).collect(),
                                    });
                                }
                            }
                            idl::idl::program::events::OrderFilled::DISCRIMINATOR => {
                                if let Ok(event) =
                                    idl::idl::program::events::OrderFilled::deserialize(
                                        &mut &slice_u8[8..],
                                    )
                                {
                                    order_filled_event_list.push(OrderFilledEvent {
                                        trx_hash: transaction.id(),
                                        order_id: event.order_id.into_iter().map(|f| f as u64).collect(),
                                        solver: event.solver.to_string(),
                                        amount_in_to_release: event.amount_in_to_release as u64,
                                        amount_out_filled: event.amount_out_filled as u64,
                                    });
                                }
                            }
                            idl::idl::program::events::OrderOpened::DISCRIMINATOR => {
                                if let Ok(event) =
                                    idl::idl::program::events::OrderOpened::deserialize(
                                        &mut &slice_u8[8..],
                                    )
                                {
                                    order_opened_event_list.push(OrderOpenedEvent {
                                        trx_hash: transaction.id(),
                                        order_id: event.order_id.into_iter().map(|f| f as u64).collect(),
                                        sender: event.sender.to_string(),
                                        token_in: event.token_in.to_string(),
                                        amount_in: event.amount_in,
                                        dest_chain_id: event.dest_chain_id,
                                        token_out: event.token_out.into_iter().map(|f| f as u64).collect(),
                                        amount_out: event.amount_out as u64,
                                        solver: event.solver.into_iter().map(|f| f as u64).collect(),
                                    });
                                }
                            }
                            idl::idl::program::events::RefundClaimed::DISCRIMINATOR => {
                                if let Ok(event) =
                                    idl::idl::program::events::RefundClaimed::deserialize(
                                        &mut &slice_u8[8..],
                                    )
                                {
                                    refund_claimed_event_list.push(RefundClaimedEvent {
                                        trx_hash: transaction.id(),
                                        order_id: event.order_id.into_iter().map(|f| f as u64).collect(),
                                        sender: event.sender.to_string(),
                                        amount: event.amount,
                                    });
                                }
                            }
                            _ => {}
                        }
                    }
                });
            });

        // ------------- INSTRUCTIONS -------------
        transaction
        .walk_instructions()
        .into_iter()
        .filter(|inst| inst.program_id().to_string() == PROGRAM_ID)
        .for_each(|inst| {
            let slice_u8: &[u8] = &inst.data()[..];

            /*
                CPI events are contained inside the instruction data
            */
            if slice_u8.len() >= 16 {
                if &slice_u8[8..16] == idl::idl::program::events::CancelReported::DISCRIMINATOR {
                    if let Ok(event) =
                        idl::idl::program::events::CancelReported::deserialize(
                            &mut &slice_u8[16..],
                        )
                    {
                        cancel_reported_event_list.push(CancelReportedEvent {
                            trx_hash: transaction.id(),
                            order_id: event.order_id.into_iter().map(|f| f as u64).collect(),
                        });
                    }
                }
                if &slice_u8[8..16] == idl::idl::program::events::DestinationSupportUpdated::DISCRIMINATOR {
                    if let Ok(event) =
                        idl::idl::program::events::DestinationSupportUpdated::deserialize(
                            &mut &slice_u8[16..],
                        )
                    {
                        destination_support_updated_event_list.push(DestinationSupportUpdatedEvent {
                            trx_hash: transaction.id(),
                            dest_chain_id: event.dest_chain_id,
                            is_supported: event.is_supported,
                        });
                    }
                }
                if &slice_u8[8..16] == idl::idl::program::events::FillReported::DISCRIMINATOR {
                    if let Ok(event) =
                        idl::idl::program::events::FillReported::deserialize(
                            &mut &slice_u8[16..],
                        )
                    {
                        fill_reported_event_list.push(FillReportedEvent {
                            trx_hash: transaction.id(),
                            order_id: event.order_id.into_iter().map(|f| f as u64).collect(),
                            amount_in_to_release: event.amount_in_to_release as u64,
                            amount_out_filled: event.amount_out_filled as u64,
                            origin_recipient: event.origin_recipient.into_iter().map(|f| f as u64).collect(),
                        });
                    }
                }
                if &slice_u8[8..16] == idl::idl::program::events::OrderCancelled::DISCRIMINATOR {
                    if let Ok(event) =
                        idl::idl::program::events::OrderCancelled::deserialize(
                            &mut &slice_u8[16..],
                        )
                    {
                        order_cancelled_event_list.push(OrderCancelledEvent {
                            trx_hash: transaction.id(),
                            order_id: event.order_id.into_iter().map(|f| f as u64).collect(),
                        });
                    }
                }
                if &slice_u8[8..16] == idl::idl::program::events::OrderCompleted::DISCRIMINATOR {
                    if let Ok(event) =
                        idl::idl::program::events::OrderCompleted::deserialize(
                            &mut &slice_u8[16..],
                        )
                    {
                        order_completed_event_list.push(OrderCompletedEvent {
                            trx_hash: transaction.id(),
                            order_id: event.order_id.into_iter().map(|f| f as u64).collect(),
                        });
                    }
                }
                if &slice_u8[8..16] == idl::idl::program::events::OrderFilled::DISCRIMINATOR {
                    if let Ok(event) =
                        idl::idl::program::events::OrderFilled::deserialize(
                            &mut &slice_u8[16..],
                        )
                    {
                        order_filled_event_list.push(OrderFilledEvent {
                            trx_hash: transaction.id(),
                            order_id: event.order_id.into_iter().map(|f| f as u64).collect(),
                            solver: event.solver.to_string(),
                            amount_in_to_release: event.amount_in_to_release as u64,
                            amount_out_filled: event.amount_out_filled as u64,
                        });
                    }
                }
                if &slice_u8[8..16] == idl::idl::program::events::OrderOpened::DISCRIMINATOR {
                    if let Ok(event) =
                        idl::idl::program::events::OrderOpened::deserialize(
                            &mut &slice_u8[16..],
                        )
                    {
                        order_opened_event_list.push(OrderOpenedEvent {
                            trx_hash: transaction.id(),
                            order_id: event.order_id.into_iter().map(|f| f as u64).collect(),
                            sender: event.sender.to_string(),
                            token_in: event.token_in.to_string(),
                            amount_in: event.amount_in,
                            dest_chain_id: event.dest_chain_id,
                            token_out: event.token_out.into_iter().map(|f| f as u64).collect(),
                            amount_out: event.amount_out as u64,
                            solver: event.solver.into_iter().map(|f| f as u64).collect(),
                        });
                    }
                }
                if &slice_u8[8..16] == idl::idl::program::events::RefundClaimed::DISCRIMINATOR {
                    if let Ok(event) =
                        idl::idl::program::events::RefundClaimed::deserialize(
                            &mut &slice_u8[16..],
                        )
                    {
                        refund_claimed_event_list.push(RefundClaimedEvent {
                            trx_hash: transaction.id(),
                            order_id: event.order_id.into_iter().map(|f| f as u64).collect(),
                            sender: event.sender.to_string(),
                            amount: event.amount,
                        });
                    }
                }
            }
            if &slice_u8[0..8] == idl::idl::program::client::args::AcceptAdminRole::DISCRIMINATOR {
                if let Ok(instruction) =
                    idl::idl::program::client::args::AcceptAdminRole::deserialize(&mut &slice_u8[8..])
                {
                    let accts = inst.accounts();
                    accept_admin_role_instruction_list.push(AcceptAdminRoleInstruction {
                        trx_hash: transaction.id(),
                        acct_new_admin: accts[0].to_string(),
                        acct_global_account: accts[1].to_string(),
                    });
                }
            }
            if &slice_u8[0..8] == idl::idl::program::client::args::AddDestination::DISCRIMINATOR {
                if let Ok(instruction) =
                    idl::idl::program::client::args::AddDestination::deserialize(&mut &slice_u8[8..])
                {
                    let accts = inst.accounts();
                    add_destination_instruction_list.push(AddDestinationInstruction {
                        trx_hash: transaction.id(),
                        dest_chain_id: instruction.dest_chain_id,
                        acct_admin: accts[0].to_string(),
                        acct_global_account: accts[1].to_string(),
                        acct_destination_account: accts[2].to_string(),
                        acct_event_authority: accts[4].to_string(),
                        acct_program: accts[5].to_string(),
                    });
                }
            }
            if &slice_u8[0..8] == idl::idl::program::client::args::CancelForeignOrder::DISCRIMINATOR {
                if let Ok(instruction) =
                    idl::idl::program::client::args::CancelForeignOrder::deserialize(&mut &slice_u8[8..])
                {
                    let accts = inst.accounts();
                    cancel_foreign_order_instruction_list.push(CancelForeignOrderInstruction {
                        trx_hash: transaction.id(),
                        order_id: instruction.order_id.into_iter().map(|f| f as u64).collect(),
                        order_data: Some(OrderData {
						version: instruction.order_data.version as u64,sender: instruction.order_data.sender.into_iter().map(|f| f as u64).collect(),nonce: instruction.order_data.nonce,origin_chain_id: instruction.order_data.origin_chain_id,dest_chain_id: instruction.order_data.dest_chain_id,created_at: instruction.order_data.created_at,fill_deadline: instruction.order_data.fill_deadline,token_in: instruction.order_data.token_in.into_iter().map(|f| f as u64).collect(),token_out: instruction.order_data.token_out.into_iter().map(|f| f as u64).collect(),amount_in: instruction.order_data.amount_in as u64,amount_out: instruction.order_data.amount_out as u64,recipient: instruction.order_data.recipient.into_iter().map(|f| f as u64).collect(),solver: instruction.order_data.solver.into_iter().map(|f| f as u64).collect(),
					}),
                        acct_signer: accts[0].to_string(),
                        acct_global_account: accts[1].to_string(),
                        acct_order: accts[2].to_string(),
                        acct_portal_global: accts[4].to_string(),
                        acct_portal_authority: accts[5].to_string(),
                        acct_bridge_adapter: accts[6].to_string(),
                        acct_event_authority: accts[8].to_string(),
                        acct_program: accts[9].to_string(),
                    });
                }
            }
            if &slice_u8[0..8] == idl::idl::program::client::args::CancelNativeOrder::DISCRIMINATOR {
                if let Ok(instruction) =
                    idl::idl::program::client::args::CancelNativeOrder::deserialize(&mut &slice_u8[8..])
                {
                    let accts = inst.accounts();
                    cancel_native_order_instruction_list.push(CancelNativeOrderInstruction {
                        trx_hash: transaction.id(),
                        order_id: instruction.order_id.into_iter().map(|f| f as u64).collect(),
                        acct_signer: accts[0].to_string(),
                        acct_sender: accts[1].to_string(),
                        acct_global_account: accts[2].to_string(),
                        acct_order: accts[3].to_string(),
                        acct_token_in_mint: accts[4].to_string(),
                        acct_sender_token_in_ata: accts[5].to_string(),
                        acct_order_token_in_ata: accts[6].to_string(),
                        acct_token_in_program: accts[7].to_string(),
                        acct_event_authority: accts[8].to_string(),
                        acct_program: accts[9].to_string(),
                    });
                }
            }
            if &slice_u8[0..8] == idl::idl::program::client::args::ClearNewAdmin::DISCRIMINATOR {
                if let Ok(instruction) =
                    idl::idl::program::client::args::ClearNewAdmin::deserialize(&mut &slice_u8[8..])
                {
                    let accts = inst.accounts();
                    clear_new_admin_instruction_list.push(ClearNewAdminInstruction {
                        trx_hash: transaction.id(),
                        acct_admin: accts[0].to_string(),
                        acct_global_account: accts[1].to_string(),
                    });
                }
            }
            if &slice_u8[0..8] == idl::idl::program::client::args::CloseOrderTokenAccount::DISCRIMINATOR {
                if let Ok(instruction) =
                    idl::idl::program::client::args::CloseOrderTokenAccount::deserialize(&mut &slice_u8[8..])
                {
                    let accts = inst.accounts();
                    close_order_token_account_instruction_list.push(CloseOrderTokenAccountInstruction {
                        trx_hash: transaction.id(),
                        order_id: instruction.order_id.into_iter().map(|f| f as u64).collect(),
                        acct_payer: accts[0].to_string(),
                        acct_sender: accts[1].to_string(),
                        acct_order: accts[2].to_string(),
                        acct_token_in_mint: accts[3].to_string(),
                        acct_order_token_in_ata: accts[4].to_string(),
                        acct_recipient_token_account: accts[5].to_string(),
                        acct_token_in_program: accts[6].to_string(),
                    });
                }
            }
            if &slice_u8[0..8] == idl::idl::program::client::args::FillForeignOrder::DISCRIMINATOR {
                if let Ok(instruction) =
                    idl::idl::program::client::args::FillForeignOrder::deserialize(&mut &slice_u8[8..])
                {
                    let accts = inst.accounts();
                    fill_foreign_order_instruction_list.push(FillForeignOrderInstruction {
                        trx_hash: transaction.id(),
                        order_id: instruction.order_id.into_iter().map(|f| f as u64).collect(),
                        order_data: Some(OrderData {
						version: instruction.order_data.version as u64,sender: instruction.order_data.sender.into_iter().map(|f| f as u64).collect(),nonce: instruction.order_data.nonce,origin_chain_id: instruction.order_data.origin_chain_id,dest_chain_id: instruction.order_data.dest_chain_id,created_at: instruction.order_data.created_at,fill_deadline: instruction.order_data.fill_deadline,token_in: instruction.order_data.token_in.into_iter().map(|f| f as u64).collect(),token_out: instruction.order_data.token_out.into_iter().map(|f| f as u64).collect(),amount_in: instruction.order_data.amount_in as u64,amount_out: instruction.order_data.amount_out as u64,recipient: instruction.order_data.recipient.into_iter().map(|f| f as u64).collect(),solver: instruction.order_data.solver.into_iter().map(|f| f as u64).collect(),
					}),
                        fill_params: Some(FillParams {
						amount_out_to_fill: instruction.fill_params.amount_out_to_fill,origin_recipient: instruction.fill_params.origin_recipient.into_iter().map(|f| f as u64).collect(),
					}),
                        acct_solver: accts[0].to_string(),
                        acct_global_account: accts[1].to_string(),
                        acct_token_out_mint: accts[2].to_string(),
                        acct_solver_token_out_account: accts[3].to_string(),
                        acct_recipient: accts[4].to_string(),
                        acct_recipient_token_out_ata: accts[5].to_string(),
                        acct_token_out_program: accts[6].to_string(),
                        acct_order: accts[9].to_string(),
                        acct_portal_global: accts[11].to_string(),
                        acct_portal_authority: accts[12].to_string(),
                        acct_bridge_adapter: accts[13].to_string(),
                        acct_event_authority: accts[14].to_string(),
                        acct_program: accts[15].to_string(),
                    });
                }
            }
            if &slice_u8[0..8] == idl::idl::program::client::args::FillNativeOrder::DISCRIMINATOR {
                if let Ok(instruction) =
                    idl::idl::program::client::args::FillNativeOrder::deserialize(&mut &slice_u8[8..])
                {
                    let accts = inst.accounts();
                    fill_native_order_instruction_list.push(FillNativeOrderInstruction {
                        trx_hash: transaction.id(),
                        order_id: instruction.order_id.into_iter().map(|f| f as u64).collect(),
                        order_data: Some(OrderData {
						version: instruction.order_data.version as u64,sender: instruction.order_data.sender.into_iter().map(|f| f as u64).collect(),nonce: instruction.order_data.nonce,origin_chain_id: instruction.order_data.origin_chain_id,dest_chain_id: instruction.order_data.dest_chain_id,created_at: instruction.order_data.created_at,fill_deadline: instruction.order_data.fill_deadline,token_in: instruction.order_data.token_in.into_iter().map(|f| f as u64).collect(),token_out: instruction.order_data.token_out.into_iter().map(|f| f as u64).collect(),amount_in: instruction.order_data.amount_in as u64,amount_out: instruction.order_data.amount_out as u64,recipient: instruction.order_data.recipient.into_iter().map(|f| f as u64).collect(),solver: instruction.order_data.solver.into_iter().map(|f| f as u64).collect(),
					}),
                        fill_params: Some(FillParams {
						amount_out_to_fill: instruction.fill_params.amount_out_to_fill,origin_recipient: instruction.fill_params.origin_recipient.into_iter().map(|f| f as u64).collect(),
					}),
                        acct_solver: accts[0].to_string(),
                        acct_global_account: accts[1].to_string(),
                        acct_token_out_mint: accts[2].to_string(),
                        acct_solver_token_out_account: accts[3].to_string(),
                        acct_recipient: accts[4].to_string(),
                        acct_recipient_token_out_ata: accts[5].to_string(),
                        acct_token_out_program: accts[6].to_string(),
                        acct_order: accts[9].to_string(),
                        acct_token_in_mint: accts[10].to_string(),
                        acct_solver_token_in_account: accts[11].to_string(),
                        acct_order_token_in_ata: accts[12].to_string(),
                        acct_token_in_program: accts[13].to_string(),
                        acct_event_authority: accts[14].to_string(),
                        acct_program: accts[15].to_string(),
                    });
                }
            }
            if &slice_u8[0..8] == idl::idl::program::client::args::IdlInstruction::DISCRIMINATOR {
                if let Ok(instruction) =
                    idl::idl::program::client::args::IdlInstruction::deserialize(&mut &slice_u8[8..])
                {
                    let accts = inst.accounts();
                    idl_instruction_instruction_list.push(IdlInstructionInstruction {
                        trx_hash: transaction.id(),
                        foreign: Some(ForeignOrder {
						status: map_enum_order_status(instruction.foreign.status),amount_in_released: instruction.foreign.amount_in_released as u64,amount_out_filled: instruction.foreign.amount_out_filled as u64,amount_in_refunded: instruction.foreign.amount_in_refunded as u64,
					}),
                        native: Some(NativeOrder {
						status: map_enum_order_status(instruction.native.status),version: instruction.native.version as u64,sender: instruction.native.sender.to_string(),payer: instruction.native.payer.to_string(),nonce: instruction.native.nonce,dest_chain_id: instruction.native.dest_chain_id,created_at: instruction.native.created_at,fill_deadline: instruction.native.fill_deadline,token_in: instruction.native.token_in.to_string(),token_out: instruction.native.token_out.into_iter().map(|f| f as u64).collect(),amount_in: instruction.native.amount_in as u64,amount_out: instruction.native.amount_out as u64,recipient: instruction.native.recipient.into_iter().map(|f| f as u64).collect(),solver: instruction.native.solver.into_iter().map(|f| f as u64).collect(),amount_in_released: instruction.native.amount_in_released as u64,amount_out_filled: instruction.native.amount_out_filled as u64,amount_in_refunded: instruction.native.amount_in_refunded as u64,
					}),
                    });
                }
            }
            if &slice_u8[0..8] == idl::idl::program::client::args::Initialize::DISCRIMINATOR {
                if let Ok(instruction) =
                    idl::idl::program::client::args::Initialize::deserialize(&mut &slice_u8[8..])
                {
                    let accts = inst.accounts();
                    initialize_instruction_list.push(InitializeInstruction {
                        trx_hash: transaction.id(),
                        chain_id: instruction.chain_id,
                        portal_authority: instruction.portal_authority.to_string(),
                        acct_admin: accts[0].to_string(),
                        acct_global_account: accts[1].to_string(),
                    });
                }
            }
            if &slice_u8[0..8] == idl::idl::program::client::args::OpenOrder::DISCRIMINATOR {
                if let Ok(instruction) =
                    idl::idl::program::client::args::OpenOrder::deserialize(&mut &slice_u8[8..])
                {
                    let accts = inst.accounts();
                    open_order_instruction_list.push(OpenOrderInstruction {
                        trx_hash: transaction.id(),
                        params: Some(OrderParams {
						dest_chain_id: instruction.params.dest_chain_id,created_at: instruction.params.created_at,fill_deadline: instruction.params.fill_deadline,token_out: instruction.params.token_out.into_iter().map(|f| f as u64).collect(),amount_in: instruction.params.amount_in,amount_out: instruction.params.amount_out as u64,recipient: instruction.params.recipient.into_iter().map(|f| f as u64).collect(),solver: instruction.params.solver.into_iter().map(|f| f as u64).collect(),
					}),
                        acct_payer: accts[0].to_string(),
                        acct_token_authority: accts[1].to_string(),
                        acct_global_account: accts[2].to_string(),
                        acct_destination_account: accts[3].to_string(),
                        acct_token_in_mint: accts[4].to_string(),
                        acct_sender_token_in_account: accts[5].to_string(),
                        acct_sender_nonce_account: accts[6].to_string(),
                        acct_order: accts[7].to_string(),
                        acct_order_token_in_ata: accts[8].to_string(),
                        acct_token_in_program: accts[9].to_string(),
                        acct_event_authority: accts[12].to_string(),
                        acct_program: accts[13].to_string(),
                    });
                }
            }
            if &slice_u8[0..8] == idl::idl::program::client::args::Pause::DISCRIMINATOR {
                if let Ok(instruction) =
                    idl::idl::program::client::args::Pause::deserialize(&mut &slice_u8[8..])
                {
                    let accts = inst.accounts();
                    pause_instruction_list.push(PauseInstruction {
                        trx_hash: transaction.id(),
                        acct_admin: accts[0].to_string(),
                        acct_global_account: accts[1].to_string(),
                    });
                }
            }
            if &slice_u8[0..8] == idl::idl::program::client::args::RemoveDestination::DISCRIMINATOR {
                if let Ok(instruction) =
                    idl::idl::program::client::args::RemoveDestination::deserialize(&mut &slice_u8[8..])
                {
                    let accts = inst.accounts();
                    remove_destination_instruction_list.push(RemoveDestinationInstruction {
                        trx_hash: transaction.id(),
                        dest_chain_id: instruction.dest_chain_id,
                        acct_admin: accts[0].to_string(),
                        acct_global_account: accts[1].to_string(),
                        acct_destination_account: accts[2].to_string(),
                        acct_event_authority: accts[4].to_string(),
                        acct_program: accts[5].to_string(),
                    });
                }
            }
            if &slice_u8[0..8] == idl::idl::program::client::args::ReportOrderCancel::DISCRIMINATOR {
                if let Ok(instruction) =
                    idl::idl::program::client::args::ReportOrderCancel::deserialize(&mut &slice_u8[8..])
                {
                    let accts = inst.accounts();
                    report_order_cancel_instruction_list.push(ReportOrderCancelInstruction {
                        trx_hash: transaction.id(),
                        source_chain_id: instruction.source_chain_id,
                        cancel_report: Some(CancelReport {
						order_id: instruction.cancel_report.order_id.into_iter().map(|f| f as u64).collect(),order_sender: instruction.cancel_report.order_sender.into_iter().map(|f| f as u64).collect(),token_in: instruction.cancel_report.token_in.into_iter().map(|f| f as u64).collect(),amount_in_to_refund: instruction.cancel_report.amount_in_to_refund as u64,
					}),
                        acct_relayer: accts[0].to_string(),
                        acct_portal_authority: accts[1].to_string(),
                        acct_global_account: accts[2].to_string(),
                        acct_order: accts[3].to_string(),
                        acct_token_in_mint: accts[4].to_string(),
                        acct_order_sender: accts[5].to_string(),
                        acct_sender_token_in_ata: accts[6].to_string(),
                        acct_order_token_in_ata: accts[7].to_string(),
                        acct_token_in_program: accts[8].to_string(),
                        acct_event_authority: accts[11].to_string(),
                        acct_program: accts[12].to_string(),
                    });
                }
            }
            if &slice_u8[0..8] == idl::idl::program::client::args::ReportOrderFill::DISCRIMINATOR {
                if let Ok(instruction) =
                    idl::idl::program::client::args::ReportOrderFill::deserialize(&mut &slice_u8[8..])
                {
                    let accts = inst.accounts();
                    report_order_fill_instruction_list.push(ReportOrderFillInstruction {
                        trx_hash: transaction.id(),
                        source_chain_id: instruction.source_chain_id,
                        fill_report: Some(FillReport {
						order_id: instruction.fill_report.order_id.into_iter().map(|f| f as u64).collect(),amount_in_to_release: instruction.fill_report.amount_in_to_release as u64,amount_out_filled: instruction.fill_report.amount_out_filled as u64,origin_recipient: instruction.fill_report.origin_recipient.into_iter().map(|f| f as u64).collect(),token_in: instruction.fill_report.token_in.into_iter().map(|f| f as u64).collect(),
					}),
                        acct_relayer: accts[0].to_string(),
                        acct_portal_authority: accts[1].to_string(),
                        acct_global_account: accts[2].to_string(),
                        acct_order: accts[3].to_string(),
                        acct_token_in_mint: accts[4].to_string(),
                        acct_origin_recipient: accts[5].to_string(),
                        acct_recipient_token_in_ata: accts[6].to_string(),
                        acct_order_token_in_ata: accts[7].to_string(),
                        acct_token_in_program: accts[8].to_string(),
                        acct_event_authority: accts[11].to_string(),
                        acct_program: accts[12].to_string(),
                    });
                }
            }
            if &slice_u8[0..8] == idl::idl::program::client::args::SetNewAdmin::DISCRIMINATOR {
                if let Ok(instruction) =
                    idl::idl::program::client::args::SetNewAdmin::deserialize(&mut &slice_u8[8..])
                {
                    let accts = inst.accounts();
                    set_new_admin_instruction_list.push(SetNewAdminInstruction {
                        trx_hash: transaction.id(),
                        new_admin: instruction.new_admin.to_string(),
                        acct_admin: accts[0].to_string(),
                        acct_global_account: accts[1].to_string(),
                    });
                }
            }
            if &slice_u8[0..8] == idl::idl::program::client::args::SetPortalAuthority::DISCRIMINATOR {
                if let Ok(instruction) =
                    idl::idl::program::client::args::SetPortalAuthority::deserialize(&mut &slice_u8[8..])
                {
                    let accts = inst.accounts();
                    set_portal_authority_instruction_list.push(SetPortalAuthorityInstruction {
                        trx_hash: transaction.id(),
                        portal_authority: instruction.portal_authority.to_string(),
                        acct_admin: accts[0].to_string(),
                        acct_global_account: accts[1].to_string(),
                    });
                }
            }
            if &slice_u8[0..8] == idl::idl::program::client::args::Unpause::DISCRIMINATOR {
                if let Ok(instruction) =
                    idl::idl::program::client::args::Unpause::deserialize(&mut &slice_u8[8..])
                {
                    let accts = inst.accounts();
                    unpause_instruction_list.push(UnpauseInstruction {
                        trx_hash: transaction.id(),
                        acct_admin: accts[0].to_string(),
                        acct_global_account: accts[1].to_string(),
                    });
                }
            }
        });
    });

    Data {
        cancel_reported_event_list,
        destination_support_updated_event_list,
        fill_reported_event_list,
        order_cancelled_event_list,
        order_completed_event_list,
        order_filled_event_list,
        order_opened_event_list,
        refund_claimed_event_list,
        accept_admin_role_instruction_list,
        add_destination_instruction_list,
        cancel_foreign_order_instruction_list,
        cancel_native_order_instruction_list,
        clear_new_admin_instruction_list,
        close_order_token_account_instruction_list,
        fill_foreign_order_instruction_list,
        fill_native_order_instruction_list,
        idl_instruction_instruction_list,
        initialize_instruction_list,
        open_order_instruction_list,
        pause_instruction_list,
        remove_destination_instruction_list,
        report_order_cancel_instruction_list,
        report_order_fill_instruction_list,
        set_new_admin_instruction_list,
        set_portal_authority_instruction_list,
        unpause_instruction_list,
    }
}

fn map_option_cancel_report(
    value: Option<idl::idl::program::types::CancelReport>,
) -> Option<CancelReport> {
    match value {
        Some(cancel_report) => {
            return Some(CancelReport {
                order_id: cancel_report
                    .order_id
                    .into_iter()
                    .map(|f| f as u64)
                    .collect(),
                order_sender: cancel_report
                    .order_sender
                    .into_iter()
                    .map(|f| f as u64)
                    .collect(),
                token_in: cancel_report
                    .token_in
                    .into_iter()
                    .map(|f| f as u64)
                    .collect(),
                amount_in_to_refund: cancel_report.amount_in_to_refund as u64,
            })
        }
        None => {
            return None;
        }
    }
}

fn map_option_fill_params(
    value: Option<idl::idl::program::types::FillParams>,
) -> Option<FillParams> {
    match value {
        Some(fill_params) => {
            return Some(FillParams {
                amount_out_to_fill: fill_params.amount_out_to_fill,
                origin_recipient: fill_params
                    .origin_recipient
                    .into_iter()
                    .map(|f| f as u64)
                    .collect(),
            })
        }
        None => {
            return None;
        }
    }
}

fn map_option_fill_report(
    value: Option<idl::idl::program::types::FillReport>,
) -> Option<FillReport> {
    match value {
        Some(fill_report) => {
            return Some(FillReport {
                order_id: fill_report.order_id.into_iter().map(|f| f as u64).collect(),
                amount_in_to_release: fill_report.amount_in_to_release as u64,
                amount_out_filled: fill_report.amount_out_filled as u64,
                origin_recipient: fill_report
                    .origin_recipient
                    .into_iter()
                    .map(|f| f as u64)
                    .collect(),
                token_in: fill_report.token_in.into_iter().map(|f| f as u64).collect(),
            })
        }
        None => {
            return None;
        }
    }
}

fn map_option_foreign_order(
    value: Option<idl::idl::program::types::ForeignOrder>,
) -> Option<ForeignOrder> {
    match value {
        Some(foreign_order) => {
            return Some(ForeignOrder {
                status: map_enum_order_status(foreign_order.status),
                amount_in_released: foreign_order.amount_in_released as u64,
                amount_out_filled: foreign_order.amount_out_filled as u64,
                amount_in_refunded: foreign_order.amount_in_refunded as u64,
            })
        }
        None => {
            return None;
        }
    }
}

fn map_option_native_order(
    value: Option<idl::idl::program::types::NativeOrder>,
) -> Option<NativeOrder> {
    match value {
        Some(native_order) => {
            return Some(NativeOrder {
                status: map_enum_order_status(native_order.status),
                version: native_order.version as u64,
                sender: native_order.sender.to_string(),
                payer: native_order.payer.to_string(),
                nonce: native_order.nonce,
                dest_chain_id: native_order.dest_chain_id,
                created_at: native_order.created_at,
                fill_deadline: native_order.fill_deadline,
                token_in: native_order.token_in.to_string(),
                token_out: native_order
                    .token_out
                    .into_iter()
                    .map(|f| f as u64)
                    .collect(),
                amount_in: native_order.amount_in as u64,
                amount_out: native_order.amount_out as u64,
                recipient: native_order
                    .recipient
                    .into_iter()
                    .map(|f| f as u64)
                    .collect(),
                solver: native_order.solver.into_iter().map(|f| f as u64).collect(),
                amount_in_released: native_order.amount_in_released as u64,
                amount_out_filled: native_order.amount_out_filled as u64,
                amount_in_refunded: native_order.amount_in_refunded as u64,
            })
        }
        None => {
            return None;
        }
    }
}

fn map_option_order_data(value: Option<idl::idl::program::types::OrderData>) -> Option<OrderData> {
    match value {
        Some(order_data) => {
            return Some(OrderData {
                version: order_data.version as u64,
                sender: order_data.sender.into_iter().map(|f| f as u64).collect(),
                nonce: order_data.nonce,
                origin_chain_id: order_data.origin_chain_id,
                dest_chain_id: order_data.dest_chain_id,
                created_at: order_data.created_at,
                fill_deadline: order_data.fill_deadline,
                token_in: order_data.token_in.into_iter().map(|f| f as u64).collect(),
                token_out: order_data.token_out.into_iter().map(|f| f as u64).collect(),
                amount_in: order_data.amount_in as u64,
                amount_out: order_data.amount_out as u64,
                recipient: order_data.recipient.into_iter().map(|f| f as u64).collect(),
                solver: order_data.solver.into_iter().map(|f| f as u64).collect(),
            })
        }
        None => {
            return None;
        }
    }
}

fn map_option_order_params(
    value: Option<idl::idl::program::types::OrderParams>,
) -> Option<OrderParams> {
    match value {
        Some(order_params) => {
            return Some(OrderParams {
                dest_chain_id: order_params.dest_chain_id,
                created_at: order_params.created_at,
                fill_deadline: order_params.fill_deadline,
                token_out: order_params
                    .token_out
                    .into_iter()
                    .map(|f| f as u64)
                    .collect(),
                amount_in: order_params.amount_in,
                amount_out: order_params.amount_out as u64,
                recipient: order_params
                    .recipient
                    .into_iter()
                    .map(|f| f as u64)
                    .collect(),
                solver: order_params.solver.into_iter().map(|f| f as u64).collect(),
            })
        }
        None => {
            return None;
        }
    }
}

fn map_enum_order_status(value: idl::idl::program::types::OrderStatus) -> i32 {
    match value {
        idl::idl::program::types::OrderStatus::DoesNotExist => return 0,
        idl::idl::program::types::OrderStatus::Created => return 1,
        idl::idl::program::types::OrderStatus::Cancelled => return 2,
        idl::idl::program::types::OrderStatus::Completed => return 3,
        _ => 0,
    }
}
fn map_enum_order_type(value: idl::idl::program::types::OrderType) -> i32 {
    match value {
        idl::idl::program::types::OrderType::Native => return 0,
        idl::idl::program::types::OrderType::Foreign => return 1,
        _ => 0,
    }
}
