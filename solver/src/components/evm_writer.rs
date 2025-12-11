use async_trait::async_trait;
use slog::Logger;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::components::ComponentParams;
use crate::config::Signers;
use crate::error::Result;
use crate::events::{EventHandler, EventProcessor, SolverEvent};
use crate::providers::ProviderManager;
use crate::stores::OrderStore;

pub struct EvmWriter {
    signers: Signers,
    order_store: Arc<RwLock<OrderStore>>,
    provider_manager: Arc<ProviderManager>,
    logger: Logger,
}

impl EvmWriter {
    pub fn new(params: &ComponentParams) -> Self {
        Self {
            signers: params.config.signers.clone(),
            order_store: Arc::new(RwLock::new(OrderStore::new())),
            provider_manager: params.provider_manager.clone(),
            logger: params.logger.new(slog::o!("component" => "EvmWriter")),
        }
    }
}

#[async_trait]
impl EventHandler for EvmWriter {
    fn name(&self) -> &'static str {
        "EvmWriter"
    }

    async fn initialize(&self) -> Result<()> {
        self.order_store.write().await.initialize().await?;

        Ok(())
    }

    async fn handle_event(&self, event: SolverEvent) -> Result<Vec<SolverEvent>> {
        let store = self.order_store.read().await;
        let _ = store.handle_event(event.clone()).await;

        match event {
            SolverEvent::RequestFillOrder(_e) => {}
            _ => {}
        }

        Ok(vec![])
    }
}
