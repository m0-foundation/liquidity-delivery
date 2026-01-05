use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use slog::{info, Logger};
use std::sync::Arc;

use crate::grpc_server::QuoteGrpcService;
use crate::models::QuoteRequest;

#[derive(Clone)]
pub struct ApiState {
    pub grpc_service: QuoteGrpcService,
    pub logger: Logger,
}

pub fn create_router(state: ApiState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/quote", post(handle_quote_request))
        .with_state(Arc::new(state))
}

async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

async fn handle_quote_request(
    State(state): State<Arc<ApiState>>,
    Json(request): Json<QuoteRequest>,
) -> impl IntoResponse {
    info!(
        state.logger,
        "Received quote request";
        "input_token" => &request.input_token,
        "input_chain_id" => request.input_chain_id,
        "output_token" => &request.output_token,
        "output_chain_id" => request.output_chain_id,
        "amount_in" => request.amount_in
    );

    let responses = state.grpc_service.request_quotes(request).await;
    info!(
        state.logger,
        "Collected quote responses";
        "count" => responses.len(),
        "rejects" => responses.iter().filter(|r| r.rejected).count()
    );

    (StatusCode::OK, Json(responses))
}
