use async_trait::async_trait;

use crate::error::Result;
use crate::events::{EventHandler, SolverEvent};

/// Component that listens to new orders created
pub struct InventoryManager {}

impl InventoryManager {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl EventHandler for InventoryManager {
    fn name(&self) -> &'static str {
        "InventoryManager"
    }

    async fn initialize(&self) -> Result<()> {
        tracing::info!("Initializing");
        Ok(())
    }

    async fn handle_event(&self, _event: SolverEvent) -> Result<Vec<SolverEvent>> {
        Ok(vec![])
    }
}
