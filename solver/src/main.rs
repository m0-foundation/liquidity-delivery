use slog::{info, Drain};
use solver::config::{Config, Environment};
use solver::loki::LokiDrain;
use solver::make_logger;
use std::collections::HashMap;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let config_path = std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config.yaml".to_string());

    let config = Config::from_file(&config_path)?;

    // Create base drain (stdout)
    let stdout_drain = if config.environment != Environment::Local {
        // JSON format for production
        let drain = slog_json::Json::default(std::io::stdout()).fuse();
        slog_async::Async::new(drain).build().fuse()
    } else {
        // Human-readable format for development
        let decorator = slog_term::TermDecorator::new().build();
        let drain = slog_term::FullFormat::new(decorator).build().fuse();
        slog_async::Async::new(drain).build().fuse()
    };

    // Create logger with optional Loki drain
    let logger = if let Some(loki_url) = &config.loki_url {
        let mut labels = HashMap::new();
        labels.insert("service".to_string(), "solver".to_string());
        labels.insert("environment".to_string(), config.environment.to_str());

        let loki_drain = LokiDrain::new(loki_url.clone(), labels);
        let combined_drain = slog::Duplicate::new(stdout_drain, loki_drain).fuse();
        make_logger!(combined_drain, config.environment.to_str())
    } else {
        make_logger!(stdout_drain, config.environment.to_str())
    };

    let shutdown_tx = solver::run_solver(config, logger.clone()).await?;

    // Wait for SIGINT (Ctrl+C)
    tokio::signal::ctrl_c().await?;
    info!(logger, "Received shutdown signal");
    let _ = shutdown_tx.send(());

    // Wait for components to shutdown gracefully
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    Ok(())
}
