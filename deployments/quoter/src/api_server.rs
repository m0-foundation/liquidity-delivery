use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use slog::{warn, Logger};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use crate::config::{ChainConfig, ChainType};
use crate::grpc_server::QuoteGrpcService;
use crate::models::{CancelRequest, CancelResponse, EvmTransaction, QuoteRequest};
use crate::transaction_builder::{
    CancelOrderInput, EvmTransactionBuilder, EvmTransactionResult, OpenOrderInput,
    SvmTransactionBuilder, TransactionBuilderError, TransactionResult,
};

#[derive(Clone)]
pub struct ApiState {
    pub grpc_service: QuoteGrpcService,
    pub chains: Vec<ChainConfig>,
    pub logger: Logger,
}

pub fn create_router(state: ApiState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/quote", post(handle_quote_request))
        .route("/cancel", post(handle_cancel_request))
        .layer(CorsLayer::permissive())
        .with_state(Arc::new(state))
}

async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

async fn handle_quote_request(
    State(state): State<Arc<ApiState>>,
    Json(request): Json<QuoteRequest>,
) -> impl IntoResponse {
    let mut quotes = state.grpc_service.request_quotes(request.clone()).await;

    // If sender_address is provided, build transactions for each non-rejected quote
    if let Some(ref sender_address) = request.sender_address {
        // Find the input chain config
        let input_chain = state
            .chains
            .iter()
            .find(|c| c.chain_id == request.input_chain_id);

        if let Some(chain) = input_chain {
            let fill_deadline = u32::MAX as u64;
            let recipient = request.recipient.as_ref().unwrap_or(sender_address);

            for quote in quotes.iter_mut() {
                if quote.rejected {
                    continue;
                }

                // Parse recipient address
                let recipient_bytes = match parse_address_to_bytes32(recipient) {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        warn!(state.logger, "Failed to parse recipient address"; "error" => %e);
                        continue;
                    }
                };

                // Parse solver address
                let solver_bytes = if quote.solver_address.is_empty() {
                    [0u8; 32] // No solver restriction
                } else {
                    match parse_address_to_bytes32(&quote.solver_address) {
                        Ok(bytes) => bytes,
                        Err(e) => {
                            warn!(state.logger, "Failed to parse solver address"; "error" => %e);
                            [0u8; 32]
                        }
                    }
                };

                let input = OpenOrderInput {
                    sender_address: sender_address.clone(),
                    token_in: request.input_token.clone(),
                    token_out: request.output_token.clone(),
                    amount_in: request.amount_in,
                    amount_out: quote.output_amount as u128,
                    recipient: recipient_bytes,
                    solver: solver_bytes,
                    dest_chain_id: request.output_chain_id,
                    fill_deadline,
                };

                match chain.chain_type {
                    ChainType::Evm => match build_evm_transaction(chain, &input).await {
                        Ok(result) => {
                            quote.evm_transaction = Some(result.transaction);
                            quote.approval_transaction = result.approval_transaction;
                            // order_id is None for EVM - frontend must get it from tx logs
                            quote.order_id = result.order_id;
                            quote.nonce = Some(result.nonce);
                            quote.orderbook_address = Some(chain.order_book_address.clone());
                        }
                        Err(e) => {
                            warn!(state.logger, "Failed to build EVM transaction"; "error" => %e);
                        }
                    },
                    ChainType::Svm => match build_svm_transaction(chain, &input).await {
                        Ok(result) => {
                            quote.svm_transaction = Some(result.transaction);
                            quote.order_id = Some(result.order_id);
                            quote.nonce = Some(result.nonce);
                            quote.orderbook_address = Some(chain.order_book_address.clone());
                        }
                        Err(e) => {
                            warn!(state.logger, "Failed to build SVM transaction"; "error" => %e);
                        }
                    },
                }
            }
        } else {
            warn!(
                state.logger,
                "Chain config not found for input chain";
                "input_chain_id" => request.input_chain_id,
                "available_chains" => ?state.chains.iter().map(|c| c.chain_id).collect::<Vec<_>>()
            );
        }
    }

    (StatusCode::OK, Json(quotes))
}

async fn build_evm_transaction(
    chain: &ChainConfig,
    input: &OpenOrderInput,
) -> Result<EvmTransactionResult, TransactionBuilderError> {
    let builder = EvmTransactionBuilder::new(
        chain.rpc_url.clone(),
        chain.order_book_address.clone(),
        chain.chain_id,
    )?;
    builder.build_open_order_calldata(input).await
}

async fn build_svm_transaction(
    chain: &ChainConfig,
    input: &OpenOrderInput,
) -> Result<TransactionResult, TransactionBuilderError> {
    let builder = SvmTransactionBuilder::new(
        chain.rpc_url.clone(),
        Some(chain.order_book_address.clone()),
        chain.chain_id,
    )?;
    builder.build_open_order_transaction(input).await
}

fn parse_address_to_bytes32(address: &str) -> Result<[u8; 32], String> {
    // Try hex (with or without 0x prefix)
    let hex_str = address.strip_prefix("0x").unwrap_or(address);
    if hex_str.len() == 40 {
        // EVM address - left-pad with zeros
        let mut bytes = [0u8; 32];
        let addr_bytes = hex::decode(hex_str).map_err(|e| e.to_string())?;
        bytes[12..].copy_from_slice(&addr_bytes);
        return Ok(bytes);
    }
    if hex_str.len() == 64 {
        // Full bytes32
        let bytes: [u8; 32] = hex::decode(hex_str)
            .map_err(|e| e.to_string())?
            .try_into()
            .map_err(|_| "Invalid length")?;
        return Ok(bytes);
    }

    // Try base58 (Solana pubkey)
    let decoded = bs58::decode(address)
        .into_vec()
        .map_err(|e| e.to_string())?;
    if decoded.len() == 32 {
        let bytes: [u8; 32] = decoded.try_into().map_err(|_| "Invalid length")?;
        return Ok(bytes);
    }

    Err(format!("Cannot parse address: {}", address))
}

/// Parse a hex string (with or without 0x prefix) as [u8; 32]
fn parse_order_id(order_id: &str) -> Result<[u8; 32], String> {
    let hex_str = order_id.strip_prefix("0x").unwrap_or(order_id);
    if hex_str.len() != 64 {
        return Err(format!("Invalid order ID length: {}", hex_str.len()));
    }
    let bytes: [u8; 32] = hex::decode(hex_str)
        .map_err(|e| e.to_string())?
        .try_into()
        .map_err(|_| "Invalid length")?;
    Ok(bytes)
}

async fn handle_cancel_request(
    State(state): State<Arc<ApiState>>,
    Json(request): Json<CancelRequest>,
) -> impl IntoResponse {
    // Parse order_id
    let order_id = match parse_order_id(&request.order_id) {
        Ok(id) => id,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": format!("Invalid order ID: {}", e) })),
            )
                .into_response();
        }
    };

    // Parse addresses to bytes32
    let sender = match parse_address_to_bytes32(&request.sender) {
        Ok(bytes) => bytes,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": format!("Invalid sender address: {}", e) })),
            )
                .into_response();
        }
    };

    let recipient = match parse_address_to_bytes32(&request.recipient) {
        Ok(bytes) => bytes,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": format!("Invalid recipient address: {}", e) })),
            )
                .into_response();
        }
    };

    let token_in = match parse_address_to_bytes32(&request.token_in) {
        Ok(bytes) => bytes,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": format!("Invalid token_in address: {}", e) })),
            )
                .into_response();
        }
    };

    let token_out = match parse_address_to_bytes32(&request.token_out) {
        Ok(bytes) => bytes,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": format!("Invalid token_out address: {}", e) })),
            )
                .into_response();
        }
    };

    let solver = match parse_address_to_bytes32(&request.solver) {
        Ok(bytes) => bytes,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": format!("Invalid solver address: {}", e) })),
            )
                .into_response();
        }
    };

    // Find the destination chain config (cancel tx goes to destination chain)
    let dest_chain = state
        .chains
        .iter()
        .find(|c| c.chain_id == request.dest_chain_id);

    let chain = match dest_chain {
        Some(c) => c,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": format!("Unsupported destination chain: {}", request.dest_chain_id) })),
            )
                .into_response();
        }
    };

    let cancel_input = CancelOrderInput {
        order_id,
        version: request.version,
        sender,
        nonce: request.nonce,
        origin_chain_id: request.origin_chain_id,
        dest_chain_id: request.dest_chain_id,
        created_at: request.created_at,
        fill_deadline: request.fill_deadline,
        token_in,
        token_out,
        amount_in: request.amount_in as u128,
        amount_out: request.amount_out as u128,
        recipient,
        solver,
        caller_address: request.caller_address.clone(),
    };

    match chain.chain_type {
        ChainType::Evm => match build_evm_cancel_transaction(chain, &cancel_input).await {
            Ok(tx) => {
                let response = CancelResponse {
                    order_id: request.order_id,
                    evm_transaction: Some(tx),
                    svm_transaction: None,
                    orderbook_address: chain.order_book_address.clone(),
                    chain_id: chain.chain_id,
                };
                (StatusCode::OK, Json(response)).into_response()
            }
            Err(e) => {
                warn!(state.logger, "Failed to build EVM cancel transaction"; "error" => %e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": format!("Failed to build transaction: {}", e) })),
                )
                    .into_response()
            }
        },
        ChainType::Svm => match build_svm_cancel_transaction(chain, &cancel_input).await {
            Ok(result) => {
                let response = CancelResponse {
                    order_id: request.order_id,
                    evm_transaction: None,
                    svm_transaction: Some(result.transaction),
                    orderbook_address: chain.order_book_address.clone(),
                    chain_id: chain.chain_id,
                };
                (StatusCode::OK, Json(response)).into_response()
            }
            Err(e) => {
                warn!(state.logger, "Failed to build SVM cancel transaction"; "error" => %e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": format!("Failed to build transaction: {}", e) })),
                )
                    .into_response()
            }
        },
    }
}

async fn build_evm_cancel_transaction(
    chain: &ChainConfig,
    input: &CancelOrderInput,
) -> Result<EvmTransaction, TransactionBuilderError> {
    let builder = EvmTransactionBuilder::new(
        chain.rpc_url.clone(),
        chain.order_book_address.clone(),
        chain.chain_id,
    )?;
    builder.build_cancel_order_calldata(input).await
}

async fn build_svm_cancel_transaction(
    chain: &ChainConfig,
    input: &CancelOrderInput,
) -> Result<TransactionResult, TransactionBuilderError> {
    let builder = SvmTransactionBuilder::new(
        chain.rpc_url.clone(),
        Some(chain.order_book_address.clone()),
        chain.chain_id,
    )?;
    builder.build_cancel_order_transaction(input).await
}
