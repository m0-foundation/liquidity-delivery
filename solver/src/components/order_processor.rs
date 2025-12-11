use async_trait::async_trait;
use slog::Logger;
use std::cmp::min;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::components::ComponentParams;
use crate::error::Result;
use crate::events::{
    EventHandler, EventProcessor, OrderRejectEvent, RequestFillOrderEvent, RequestHoldEvent,
    SolverEvent,
};
use crate::stores::{AssetStore, OrderStore};

pub struct OrderProcessor {
    order_store: Arc<RwLock<OrderStore>>,
    asset_store: Arc<RwLock<AssetStore>>,
    logger: Logger,
    max_clip_size: u128,
    fee_bps: u128,
}

impl OrderProcessor {
    pub fn new(params: &ComponentParams) -> Self {
        Self {
            order_store: Arc::new(RwLock::new(OrderStore::new())),
            asset_store: Arc::new(RwLock::new(AssetStore::new(
                params.config.liquidity_api_url.clone(),
            ))),
            logger: params.logger.new(slog::o!("component" => "OrderProcessor")),
            max_clip_size: params.config.max_order_clip_size as u128,
            fee_bps: params.config.solver_fee_bps as u128,
        }
    }
}

#[async_trait]
impl EventHandler for OrderProcessor {
    fn name(&self) -> &'static str {
        "OrderProcessor"
    }

    async fn initialize(&self) -> Result<()> {
        self.order_store.write().await.initialize().await?;
        self.asset_store.write().await.initialize().await?;
        Ok(())
    }

    async fn handle_event(&self, event: SolverEvent) -> Result<Vec<SolverEvent>> {
        let store = self.order_store.read().await;
        let _ = store.handle_event(event.clone()).await;

        match event {
            SolverEvent::OrderCreated(e) => {
                let asset_store = self.asset_store.read().await;

                let source_asset = (*asset_store)
                    .get_asset(e.token_in, e.order.origin_chain_id)
                    .await?;
                let destination_asset = (*asset_store)
                    .get_asset(e.order.token_out, e.order.dest_chain_id)
                    .await?;

                // Ignore orders on assets we don't support
                if source_asset.is_none() || destination_asset.is_none() {
                    return Ok(vec![SolverEvent::OrderRejected(OrderRejectEvent::new(
                        e.order_id,
                        "Asset not supported".to_string(),
                    ))]);
                }

                // TODO: get whitelist assets from config

                // Make sure amount_out covers our fee
                let min_amount_out = e.order.amount_in * (10_000 - self.fee_bps) / 10_000;
                if min_amount_out < e.order.amount_out {
                    return Ok(vec![SolverEvent::OrderRejected(OrderRejectEvent::new(
                        e.order_id,
                        format!(
                            "Order amount_out {} does not cover fee-inclusive amount_out {}",
                            e.order.amount_out, min_amount_out
                        ),
                    ))]);
                }

                // Clip large orders
                let fill_amount = min(e.order.amount_out, self.max_clip_size);

                // Request hold on destination asset
                return Ok(vec![SolverEvent::RequestHold(RequestHoldEvent::new(
                    e.order_id,
                    destination_asset.unwrap(),
                    fill_amount,
                ))]);
            }
            SolverEvent::HoldSuccessful(e) => {
                return Ok(vec![SolverEvent::RequestFillOrder(
                    RequestFillOrderEvent::new(e.order_id, e.hold_amount),
                )]);
            }
            _ => {}
        }

        Ok(vec![])
    }
}
