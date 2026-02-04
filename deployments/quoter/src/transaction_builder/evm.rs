use alloy::primitives::{address, Address, Bytes, FixedBytes, U256};
use alloy::providers::ProviderBuilder;
use alloy::sol;
use alloy::sol_types::SolCall;
use alloy::transports::http::reqwest;
use m0_portal_common::get_wormhole_chain_id;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use super::error::TransactionBuilderError;
use super::{CancelOrderInput, EvmTransactionResult, OpenOrderInput};
use crate::models::EvmTransaction;

const DEFAULT_GAS_LIMIT: u128 = 750_000;
const DEFAULT_MSG_VALUE: u128 = 25_000_000;

// Solana devnet/mainnet chain IDs
const SOLANA_DEVNET: u32 = 1399811149;
const SOLANA_MAINNET: u32 = 1399811150;

sol! {
    #[sol(rpc)]
    interface IOrderBook {
        struct OrderParams {
            uint32 destChainId;
            uint32 fillDeadline;
            address tokenIn;
            bytes32 tokenOut;
            uint128 amountIn;
            uint128 amountOut;
            bytes32 recipient;
            bytes32 solver;
        }

        struct OrderData {
            uint16 version;
            bytes32 sender;
            uint64 nonce;
            uint32 originChainId;
            uint32 destChainId;
            uint64 createdAt;
            uint64 fillDeadline;
            bytes32 tokenIn;
            bytes32 tokenOut;
            uint128 amountIn;
            uint128 amountOut;
            bytes32 recipient;
            bytes32 solver;
        }

        function openOrder(OrderParams calldata orderParams_) external returns (bytes32);
        function cancelOrder(
            bytes32 orderId_,
            OrderData calldata orderData_,
            address bridgeAdapter_,
            bytes calldata bridgeAdapterArgs_
        ) external payable returns (bytes32);

        function getSenderNonce(address sender_) external view returns (uint64);
    }
}

sol! {
    #[sol(rpc)]
    interface IPortal {
        #[derive(Debug)]
        enum PayloadType {
            TokenTransfer,
            Index,
            RegistrarKey,
            RegistrarList,
            FillReport
        }

        function quote(uint32 destinationChainId, PayloadType payloadType) external view returns (uint256);
    }
}

sol! {
    #[sol(rpc)]
    interface IERC20 {
        function allowance(address owner, address spender) external view returns (uint256);
        function approve(address spender, uint256 amount) external returns (bool);
    }
}

type Result<T> = std::result::Result<T, TransactionBuilderError>;

/// Helper trait for parsing addresses with consistent error handling
trait ParseAddress {
    fn parse_address(&self) -> Result<Address>;
}

impl ParseAddress for str {
    fn parse_address(&self) -> Result<Address> {
        Address::from_str(self).map_err(|e| TransactionBuilderError::InvalidAddress(e.to_string()))
    }
}

pub struct EvmTransactionBuilder {
    rpc_url: url::Url,
    contract_address: Address,
    #[allow(dead_code)]
    chain_id: u32,
}

impl EvmTransactionBuilder {
    pub fn new(rpc_url: String, contract_address: String, chain_id: u32) -> Result<Self> {
        Ok(Self {
            rpc_url: rpc_url
                .parse()
                .map_err(|e| TransactionBuilderError::RpcError(format!("Invalid RPC URL: {e}")))?,
            contract_address: contract_address.as_str().parse_address()?,
            chain_id,
        })
    }

    fn contract_address_hex(&self) -> String {
        format!("{:?}", self.contract_address)
    }

    pub async fn get_sender_nonce(&self, sender: &str) -> Result<u64> {
        let provider = ProviderBuilder::new().connect_http(self.rpc_url.clone());
        let contract = IOrderBook::new(self.contract_address, &provider);
        contract
            .getSenderNonce(sender.parse_address()?)
            .call()
            .await
            .map_err(|e| TransactionBuilderError::RpcError(e.to_string()))
    }

    pub async fn get_allowance(&self, token: &str, owner: &str, spender: &str) -> Result<U256> {
        let provider = ProviderBuilder::new().connect_http(self.rpc_url.clone());
        let contract = IERC20::new(token.parse_address()?, &provider);
        contract
            .allowance(owner.parse_address()?, spender.parse_address()?)
            .call()
            .await
            .map_err(|e| TransactionBuilderError::RpcError(e.to_string()))
    }

    pub fn build_approve_calldata(
        token: &str,
        spender: &str,
        amount: u128,
    ) -> Result<EvmTransaction> {
        let calldata = IERC20::approveCall {
            spender: spender.parse_address()?,
            amount: U256::from(amount),
        }
        .abi_encode();

        Ok(EvmTransaction {
            to: token.to_string(),
            data: format!("0x{}", hex::encode(&calldata)),
            value: "0x0".to_string(),
        })
    }

    pub async fn build_open_order_calldata(
        &self,
        input: &OpenOrderInput,
    ) -> Result<EvmTransactionResult> {
        let nonce = self.get_sender_nonce(&input.sender_address).await?;
        let contract_addr = self.contract_address_hex();

        let current_allowance = self
            .get_allowance(&input.token_in, &input.sender_address, &contract_addr)
            .await?;

        let approval_transaction = (current_allowance < U256::from(input.amount_in))
            .then(|| Self::build_approve_calldata(&input.token_in, &contract_addr, u128::MAX))
            .transpose()?;

        let order_params = IOrderBook::OrderParams {
            destChainId: input.dest_chain_id,
            fillDeadline: input.fill_deadline as u32,
            tokenIn: input.token_in.as_str().parse_address()?,
            tokenOut: parse_bytes32(&input.token_out)?,
            amountIn: input.amount_in as u128,
            amountOut: input.amount_out,
            recipient: FixedBytes::from(input.recipient),
            solver: FixedBytes::from(input.solver),
        };

        let calldata = IOrderBook::openOrderCall {
            orderParams_: order_params,
        }
        .abi_encode();

        Ok(EvmTransactionResult {
            transaction: EvmTransaction {
                to: contract_addr.clone(),
                data: format!("0x{}", hex::encode(&calldata)),
                value: "0x0".to_string(),
            },
            approval_transaction,
            order_id: None, // EVM uses block.timestamp; must extract from logs
            nonce,
            contract_address: contract_addr,
        })
    }

    pub async fn build_cancel_order_calldata(
        &self,
        input: &CancelOrderInput,
    ) -> Result<EvmTransaction> {
        let order_data = IOrderBook::OrderData {
            version: input.version,
            sender: FixedBytes::from(input.sender),
            nonce: input.nonce,
            originChainId: input.origin_chain_id,
            destChainId: input.dest_chain_id,
            createdAt: input.created_at,
            fillDeadline: input.fill_deadline,
            tokenIn: FixedBytes::from(input.token_in),
            tokenOut: FixedBytes::from(input.token_out),
            amountIn: input.amount_in,
            amountOut: input.amount_out,
            recipient: FixedBytes::from(input.recipient),
            solver: FixedBytes::from(input.solver),
        };

        let (bridge_adapter, bridge_adapter_args, msg_value) = self
            .get_bridge_params(input.origin_chain_id, input.dest_chain_id)
            .await?;

        let calldata = IOrderBook::cancelOrderCall {
            orderId_: FixedBytes::from(input.order_id),
            orderData_: order_data,
            bridgeAdapter_: bridge_adapter,
            bridgeAdapterArgs_: bridge_adapter_args,
        }
        .abi_encode();

        Ok(EvmTransaction {
            to: self.contract_address_hex(),
            data: format!("0x{}", hex::encode(&calldata)),
            value: format!("0x{:x}", msg_value),
        })
    }

    async fn get_bridge_params(
        &self,
        origin_chain_id: u32,
        dest_chain_id: u32,
    ) -> Result<(Address, Bytes, U256)> {
        if matches!(origin_chain_id, SOLANA_DEVNET | SOLANA_MAINNET) {
            let (signed_quote, estimated_cost) = self
                .fetch_wormhole_quote(dest_chain_id, origin_chain_id)
                .await?;
            let wormhole_adapter = address!("0x6b2A7bFa5F1C03EbFae779Df6988b8aC14CA4155");
            Ok((wormhole_adapter, Bytes::from(signed_quote), estimated_cost))
        } else {
            let provider = ProviderBuilder::new().connect_http(self.rpc_url.clone());
            let portal = IPortal::new(
                address!("0x50D65829Eae411B655bAA92539E4F8c46D20638C"),
                &provider,
            );
            let quote = portal
                .quote(origin_chain_id, IPortal::PayloadType::FillReport)
                .call()
                .await
                .map_err(|e| TransactionBuilderError::RpcError(e.to_string()))?;
            Ok((Address::ZERO, Bytes::new(), quote))
        }
    }

    async fn fetch_wormhole_quote(
        &self,
        src_chain_id: u32,
        dst_chain_id: u32,
    ) -> Result<(Vec<u8>, U256)> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Request {
            src_chain: u16,
            dst_chain: u16,
            relay_instructions: String,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Response {
            signed_quote: String,
            estimated_cost: Option<String>,
        }

        let relay_instructions = encode_relay_instructions(DEFAULT_GAS_LIMIT, DEFAULT_MSG_VALUE);

        let response: Response = reqwest::Client::new()
            .post("https://executor-testnet.labsapis.com/v0/quote")
            .json(&Request {
                src_chain: get_wormhole_chain_id(src_chain_id).unwrap(),
                dst_chain: get_wormhole_chain_id(dst_chain_id).unwrap(),
                relay_instructions,
            })
            .send()
            .await
            .map_err(|e| TransactionBuilderError::RpcError(e.to_string()))?
            .json()
            .await
            .map_err(|e| TransactionBuilderError::RpcError(e.to_string()))?;

        let signed_quote = hex::decode(
            response
                .signed_quote
                .strip_prefix("0x")
                .unwrap_or(&response.signed_quote),
        )
        .map_err(|e| TransactionBuilderError::InvalidAddress(e.to_string()))?;

        let estimated_cost = response
            .estimated_cost
            .and_then(|c| c.parse::<u128>().ok())
            .map_or(U256::ZERO, U256::from);

        Ok((signed_quote, estimated_cost))
    }
}

fn encode_relay_instructions(gas_limit: u128, msg_value: u128) -> String {
    let mut data = Vec::with_capacity(33);
    data.push(1u8);
    data.extend_from_slice(&gas_limit.to_be_bytes());
    data.extend_from_slice(&msg_value.to_be_bytes());
    format!("0x{}", hex::encode(data))
}

fn parse_bytes32(s: &str) -> Result<FixedBytes<32>> {
    let s = s.strip_prefix("0x").unwrap_or(s);

    // EVM address (20 bytes hex) - left-pad with zeros
    if s.len() == 40 {
        let mut bytes = [0u8; 32];
        bytes[12..].copy_from_slice(
            &hex::decode(s).map_err(|e| TransactionBuilderError::InvalidAddress(e.to_string()))?,
        );
        return Ok(FixedBytes::from(bytes));
    }

    // Full bytes32 (32 bytes hex)
    if s.len() == 64 {
        let bytes: [u8; 32] = hex::decode(s)
            .map_err(|e| TransactionBuilderError::InvalidAddress(e.to_string()))?
            .try_into()
            .map_err(|_| TransactionBuilderError::InvalidAddress("Invalid length".into()))?;
        return Ok(FixedBytes::from(bytes));
    }

    // Base58 (Solana pubkey)
    let bytes: [u8; 32] = bs58::decode(s)
        .into_vec()
        .map_err(|e| TransactionBuilderError::InvalidAddress(e.to_string()))?
        .try_into()
        .map_err(|_| TransactionBuilderError::InvalidAddress(format!("Cannot parse: {s}")))?;
    Ok(FixedBytes::from(bytes))
}
