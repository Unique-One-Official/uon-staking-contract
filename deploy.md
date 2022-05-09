near create-account uon_staking_test_03.supernova11.testnet --masterAccount supernova11.testnet --initialBalance 6


near deploy --accountId uon_staking_test_03.supernova11.testnet --wasmFile out/masterchef.wasm --initFunction new --initArgs '{"owner_id": "supernova11.testnet"}'
