use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::components::Component;
use crate::error::Result;
use crate::events::{EventHandler, OrderEvent};
use crate::EventBus;

/// Component that listens to new orders created
pub struct InventoryManager {}

impl InventoryManager {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl EventHandler for InventoryManager {
    async fn handle_event(&self, _event: Arc<OrderEvent>) -> Result<()> {
        // TODO: Implement inventory management logic
        Ok(())
    }
}

#[async_trait]
impl Component for InventoryManager {
    fn name() -> &'static str {
        "InventoryManager"
    }

    async fn initialize(&self) -> Result<()> {
        tracing::info!("Initializing");
        Ok(())
    }

    async fn start(
        &self,
        event_bus: Arc<EventBus>,
        shutdown_rx: broadcast::Receiver<()>,
    ) -> Result<()> {
        tracing::info!("Starting");

        Self::spawn_event_handler(event_bus, shutdown_rx, |_event| async move {
            // TODO: Implement inventory management logic
            Ok(())
        });

        Ok(())
    }
}
