build-and-publish-solana-devnet:
	cargo build -p orderbook-substream --target wasm32-unknown-unknown --release --no-default-features --features svm
	substreams pack substreams-solana-devnet.yaml
	substreams publish solana-devnet-orderbook-v0.1.1.spkg

build-and-publish-sepolia:
	cargo build -p orderbook-substream --target wasm32-unknown-unknown --release --no-default-features --features evm
	substreams pack substreams-sepolia.yaml
	substreams publish sepolia-orderbook-v0.1.1.spkg

build-and-publish-arbitrum-sepolia:
	cargo build -p orderbook-substream --target wasm32-unknown-unknown --release --no-default-features --features evm
	substreams pack substreams-arbitrum-sepolia.yaml
	substreams publish arbitrum-sepolia-orderbook-v0.1.1.spkg

build-docker-image:
	docker build --platform linux/amd64 -t ghcr.io/m0-foundation/liquidity-delivery:orderbook-substreams .
	docker push ghcr.io/m0-foundation/liquidity-delivery:orderbook-substreams

debug-solana-gui:
	substreams gui substreams-solana-devnet.yaml --stop-block +10 --start-block 439866040

debug-sepolia-gui:
	substreams gui substreams-sepolia.yaml --stop-block +10 --start-block 10190942

debug-arbitrum-sepolia-gui:
	substreams gui substreams-arbitrum-sepolia.yaml --stop-block +10 --start-block 239185283
