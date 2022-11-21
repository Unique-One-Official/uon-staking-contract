near create-account v1_1.stakenet.testnet --masterAccount stakenet.testnet --initialBalance 6


near deploy --accountId v1_1.stakenet.testnet --wasmFile out/masterchef.wasm --initFunction new --initArgs '{"owner_id": "stakenet.testnet"}'
