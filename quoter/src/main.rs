mod api_server;
mod grpc_server;
mod models;

use api_server::{create_router, ApiState};
use grpc_server::QuoteGrpcService;
use tonic::transport::Server;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "quoter=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let grpc_service = QuoteGrpcService::new();

    let grpc_addr = "[::1]:50051".parse()?;
    let api_addr = "0.0.0.0:3000";

    // Spawn gRPC server
    let grpc_service_clone = grpc_service.clone();
    let grpc_server = tokio::spawn(async move {
        Server::builder()
            .add_service(grpc_service_clone.get_server())
            .serve(grpc_addr)
            .await
    });

    // Spawn HTTP API server
    let api_state = ApiState {
        grpc_service: grpc_service.clone(),
    };

    let app = create_router(api_state);
    let listener = tokio::net::TcpListener::bind(api_addr).await?;
    let api_server = tokio::spawn(async move { axum::serve(listener, app).await });

    info!(
        "Servers running: gRPC on {}, HTTP API on {}",
        grpc_addr, api_addr
    );

    // Wait for both servers
    tokio::select! {
        result = grpc_server => {
            match result {
                Ok(Ok(_)) => tracing::info!("gRPC server stopped"),
                Ok(Err(e)) => tracing::error!("gRPC server error: {}", e),
                Err(e) => tracing::error!("gRPC server task error: {}", e),
            }
        }
        result = api_server => {
            match result {
                Ok(Ok(_)) => tracing::info!("API server stopped"),
                Ok(Err(e)) => tracing::error!("API server error: {}", e),
                Err(e) => tracing::error!("API server task error: {}", e),
            }
        }
    }

    Ok(())
}
