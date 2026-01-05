use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::post,
    Router,
};
use std::sync::Arc;

use crate::grpc_server::QuoteGrpcService;
use crate::models::QuoteRequest;

#[derive(Clone)]
pub struct ApiState {
    pub grpc_service: QuoteGrpcService,
}

pub fn create_router(state: ApiState) -> Router {
    Router::new()
        .route("/quote", post(handle_quote_request))
        .with_state(Arc::new(state))
}

async fn handle_quote_request(
    State(state): State<Arc<ApiState>>,
    Json(request): Json<QuoteRequest>,
) -> impl IntoResponse {
    tracing::info!(
        "Received quote request: {} {} -> {} {}",
        request.input_token,
        request.input_chain_id,
        request.output_token,
        request.output_chain_id
    );

    let responses = state.grpc_service.request_quotes(request).await;

    tracing::info!("Collected {} quote responses", responses.len());

    (StatusCode::OK, Json(responses))
}
