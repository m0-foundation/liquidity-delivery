use alloy::primitives::Address;
use solver::utils::chain_from_id;

#[derive(Clone, Debug)]
pub struct Asset {
    pub address: Address,
    pub chain_id: u32,
    pub symbol: String,
}

impl Asset {
    fn to_json(&self) -> String {
        format!(
            r#"{{
                "chain": "{}",
                "address": "{}",
                "symbol": "{}",
                "icon": "",
                "name": "{}",
                "decimals": 6,
                "m0Extension": false,
                "runtime": "evm"
            }}"#,
            chain_from_id(self.chain_id),
            self.address,
            self.symbol,
            self.symbol
        )
    }
}

pub async fn mock_api_with_assets(additional_assets: Vec<Asset>) -> mockito::ServerGuard {
    let mut server = mockito::Server::new_async().await;

    let mut assets = vec![r#"{
            "chain": "Ethereum",
            "address": "0x437cc33344a0B27A429f795ff6B469C72698B291",
            "symbol": "wM",
            "icon": "",
            "name": "Wrapped $M",
            "decimals": 6,
            "m0Extension": true,
            "runtime": "evm"
        }"#
    .to_string()];

    assets.extend(additional_assets.into_iter().map(|a| a.to_json()));

    let body = format!("[{}]", assets.join(","));

    // Assets endpoint
    let _ = server
        .mock("GET", "/supported-assets")
        .with_status(200)
        .with_body(body)
        .create();

    server
}
