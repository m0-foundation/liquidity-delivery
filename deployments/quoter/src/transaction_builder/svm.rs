use anchor_lang::AnchorSerialize;
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine};
use m0_portal_common::{
    build_relay_instruction, get_wormhole_chain_id, wormhole, WormholeRemainingAccounts,
};
use order_book::{compute_order_id, OrderData, OrderParams};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    address_lookup_table::AddressLookupTableAccount,
    instruction::{AccountMeta, Instruction},
    message::{v0, Message, VersionedMessage},
    pubkey::Pubkey,
    system_program,
    transaction::{Transaction, VersionedTransaction},
};
use spl_associated_token_account::get_associated_token_address_with_program_id;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use super::error::TransactionBuilderError;
use super::{CancelOrderInput, OpenOrderInput, TransactionResult};

// Order book constants
const DEFAULT_ORDER_BOOK_PROGRAM_ID: &str = "MzLoYnJ6sF6eeejs4vV95TNmXqS3W4cAtLGKkjT4ZrK";
const VERSION: u16 = 1;

// Instruction discriminators (Anchor-generated)
const OPEN_ORDER_DISCRIMINATOR: [u8; 8] = [206, 88, 88, 143, 38, 136, 50, 224];
const CANCEL_NATIVE_ORDER_DISCRIMINATOR: [u8; 8] = [161, 5, 252, 99, 194, 215, 19, 84];
const CANCEL_FOREIGN_ORDER_DISCRIMINATOR: [u8; 8] = [64, 42, 40, 137, 244, 90, 185, 107];

// PDA seeds
const GLOBAL_SEED: &[u8] = b"global";
const NONCE_SEED: &[u8] = b"nonce";
const DESTINATION_SEED: &[u8] = b"destination";
const ORDER_SEED: &[u8] = b"order";
const EVENT_AUTHORITY_SEED: &[u8] = b"__event_authority";

// Portal/bridge constants
const PORTAL_PROGRAM_ID: &str = "MzBrgc8yXBj4P16GTkcSyDZkEQZB9qDqf3fh9bByJce";
const BRIDGE_ADAPTER: &str = "mzp1q2j5Hr1QuLC3KFBCAUz5aUckT6qyuZKZ3WJnMmY";
const LOOKUP_TABLE: &str = "9Ez9PMwWdZ35uFBXZH6QEzmqNyAqPeW8sLdVbJWyo8nT";
const LOOKUP_TABLE_META_SIZE: usize = 56;

type Result<T> = std::result::Result<T, TransactionBuilderError>;

/// Extension trait for converting errors to TransactionBuilderError
trait IntoTxError<T> {
    fn addr_err(self) -> Result<T>;
    fn rpc_err(self) -> Result<T>;
    fn ser_err(self) -> Result<T>;
}

impl<T, E: ToString> IntoTxError<T> for std::result::Result<T, E> {
    fn addr_err(self) -> Result<T> {
        self.map_err(|e| TransactionBuilderError::InvalidAddress(e.to_string()))
    }
    fn rpc_err(self) -> Result<T> {
        self.map_err(|e| TransactionBuilderError::RpcError(e.to_string()))
    }
    fn ser_err(self) -> Result<T> {
        self.map_err(|e| TransactionBuilderError::SerializationError(e.to_string()))
    }
}

/// Helper to parse a pubkey from string
fn parse_pubkey(s: &str) -> Result<Pubkey> {
    Pubkey::from_str(s).addr_err()
}

/// Helper to derive a PDA
fn find_pda(seeds: &[&[u8]], program_id: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(seeds, program_id).0
}

/// Helper to get current unix timestamp with offset
fn current_timestamp_with_offset(offset_secs: u64) -> Result<u64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ser_err()?
        .as_secs()
        + offset_secs)
}

/// Serialize instruction data with discriminator
fn serialize_instruction<T: AnchorSerialize>(
    discriminator: &[u8; 8],
    params: &T,
) -> Result<Vec<u8>> {
    let mut data = Vec::with_capacity(256);
    data.extend_from_slice(discriminator);
    params.serialize(&mut data).ser_err()?;
    Ok(data)
}

pub struct SvmTransactionBuilder {
    rpc_client: RpcClient,
    program_id: Pubkey,
    chain_id: u32,
}

impl SvmTransactionBuilder {
    pub fn new(rpc_url: String, program_id: Option<String>, chain_id: u32) -> Result<Self> {
        let program_id = program_id
            .as_deref()
            .unwrap_or(DEFAULT_ORDER_BOOK_PROGRAM_ID);
        let program_id = parse_pubkey(program_id)?;

        Ok(Self {
            rpc_client: RpcClient::new(rpc_url),
            program_id,
            chain_id,
        })
    }

    // ─────────────────────────────────────────────────────────────────────────
    // PDA derivation helpers
    // ─────────────────────────────────────────────────────────────────────────

    fn global_pda(&self) -> Pubkey {
        find_pda(&[GLOBAL_SEED], &self.program_id)
    }

    fn nonce_pda(&self, sender: &Pubkey) -> Pubkey {
        find_pda(&[NONCE_SEED, sender.as_ref()], &self.program_id)
    }

    fn order_pda(&self, order_id: &[u8; 32]) -> Pubkey {
        find_pda(&[ORDER_SEED, order_id], &self.program_id)
    }

    fn event_authority_pda(&self) -> Pubkey {
        find_pda(&[EVENT_AUTHORITY_SEED], &self.program_id)
    }

    fn destination_pda(&self, dest_chain_id: u32) -> Pubkey {
        find_pda(
            &[DESTINATION_SEED, &dest_chain_id.to_be_bytes()],
            &self.program_id,
        )
    }

    // ─────────────────────────────────────────────────────────────────────────
    // RPC helpers
    // ─────────────────────────────────────────────────────────────────────────

    pub async fn get_sender_nonce(&self, sender: &Pubkey) -> Result<u64> {
        let nonce_pda = self.nonce_pda(sender);

        match self.rpc_client.get_account(&nonce_pda).await {
            Ok(account) if account.data.len() >= 17 => {
                let nonce_bytes: [u8; 8] = account.data[9..17]
                    .try_into()
                    .map_err(|_| TransactionBuilderError::AccountParseError)?;
                Ok(u64::from_le_bytes(nonce_bytes))
            }
            Ok(_) => Err(TransactionBuilderError::AccountParseError),
            Err(_) => Ok(0), // Account doesn't exist yet
        }
    }

    async fn get_blockhash(&self) -> Result<solana_sdk::hash::Hash> {
        self.rpc_client.get_latest_blockhash().await.rpc_err()
    }

    async fn get_lookup_table(&self, address: &Pubkey) -> Result<AddressLookupTableAccount> {
        let account = self.rpc_client.get_account(address).await.rpc_err()?;

        if account.data.len() < LOOKUP_TABLE_META_SIZE {
            return Err(TransactionBuilderError::AccountParseError);
        }

        let addresses_data = &account.data[LOOKUP_TABLE_META_SIZE..];
        if addresses_data.len() % 32 != 0 {
            return Err(TransactionBuilderError::AccountParseError);
        }

        let addresses = addresses_data
            .chunks_exact(32)
            .map(|chunk| Pubkey::new_from_array(chunk.try_into().unwrap()))
            .collect();

        Ok(AddressLookupTableAccount {
            key: *address,
            addresses,
        })
    }

    async fn get_mint_token_program(&self, mint: &Pubkey) -> Result<Pubkey> {
        let account = self.rpc_client.get_account(mint).await.rpc_err()?;

        match account.owner {
            id if id == spl_token::ID => Ok(spl_token::ID),
            id if id == spl_token_2022::ID => Ok(spl_token_2022::ID),
            _ => Err(TransactionBuilderError::InvalidAddress(format!(
                "Mint {} is not owned by a known token program",
                mint
            ))),
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Open Order Transaction
    // ─────────────────────────────────────────────────────────────────────────

    pub async fn build_open_order_transaction(
        &self,
        input: &OpenOrderInput,
    ) -> Result<TransactionResult> {
        let sender = parse_pubkey(&input.sender_address)?;
        let token_in_mint = parse_pubkey(&input.token_in)?;
        let token_out = parse_bytes32(&input.token_out)?;

        let token_program = self.get_mint_token_program(&token_in_mint).await?;
        let nonce = self.get_sender_nonce(&sender).await?;
        let blockhash = self.get_blockhash().await?;
        let created_at = current_timestamp_with_offset(30)?;

        // Compute order ID
        let order_data = OrderData {
            version: VERSION,
            sender: sender.to_bytes(),
            nonce,
            origin_chain_id: self.chain_id,
            dest_chain_id: input.dest_chain_id,
            created_at,
            fill_deadline: input.fill_deadline,
            token_in: token_in_mint.to_bytes(),
            token_out,
            amount_in: input.amount_in as u128,
            amount_out: input.amount_out,
            recipient: input.recipient,
            solver: input.solver,
        };
        let order_id = compute_order_id(&order_data);

        // Derive PDAs
        let order_pda = self.order_pda(&order_id);
        let destination = if input.dest_chain_id != self.chain_id {
            self.destination_pda(input.dest_chain_id)
        } else {
            self.program_id // Placeholder for None
        };

        // Token accounts
        let sender_ata =
            get_associated_token_address_with_program_id(&sender, &token_in_mint, &token_program);
        let order_ata = get_associated_token_address_with_program_id(
            &order_pda,
            &token_in_mint,
            &token_program,
        );

        // Build instruction
        let order_params = OrderParams {
            dest_chain_id: input.dest_chain_id,
            created_at,
            fill_deadline: input.fill_deadline,
            token_out,
            amount_in: input.amount_in,
            amount_out: input.amount_out,
            recipient: input.recipient,
            solver: input.solver,
        };

        let instruction = Instruction {
            program_id: self.program_id,
            accounts: vec![
                AccountMeta::new(sender, true),                      // payer/signer
                AccountMeta::new_readonly(self.program_id, false),   // token_authority (None)
                AccountMeta::new_readonly(self.global_pda(), false), // global
                AccountMeta::new_readonly(destination, false),       // destination
                AccountMeta::new_readonly(token_in_mint, false),     // token_in_mint
                AccountMeta::new(sender_ata, false),                 // sender_token_in
                AccountMeta::new(self.nonce_pda(&sender), false),    // nonce
                AccountMeta::new(order_pda, false),                  // order
                AccountMeta::new(order_ata, false),                  // order_token_in
                AccountMeta::new_readonly(token_program, false),     // token_program
                AccountMeta::new_readonly(spl_associated_token_account::ID, false), // ata_program
                AccountMeta::new_readonly(system_program::ID, false), // system_program
                AccountMeta::new_readonly(self.event_authority_pda(), false), // event_authority
                AccountMeta::new_readonly(self.program_id, false),   // program
            ],
            data: serialize_instruction(&OPEN_ORDER_DISCRIMINATOR, &order_params)?,
        };

        // Build transaction
        let message = Message::new(&[instruction], Some(&sender));
        let mut tx = Transaction::new_unsigned(message);
        tx.message.recent_blockhash = blockhash;

        Ok(TransactionResult {
            transaction: BASE64_STANDARD.encode(bincode::serialize(&tx).ser_err()?),
            order_id: format!("0x{}", hex::encode(order_id)),
            nonce,
            contract_address: self.program_id.to_string(),
        })
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Cancel Order Transaction
    // ─────────────────────────────────────────────────────────────────────────

    pub async fn build_cancel_order_transaction(
        &self,
        input: &CancelOrderInput,
    ) -> Result<TransactionResult> {
        let caller = parse_pubkey(&input.caller_address)?;
        let blockhash = self.get_blockhash().await?;

        let is_native =
            input.origin_chain_id == input.dest_chain_id && input.dest_chain_id == self.chain_id;

        let instructions = if is_native {
            self.build_cancel_native_instruction(input, &caller).await?
        } else {
            self.build_cancel_foreign_instruction(input, &caller)
                .await?
        };

        // Fetch lookup table
        let lookup_table = self.get_lookup_table(&parse_pubkey(LOOKUP_TABLE)?).await?;

        // Build versioned transaction
        let message = v0::Message::try_compile(&caller, &instructions, &[lookup_table], blockhash)
            .ser_err()?;
        let tx = VersionedTransaction {
            signatures: vec![Default::default()],
            message: VersionedMessage::V0(message),
        };

        Ok(TransactionResult {
            transaction: BASE64_STANDARD.encode(bincode::serialize(&tx).ser_err()?),
            order_id: format!("0x{}", hex::encode(input.order_id)),
            nonce: input.nonce,
            contract_address: self.program_id.to_string(),
        })
    }

    async fn build_cancel_native_instruction(
        &self,
        input: &CancelOrderInput,
        caller: &Pubkey,
    ) -> Result<Vec<Instruction>> {
        let sender = Pubkey::new_from_array(input.sender);
        let token_in_mint = Pubkey::new_from_array(input.token_in);
        let token_program = self.get_mint_token_program(&token_in_mint).await?;
        let order_pda = self.order_pda(&input.order_id);

        let sender_ata =
            get_associated_token_address_with_program_id(&sender, &token_in_mint, &token_program);
        let order_ata = get_associated_token_address_with_program_id(
            &order_pda,
            &token_in_mint,
            &token_program,
        );

        let mut data = Vec::with_capacity(40);
        data.extend_from_slice(&CANCEL_NATIVE_ORDER_DISCRIMINATOR);
        data.extend_from_slice(&input.order_id);

        Ok(vec![Instruction {
            program_id: self.program_id,
            accounts: vec![
                AccountMeta::new_readonly(*caller, true), // signer
                AccountMeta::new_readonly(sender, false), // sender
                AccountMeta::new_readonly(self.global_pda(), false), // global
                AccountMeta::new(order_pda, false),       // order
                AccountMeta::new_readonly(token_in_mint, false), // token_in_mint
                AccountMeta::new(sender_ata, false),      // sender_token_in
                AccountMeta::new(order_ata, false),       // order_token_in
                AccountMeta::new_readonly(token_program, false), // token_program
                AccountMeta::new_readonly(self.event_authority_pda(), false), // event_authority
                AccountMeta::new_readonly(self.program_id, false), // program
            ],
            data,
        }])
    }

    async fn build_cancel_foreign_instruction(
        &self,
        input: &CancelOrderInput,
        caller: &Pubkey,
    ) -> Result<Vec<Instruction>> {
        let portal_program = parse_pubkey(PORTAL_PROGRAM_ID)?;
        let bridge_adapter = parse_pubkey(BRIDGE_ADAPTER)?;

        let portal_global = find_pda(&[GLOBAL_SEED], &portal_program);
        let portal_authority = find_pda(&[b"authority"], &portal_program);

        let order_data = OrderData {
            version: input.version,
            sender: input.sender,
            nonce: input.nonce,
            origin_chain_id: input.origin_chain_id,
            dest_chain_id: input.dest_chain_id,
            created_at: input.created_at,
            fill_deadline: input.fill_deadline,
            token_in: input.token_in,
            token_out: input.token_out,
            amount_in: input.amount_in,
            amount_out: input.amount_out,
            recipient: input.recipient,
            solver: input.solver,
        };

        let mut data = Vec::with_capacity(256);
        data.extend_from_slice(&CANCEL_FOREIGN_ORDER_DISCRIMINATOR);
        data.extend_from_slice(&input.order_id);
        order_data.serialize(&mut data).ser_err()?;

        let mut accounts = vec![
            AccountMeta::new(*caller, true),                          // signer
            AccountMeta::new(self.global_pda(), false),               // global
            AccountMeta::new(self.order_pda(&input.order_id), false), // order
            AccountMeta::new_readonly(portal_program, false),         // portal_program
            AccountMeta::new(portal_global, false),                   // portal_global
            AccountMeta::new_readonly(portal_authority, false),       // portal_authority
            AccountMeta::new_readonly(bridge_adapter, false),         // bridge_adapter
            AccountMeta::new_readonly(system_program::ID, false),     // system_program
            AccountMeta::new_readonly(self.event_authority_pda(), false), // event_authority
            AccountMeta::new_readonly(self.program_id, false),        // program
        ];
        accounts.extend(WormholeRemainingAccounts::account_metas(true));

        let sequence = wormhole::get_current_sequence(&self.rpc_client, true)
            .await
            .rpc_err()?;
        let wormhole_chain_id = get_wormhole_chain_id(input.origin_chain_id)
            .ok_or_else(|| TransactionBuilderError::InvalidAddress("Unknown chain ID".into()))?;

        // TODO: Replace hardcoded relay target address
        let relay_target = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 107, 42, 123, 250, 95, 28, 3, 235, 250, 231, 121,
            223, 105, 136, 184, 172, 20, 202, 65, 85,
        ];

        let relay_ix = build_relay_instruction(
            caller,
            wormhole_chain_id,
            sequence,
            &relay_target,
            Some(500_000),
            Some(20_000_000),
        )
        .await
        .map_err(|e| TransactionBuilderError::RpcError(e.to_string()))?;

        Ok(vec![
            Instruction {
                program_id: self.program_id,
                accounts,
                data,
            },
            relay_ix,
        ])
    }
}

/// Parse a string as [u8; 32], supporting Solana pubkeys, hex (with/without 0x), and EVM addresses
fn parse_bytes32(s: &str) -> Result<[u8; 32]> {
    // Try Solana base58 pubkey
    if let Ok(pubkey) = Pubkey::from_str(s) {
        return Ok(pubkey.to_bytes());
    }

    let s = s.strip_prefix("0x").unwrap_or(s);

    match s.len() {
        // Full 32-byte hex
        64 => hex::decode(s)
            .addr_err()?
            .try_into()
            .map_err(|_| TransactionBuilderError::InvalidAddress("Invalid length".into())),

        // EVM address (20 bytes) - left-pad with zeros
        40 => {
            let mut bytes = [0u8; 32];
            bytes[12..].copy_from_slice(&hex::decode(s).addr_err()?);
            Ok(bytes)
        }

        _ => Err(TransactionBuilderError::InvalidAddress(format!(
            "Cannot parse: {}",
            s
        ))),
    }
}
