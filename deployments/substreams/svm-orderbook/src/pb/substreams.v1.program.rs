// @generated
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Data {
    #[prost(message, repeated, tag="1")]
    pub cancel_reported_event_list: ::prost::alloc::vec::Vec<CancelReportedEvent>,
    #[prost(message, repeated, tag="2")]
    pub destination_support_updated_event_list: ::prost::alloc::vec::Vec<DestinationSupportUpdatedEvent>,
    #[prost(message, repeated, tag="3")]
    pub fill_reported_event_list: ::prost::alloc::vec::Vec<FillReportedEvent>,
    #[prost(message, repeated, tag="4")]
    pub order_cancelled_event_list: ::prost::alloc::vec::Vec<OrderCancelledEvent>,
    #[prost(message, repeated, tag="5")]
    pub order_completed_event_list: ::prost::alloc::vec::Vec<OrderCompletedEvent>,
    #[prost(message, repeated, tag="6")]
    pub order_filled_event_list: ::prost::alloc::vec::Vec<OrderFilledEvent>,
    #[prost(message, repeated, tag="7")]
    pub order_opened_event_list: ::prost::alloc::vec::Vec<OrderOpenedEvent>,
    #[prost(message, repeated, tag="8")]
    pub refund_claimed_event_list: ::prost::alloc::vec::Vec<RefundClaimedEvent>,
    #[prost(message, repeated, tag="9")]
    pub accept_admin_role_instruction_list: ::prost::alloc::vec::Vec<AcceptAdminRoleInstruction>,
    #[prost(message, repeated, tag="10")]
    pub add_destination_instruction_list: ::prost::alloc::vec::Vec<AddDestinationInstruction>,
    #[prost(message, repeated, tag="11")]
    pub cancel_foreign_order_instruction_list: ::prost::alloc::vec::Vec<CancelForeignOrderInstruction>,
    #[prost(message, repeated, tag="12")]
    pub cancel_native_order_instruction_list: ::prost::alloc::vec::Vec<CancelNativeOrderInstruction>,
    #[prost(message, repeated, tag="13")]
    pub clear_new_admin_instruction_list: ::prost::alloc::vec::Vec<ClearNewAdminInstruction>,
    #[prost(message, repeated, tag="14")]
    pub close_order_token_account_instruction_list: ::prost::alloc::vec::Vec<CloseOrderTokenAccountInstruction>,
    #[prost(message, repeated, tag="15")]
    pub fill_foreign_order_instruction_list: ::prost::alloc::vec::Vec<FillForeignOrderInstruction>,
    #[prost(message, repeated, tag="16")]
    pub fill_native_order_instruction_list: ::prost::alloc::vec::Vec<FillNativeOrderInstruction>,
    #[prost(message, repeated, tag="17")]
    pub idl_instruction_instruction_list: ::prost::alloc::vec::Vec<IdlInstructionInstruction>,
    #[prost(message, repeated, tag="18")]
    pub initialize_instruction_list: ::prost::alloc::vec::Vec<InitializeInstruction>,
    #[prost(message, repeated, tag="19")]
    pub open_order_instruction_list: ::prost::alloc::vec::Vec<OpenOrderInstruction>,
    #[prost(message, repeated, tag="20")]
    pub pause_instruction_list: ::prost::alloc::vec::Vec<PauseInstruction>,
    #[prost(message, repeated, tag="21")]
    pub remove_destination_instruction_list: ::prost::alloc::vec::Vec<RemoveDestinationInstruction>,
    #[prost(message, repeated, tag="22")]
    pub report_order_cancel_instruction_list: ::prost::alloc::vec::Vec<ReportOrderCancelInstruction>,
    #[prost(message, repeated, tag="23")]
    pub report_order_fill_instruction_list: ::prost::alloc::vec::Vec<ReportOrderFillInstruction>,
    #[prost(message, repeated, tag="24")]
    pub set_new_admin_instruction_list: ::prost::alloc::vec::Vec<SetNewAdminInstruction>,
    #[prost(message, repeated, tag="25")]
    pub set_portal_authority_instruction_list: ::prost::alloc::vec::Vec<SetPortalAuthorityInstruction>,
    #[prost(message, repeated, tag="26")]
    pub unpause_instruction_list: ::prost::alloc::vec::Vec<UnpauseInstruction>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CancelReportedEvent {
    #[prost(string, tag="1")]
    pub trx_hash: ::prost::alloc::string::String,
    #[prost(uint64, repeated, tag="2")]
    pub order_id: ::prost::alloc::vec::Vec<u64>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DestinationSupportUpdatedEvent {
    #[prost(string, tag="1")]
    pub trx_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub dest_chain_id: u32,
    #[prost(bool, tag="3")]
    pub is_supported: bool,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FillReportedEvent {
    #[prost(string, tag="1")]
    pub trx_hash: ::prost::alloc::string::String,
    #[prost(uint64, repeated, tag="2")]
    pub order_id: ::prost::alloc::vec::Vec<u64>,
    #[prost(uint64, tag="3")]
    pub amount_in_to_release: u64,
    #[prost(uint64, tag="4")]
    pub amount_out_filled: u64,
    #[prost(uint64, repeated, tag="5")]
    pub origin_recipient: ::prost::alloc::vec::Vec<u64>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OrderCancelledEvent {
    #[prost(string, tag="1")]
    pub trx_hash: ::prost::alloc::string::String,
    #[prost(uint64, repeated, tag="2")]
    pub order_id: ::prost::alloc::vec::Vec<u64>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OrderCompletedEvent {
    #[prost(string, tag="1")]
    pub trx_hash: ::prost::alloc::string::String,
    #[prost(uint64, repeated, tag="2")]
    pub order_id: ::prost::alloc::vec::Vec<u64>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OrderFilledEvent {
    #[prost(string, tag="1")]
    pub trx_hash: ::prost::alloc::string::String,
    #[prost(uint64, repeated, tag="2")]
    pub order_id: ::prost::alloc::vec::Vec<u64>,
    #[prost(string, tag="3")]
    pub solver: ::prost::alloc::string::String,
    #[prost(uint64, tag="4")]
    pub amount_in_to_release: u64,
    #[prost(uint64, tag="5")]
    pub amount_out_filled: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OrderOpenedEvent {
    #[prost(string, tag="1")]
    pub trx_hash: ::prost::alloc::string::String,
    #[prost(uint64, repeated, tag="2")]
    pub order_id: ::prost::alloc::vec::Vec<u64>,
    #[prost(string, tag="3")]
    pub sender: ::prost::alloc::string::String,
    #[prost(string, tag="4")]
    pub token_in: ::prost::alloc::string::String,
    #[prost(uint64, tag="5")]
    pub amount_in: u64,
    #[prost(uint32, tag="6")]
    pub dest_chain_id: u32,
    #[prost(uint64, repeated, tag="7")]
    pub token_out: ::prost::alloc::vec::Vec<u64>,
    #[prost(uint64, tag="8")]
    pub amount_out: u64,
    #[prost(uint64, repeated, tag="9")]
    pub solver: ::prost::alloc::vec::Vec<u64>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RefundClaimedEvent {
    #[prost(string, tag="1")]
    pub trx_hash: ::prost::alloc::string::String,
    #[prost(uint64, repeated, tag="2")]
    pub order_id: ::prost::alloc::vec::Vec<u64>,
    #[prost(string, tag="3")]
    pub sender: ::prost::alloc::string::String,
    #[prost(uint64, tag="4")]
    pub amount: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AcceptAdminRoleInstruction {
    #[prost(string, tag="1")]
    pub trx_hash: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub acct_new_admin: ::prost::alloc::string::String,
    #[prost(string, tag="3")]
    pub acct_global_account: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AddDestinationInstruction {
    #[prost(string, tag="1")]
    pub trx_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub dest_chain_id: u32,
    #[prost(string, tag="3")]
    pub acct_admin: ::prost::alloc::string::String,
    #[prost(string, tag="4")]
    pub acct_global_account: ::prost::alloc::string::String,
    #[prost(string, tag="5")]
    pub acct_destination_account: ::prost::alloc::string::String,
    #[prost(string, tag="6")]
    pub acct_event_authority: ::prost::alloc::string::String,
    #[prost(string, tag="7")]
    pub acct_program: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CancelForeignOrderInstruction {
    #[prost(string, tag="1")]
    pub trx_hash: ::prost::alloc::string::String,
    #[prost(uint64, repeated, tag="2")]
    pub order_id: ::prost::alloc::vec::Vec<u64>,
    #[prost(message, optional, tag="3")]
    pub order_data: ::core::option::Option<OrderData>,
    #[prost(string, tag="4")]
    pub acct_signer: ::prost::alloc::string::String,
    #[prost(string, tag="5")]
    pub acct_global_account: ::prost::alloc::string::String,
    #[prost(string, tag="6")]
    pub acct_order: ::prost::alloc::string::String,
    #[prost(string, tag="7")]
    pub acct_portal_global: ::prost::alloc::string::String,
    #[prost(string, tag="8")]
    pub acct_portal_authority: ::prost::alloc::string::String,
    #[prost(string, tag="9")]
    pub acct_bridge_adapter: ::prost::alloc::string::String,
    #[prost(string, tag="10")]
    pub acct_event_authority: ::prost::alloc::string::String,
    #[prost(string, tag="11")]
    pub acct_program: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CancelNativeOrderInstruction {
    #[prost(string, tag="1")]
    pub trx_hash: ::prost::alloc::string::String,
    #[prost(uint64, repeated, tag="2")]
    pub order_id: ::prost::alloc::vec::Vec<u64>,
    #[prost(string, tag="3")]
    pub acct_signer: ::prost::alloc::string::String,
    #[prost(string, tag="4")]
    pub acct_sender: ::prost::alloc::string::String,
    #[prost(string, tag="5")]
    pub acct_global_account: ::prost::alloc::string::String,
    #[prost(string, tag="6")]
    pub acct_order: ::prost::alloc::string::String,
    #[prost(string, tag="7")]
    pub acct_token_in_mint: ::prost::alloc::string::String,
    #[prost(string, tag="8")]
    pub acct_sender_token_in_ata: ::prost::alloc::string::String,
    #[prost(string, tag="9")]
    pub acct_order_token_in_ata: ::prost::alloc::string::String,
    #[prost(string, tag="10")]
    pub acct_token_in_program: ::prost::alloc::string::String,
    #[prost(string, tag="11")]
    pub acct_event_authority: ::prost::alloc::string::String,
    #[prost(string, tag="12")]
    pub acct_program: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClearNewAdminInstruction {
    #[prost(string, tag="1")]
    pub trx_hash: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub acct_admin: ::prost::alloc::string::String,
    #[prost(string, tag="3")]
    pub acct_global_account: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CloseOrderTokenAccountInstruction {
    #[prost(string, tag="1")]
    pub trx_hash: ::prost::alloc::string::String,
    #[prost(uint64, repeated, tag="2")]
    pub order_id: ::prost::alloc::vec::Vec<u64>,
    #[prost(string, tag="3")]
    pub acct_payer: ::prost::alloc::string::String,
    #[prost(string, tag="4")]
    pub acct_sender: ::prost::alloc::string::String,
    #[prost(string, tag="5")]
    pub acct_order: ::prost::alloc::string::String,
    #[prost(string, tag="6")]
    pub acct_token_in_mint: ::prost::alloc::string::String,
    #[prost(string, tag="7")]
    pub acct_order_token_in_ata: ::prost::alloc::string::String,
    #[prost(string, tag="8")]
    pub acct_recipient_token_account: ::prost::alloc::string::String,
    #[prost(string, tag="9")]
    pub acct_token_in_program: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FillForeignOrderInstruction {
    #[prost(string, tag="1")]
    pub trx_hash: ::prost::alloc::string::String,
    #[prost(uint64, repeated, tag="2")]
    pub order_id: ::prost::alloc::vec::Vec<u64>,
    #[prost(message, optional, tag="3")]
    pub order_data: ::core::option::Option<OrderData>,
    #[prost(message, optional, tag="4")]
    pub fill_params: ::core::option::Option<FillParams>,
    #[prost(string, tag="5")]
    pub acct_solver: ::prost::alloc::string::String,
    #[prost(string, tag="6")]
    pub acct_global_account: ::prost::alloc::string::String,
    #[prost(string, tag="7")]
    pub acct_token_out_mint: ::prost::alloc::string::String,
    #[prost(string, tag="8")]
    pub acct_solver_token_out_account: ::prost::alloc::string::String,
    #[prost(string, tag="9")]
    pub acct_recipient: ::prost::alloc::string::String,
    #[prost(string, tag="10")]
    pub acct_recipient_token_out_ata: ::prost::alloc::string::String,
    #[prost(string, tag="11")]
    pub acct_token_out_program: ::prost::alloc::string::String,
    #[prost(string, tag="12")]
    pub acct_order: ::prost::alloc::string::String,
    #[prost(string, tag="13")]
    pub acct_portal_global: ::prost::alloc::string::String,
    #[prost(string, tag="14")]
    pub acct_portal_authority: ::prost::alloc::string::String,
    #[prost(string, tag="15")]
    pub acct_bridge_adapter: ::prost::alloc::string::String,
    #[prost(string, tag="16")]
    pub acct_event_authority: ::prost::alloc::string::String,
    #[prost(string, tag="17")]
    pub acct_program: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FillNativeOrderInstruction {
    #[prost(string, tag="1")]
    pub trx_hash: ::prost::alloc::string::String,
    #[prost(uint64, repeated, tag="2")]
    pub order_id: ::prost::alloc::vec::Vec<u64>,
    #[prost(message, optional, tag="3")]
    pub order_data: ::core::option::Option<OrderData>,
    #[prost(message, optional, tag="4")]
    pub fill_params: ::core::option::Option<FillParams>,
    #[prost(string, tag="5")]
    pub acct_solver: ::prost::alloc::string::String,
    #[prost(string, tag="6")]
    pub acct_global_account: ::prost::alloc::string::String,
    #[prost(string, tag="7")]
    pub acct_token_out_mint: ::prost::alloc::string::String,
    #[prost(string, tag="8")]
    pub acct_solver_token_out_account: ::prost::alloc::string::String,
    #[prost(string, tag="9")]
    pub acct_recipient: ::prost::alloc::string::String,
    #[prost(string, tag="10")]
    pub acct_recipient_token_out_ata: ::prost::alloc::string::String,
    #[prost(string, tag="11")]
    pub acct_token_out_program: ::prost::alloc::string::String,
    #[prost(string, tag="12")]
    pub acct_order: ::prost::alloc::string::String,
    #[prost(string, tag="13")]
    pub acct_token_in_mint: ::prost::alloc::string::String,
    #[prost(string, tag="14")]
    pub acct_solver_token_in_account: ::prost::alloc::string::String,
    #[prost(string, tag="15")]
    pub acct_order_token_in_ata: ::prost::alloc::string::String,
    #[prost(string, tag="16")]
    pub acct_token_in_program: ::prost::alloc::string::String,
    #[prost(string, tag="17")]
    pub acct_event_authority: ::prost::alloc::string::String,
    #[prost(string, tag="18")]
    pub acct_program: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IdlInstructionInstruction {
    #[prost(string, tag="1")]
    pub trx_hash: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub foreign: ::core::option::Option<ForeignOrder>,
    #[prost(message, optional, tag="3")]
    pub native: ::core::option::Option<NativeOrder>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InitializeInstruction {
    #[prost(string, tag="1")]
    pub trx_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub chain_id: u32,
    #[prost(string, tag="3")]
    pub portal_authority: ::prost::alloc::string::String,
    #[prost(string, tag="4")]
    pub acct_admin: ::prost::alloc::string::String,
    #[prost(string, tag="5")]
    pub acct_global_account: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OpenOrderInstruction {
    #[prost(string, tag="1")]
    pub trx_hash: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub params: ::core::option::Option<OrderParams>,
    #[prost(string, tag="3")]
    pub acct_payer: ::prost::alloc::string::String,
    #[prost(string, tag="4")]
    pub acct_token_authority: ::prost::alloc::string::String,
    #[prost(string, tag="5")]
    pub acct_global_account: ::prost::alloc::string::String,
    #[prost(string, tag="6")]
    pub acct_destination_account: ::prost::alloc::string::String,
    #[prost(string, tag="7")]
    pub acct_token_in_mint: ::prost::alloc::string::String,
    #[prost(string, tag="8")]
    pub acct_sender_token_in_account: ::prost::alloc::string::String,
    #[prost(string, tag="9")]
    pub acct_sender_nonce_account: ::prost::alloc::string::String,
    #[prost(string, tag="10")]
    pub acct_order: ::prost::alloc::string::String,
    #[prost(string, tag="11")]
    pub acct_order_token_in_ata: ::prost::alloc::string::String,
    #[prost(string, tag="12")]
    pub acct_token_in_program: ::prost::alloc::string::String,
    #[prost(string, tag="13")]
    pub acct_event_authority: ::prost::alloc::string::String,
    #[prost(string, tag="14")]
    pub acct_program: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PauseInstruction {
    #[prost(string, tag="1")]
    pub trx_hash: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub acct_admin: ::prost::alloc::string::String,
    #[prost(string, tag="3")]
    pub acct_global_account: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RemoveDestinationInstruction {
    #[prost(string, tag="1")]
    pub trx_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub dest_chain_id: u32,
    #[prost(string, tag="3")]
    pub acct_admin: ::prost::alloc::string::String,
    #[prost(string, tag="4")]
    pub acct_global_account: ::prost::alloc::string::String,
    #[prost(string, tag="5")]
    pub acct_destination_account: ::prost::alloc::string::String,
    #[prost(string, tag="6")]
    pub acct_event_authority: ::prost::alloc::string::String,
    #[prost(string, tag="7")]
    pub acct_program: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReportOrderCancelInstruction {
    #[prost(string, tag="1")]
    pub trx_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub source_chain_id: u32,
    #[prost(message, optional, tag="3")]
    pub cancel_report: ::core::option::Option<CancelReport>,
    #[prost(string, tag="4")]
    pub acct_relayer: ::prost::alloc::string::String,
    #[prost(string, tag="5")]
    pub acct_portal_authority: ::prost::alloc::string::String,
    #[prost(string, tag="6")]
    pub acct_global_account: ::prost::alloc::string::String,
    #[prost(string, tag="7")]
    pub acct_order: ::prost::alloc::string::String,
    #[prost(string, tag="8")]
    pub acct_token_in_mint: ::prost::alloc::string::String,
    #[prost(string, tag="9")]
    pub acct_order_sender: ::prost::alloc::string::String,
    #[prost(string, tag="10")]
    pub acct_sender_token_in_ata: ::prost::alloc::string::String,
    #[prost(string, tag="11")]
    pub acct_order_token_in_ata: ::prost::alloc::string::String,
    #[prost(string, tag="12")]
    pub acct_token_in_program: ::prost::alloc::string::String,
    #[prost(string, tag="13")]
    pub acct_event_authority: ::prost::alloc::string::String,
    #[prost(string, tag="14")]
    pub acct_program: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReportOrderFillInstruction {
    #[prost(string, tag="1")]
    pub trx_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub source_chain_id: u32,
    #[prost(message, optional, tag="3")]
    pub fill_report: ::core::option::Option<FillReport>,
    #[prost(string, tag="4")]
    pub acct_relayer: ::prost::alloc::string::String,
    #[prost(string, tag="5")]
    pub acct_portal_authority: ::prost::alloc::string::String,
    #[prost(string, tag="6")]
    pub acct_global_account: ::prost::alloc::string::String,
    #[prost(string, tag="7")]
    pub acct_order: ::prost::alloc::string::String,
    #[prost(string, tag="8")]
    pub acct_token_in_mint: ::prost::alloc::string::String,
    #[prost(string, tag="9")]
    pub acct_origin_recipient: ::prost::alloc::string::String,
    #[prost(string, tag="10")]
    pub acct_recipient_token_in_ata: ::prost::alloc::string::String,
    #[prost(string, tag="11")]
    pub acct_order_token_in_ata: ::prost::alloc::string::String,
    #[prost(string, tag="12")]
    pub acct_token_in_program: ::prost::alloc::string::String,
    #[prost(string, tag="13")]
    pub acct_event_authority: ::prost::alloc::string::String,
    #[prost(string, tag="14")]
    pub acct_program: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SetNewAdminInstruction {
    #[prost(string, tag="1")]
    pub trx_hash: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub new_admin: ::prost::alloc::string::String,
    #[prost(string, tag="3")]
    pub acct_admin: ::prost::alloc::string::String,
    #[prost(string, tag="4")]
    pub acct_global_account: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SetPortalAuthorityInstruction {
    #[prost(string, tag="1")]
    pub trx_hash: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub portal_authority: ::prost::alloc::string::String,
    #[prost(string, tag="3")]
    pub acct_admin: ::prost::alloc::string::String,
    #[prost(string, tag="4")]
    pub acct_global_account: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UnpauseInstruction {
    #[prost(string, tag="1")]
    pub trx_hash: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub acct_admin: ::prost::alloc::string::String,
    #[prost(string, tag="3")]
    pub acct_global_account: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CancelReport {
    #[prost(uint64, repeated, tag="1")]
    pub order_id: ::prost::alloc::vec::Vec<u64>,
    #[prost(uint64, repeated, tag="2")]
    pub order_sender: ::prost::alloc::vec::Vec<u64>,
    #[prost(uint64, repeated, tag="3")]
    pub token_in: ::prost::alloc::vec::Vec<u64>,
    #[prost(uint64, tag="4")]
    pub amount_in_to_refund: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FillParams {
    #[prost(uint64, tag="1")]
    pub amount_out_to_fill: u64,
    #[prost(uint64, repeated, tag="2")]
    pub origin_recipient: ::prost::alloc::vec::Vec<u64>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FillReport {
    #[prost(uint64, repeated, tag="1")]
    pub order_id: ::prost::alloc::vec::Vec<u64>,
    #[prost(uint64, tag="2")]
    pub amount_in_to_release: u64,
    #[prost(uint64, tag="3")]
    pub amount_out_filled: u64,
    #[prost(uint64, repeated, tag="4")]
    pub origin_recipient: ::prost::alloc::vec::Vec<u64>,
    #[prost(uint64, repeated, tag="5")]
    pub token_in: ::prost::alloc::vec::Vec<u64>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ForeignOrder {
    #[prost(enumeration="OrderStatusEnum", tag="1")]
    pub status: i32,
    #[prost(uint64, tag="2")]
    pub amount_in_released: u64,
    #[prost(uint64, tag="3")]
    pub amount_out_filled: u64,
    #[prost(uint64, tag="4")]
    pub amount_in_refunded: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NativeOrder {
    #[prost(enumeration="OrderStatusEnum", tag="1")]
    pub status: i32,
    #[prost(uint64, tag="2")]
    pub version: u64,
    #[prost(string, tag="3")]
    pub sender: ::prost::alloc::string::String,
    #[prost(string, tag="4")]
    pub payer: ::prost::alloc::string::String,
    #[prost(uint64, tag="5")]
    pub nonce: u64,
    #[prost(uint32, tag="6")]
    pub dest_chain_id: u32,
    #[prost(uint64, tag="7")]
    pub created_at: u64,
    #[prost(uint64, tag="8")]
    pub fill_deadline: u64,
    #[prost(string, tag="9")]
    pub token_in: ::prost::alloc::string::String,
    #[prost(uint64, repeated, tag="10")]
    pub token_out: ::prost::alloc::vec::Vec<u64>,
    #[prost(uint64, tag="11")]
    pub amount_in: u64,
    #[prost(uint64, tag="12")]
    pub amount_out: u64,
    #[prost(uint64, repeated, tag="13")]
    pub recipient: ::prost::alloc::vec::Vec<u64>,
    #[prost(uint64, repeated, tag="14")]
    pub solver: ::prost::alloc::vec::Vec<u64>,
    #[prost(uint64, tag="15")]
    pub amount_in_released: u64,
    #[prost(uint64, tag="16")]
    pub amount_out_filled: u64,
    #[prost(uint64, tag="17")]
    pub amount_in_refunded: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OrderData {
    #[prost(uint64, tag="1")]
    pub version: u64,
    #[prost(uint64, repeated, tag="2")]
    pub sender: ::prost::alloc::vec::Vec<u64>,
    #[prost(uint64, tag="3")]
    pub nonce: u64,
    #[prost(uint32, tag="4")]
    pub origin_chain_id: u32,
    #[prost(uint32, tag="5")]
    pub dest_chain_id: u32,
    #[prost(uint64, tag="6")]
    pub created_at: u64,
    #[prost(uint64, tag="7")]
    pub fill_deadline: u64,
    #[prost(uint64, repeated, tag="8")]
    pub token_in: ::prost::alloc::vec::Vec<u64>,
    #[prost(uint64, repeated, tag="9")]
    pub token_out: ::prost::alloc::vec::Vec<u64>,
    #[prost(uint64, tag="10")]
    pub amount_in: u64,
    #[prost(uint64, tag="11")]
    pub amount_out: u64,
    #[prost(uint64, repeated, tag="12")]
    pub recipient: ::prost::alloc::vec::Vec<u64>,
    #[prost(uint64, repeated, tag="13")]
    pub solver: ::prost::alloc::vec::Vec<u64>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OrderParams {
    #[prost(uint32, tag="1")]
    pub dest_chain_id: u32,
    #[prost(uint64, tag="2")]
    pub created_at: u64,
    #[prost(uint64, tag="3")]
    pub fill_deadline: u64,
    #[prost(uint64, repeated, tag="4")]
    pub token_out: ::prost::alloc::vec::Vec<u64>,
    #[prost(uint64, tag="5")]
    pub amount_in: u64,
    #[prost(uint64, tag="6")]
    pub amount_out: u64,
    #[prost(uint64, repeated, tag="7")]
    pub recipient: ::prost::alloc::vec::Vec<u64>,
    #[prost(uint64, repeated, tag="8")]
    pub solver: ::prost::alloc::vec::Vec<u64>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum OrderStatusEnum {
    OrderStatusDoesNotExist = 0,
    OrderStatusCreated = 1,
    OrderStatusCancelled = 2,
    OrderStatusCompleted = 3,
}
impl OrderStatusEnum {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            OrderStatusEnum::OrderStatusDoesNotExist => "ORDER_STATUS_DOES_NOT_EXIST",
            OrderStatusEnum::OrderStatusCreated => "ORDER_STATUS_CREATED",
            OrderStatusEnum::OrderStatusCancelled => "ORDER_STATUS_CANCELLED",
            OrderStatusEnum::OrderStatusCompleted => "ORDER_STATUS_COMPLETED",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "ORDER_STATUS_DOES_NOT_EXIST" => Some(Self::OrderStatusDoesNotExist),
            "ORDER_STATUS_CREATED" => Some(Self::OrderStatusCreated),
            "ORDER_STATUS_CANCELLED" => Some(Self::OrderStatusCancelled),
            "ORDER_STATUS_COMPLETED" => Some(Self::OrderStatusCompleted),
            _ => None,
        }
    }
}
// @@protoc_insertion_point(module)
