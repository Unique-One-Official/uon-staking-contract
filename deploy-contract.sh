#  near deploy --accountId stake_urkobein.testnet --wasmFile out/vexedapes_staking.wasm --initFunction new --initArgs '{"owner_id": "nft_staking_urkobein.testnet", "total_supply": "10000000"}'

near create-account $@.stake_urkobein.testnet --masterAccount stake_urkobein.testnet --initialBalance 10
near deploy --accountId $@.stake_urkobein.testnet --wasmFile out/vexedapes_staking.wasm --initFunction new --initArgs '{"owner_id": "test1.stake_urkobein.testnet"}'
