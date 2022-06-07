use crate::*;

/// callbacks from FT Contracts
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct StakingArgs {
    pub farm_id: u16,
    pub lock_duration: u64,
}

trait FungibleTokenReceiver {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128>;
}

trait MFTTokenReceiver {
    fn mft_on_transfer(
        &mut self,
        token_id: String,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128>;
}

#[near_bindgen]
impl MFTTokenReceiver for Contract {
    fn mft_on_transfer(
        &mut self,
        token_id: String,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let StakingArgs {
            farm_id,
            lock_duration,
        } = near_sdk::serde_json::from_str(&msg).expect("Invalid PurchaseArgs");

        // let pool_id = env::predecessor_account_id();
        assert!(amount.0 > 0, "Amount must be greater than 0");

        assert!(self.farm_infos.len() > farm_id.into(), "Invalid Farm ID");

        let mut pool_id: u64 = 0;
        if token_id.starts_with(":") {
            if let Ok(pool_index) = str::parse::<u64>(&token_id[1..token_id.len()]) {
                pool_id = pool_index;
            } else {
                env::panic(b"Invalid Pool ID");
            }
        } else {
            env::panic(b"Invalid Pool ID");
        }
        let mut farm_info = self.farm_infos.get(farm_id.into()).unwrap();

        if pool_id == farm_info.pool_id {
            self.update_claim_amounts(farm_id, 1);

            farm_info.total_lp_share_amount = U128::from(
                u128::from(farm_info.total_lp_share_amount)
                    .checked_add(u128::from(amount))
                    .unwrap(),
            );
            let info: Option<StakeInfo> = farm_info.stake_infos.get(&sender_id);

            let mut stake_info = if let Some(info) = info {
                info
            } else {
                let empty_value = StakeInfo {
                    owner_id: sender_id.clone(),
                    token_amount: U128(0),
                    lp_share_amount: U128(0),
                    reward_token_to_claim: U128(0),
                    reward_lp_to_claim: U128(0),
                    token_locked: Vector::new(StorageKey::LockInfos {
                        farm_id,
                        account_id: sender_id.clone(),
                        lock_info_type: 0,
                    }),
                    lp_share_locked: Vector::new(StorageKey::LockInfos {
                        farm_id,
                        account_id: sender_id.clone(),
                        lock_info_type: 1,
                    }),
                    unlocked_at: Vector::new(StorageKey::LockInfos {
                        farm_id,
                        account_id: sender_id.clone(),
                        lock_info_type: 2,
                    }),
                    staking_duration: Vector::new(StorageKey::LockInfos {
                        farm_id,
                        account_id: sender_id.clone(),
                        lock_info_type: 3,
                    }),
                    created_at: env::block_timestamp() / 1000000,
                    claimed_token_at: env::block_timestamp() / 1000000,
                    claimed_lp_at: env::block_timestamp() / 1000000,
                };
                empty_value
            };

            stake_info.token_locked.push(&U128(0));
            stake_info.lp_share_locked.push(&amount);
            stake_info
                .unlocked_at
                .push(&(env::block_timestamp() / 1000000 + lock_duration));
            stake_info.staking_duration.push(&lock_duration);
            stake_info.lp_share_amount = U128::from(
                u128::from(stake_info.lp_share_amount)
                    .checked_add(u128::from(amount))
                    .unwrap(),
            );
            stake_info.claimed_lp_at = env::block_timestamp() / 1000000;

            farm_info.stake_infos.insert(&sender_id, &stake_info);
        }
        self.farm_infos.replace(farm_id.into(), &farm_info);
        PromiseOrValue::Value(U128(0))
    }
}

#[near_bindgen]
impl FungibleTokenReceiver for Contract {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let StakingArgs {
            farm_id,
            lock_duration,
        } = near_sdk::serde_json::from_str(&msg).expect("Invalid PurchaseArgs");

        let ft_token_id = env::predecessor_account_id();
        assert!(amount.0 > 0, "Amount must be greater than 0");

        let swap_tokens = self.swap_farms.keys_as_vector().to_vec();

        if swap_tokens.contains(&ft_token_id) {
            let mut swap_farm_info = self.swap_farms.get(&ft_token_id).unwrap();
            let swap_rate = swap_farm_info.swap_rate;
            let swaped_amount = if swap_farm_info.token_decimal >= 18 {
                let pow_num = 10u128.pow(u32::from(swap_farm_info.token_decimal) - 18);
                u128::from(amount)
                    .checked_div(pow_num)
                    .unwrap()
                    .checked_mul(100000000)
                    .unwrap()
                    .checked_div(u128::from(swap_rate))
                    .unwrap()
            } else {
                let unet_decimal: u32 = 18;
                let pow_num = 10u128.pow(
                    u32::from(unet_decimal)
                        .checked_sub(u32::from(swap_farm_info.token_decimal))
                        .unwrap(),
                );
                u128::from(amount)
                    .checked_mul(pow_num)
                    .unwrap()
                    .checked_mul(100000000)
                    .unwrap()
                    .checked_div(u128::from(swap_rate))
                    .unwrap()
            };
            // let info_tmp: Option<SwapStakeInfo> = swap_farm_info.stake_infos.get(&sender_id);
            let staked_accounts = swap_farm_info.stake_infos.keys_as_vector().to_vec();

            // let mut swap_stake_info = if let Some(info_tmp) = info_tmp {
            let mut swap_stake_info = if staked_accounts.contains(&sender_id) {
                swap_farm_info.stake_infos.get(&sender_id).unwrap()
            } else {
                let empty_value = SwapStakeInfo {
                    owner_id: sender_id.clone(),
                    token_amount: U128::from(0),
                    token_locked: Vector::new(StorageKey::SwapStakeLockInfos {
                        token_id: ft_token_id.clone(),
                        account_id: sender_id.clone(),
                        lock_info_type: 0,
                    }),
                    unlocked_at: Vector::new(StorageKey::SwapStakeLockInfos {
                        token_id: ft_token_id.clone(),
                        account_id: sender_id.clone(),
                        lock_info_type: 1,
                    }),
                    created_at: env::block_timestamp() / 1000000,
                    claimed_token_at: Vector::new(StorageKey::SwapStakeLockInfos {
                        token_id: ft_token_id.clone(),
                        account_id: sender_id.clone(),
                        lock_info_type: 2,
                    }),
                };
                empty_value
            };
            swap_stake_info.token_amount = U128::from(
                u128::from(swap_stake_info.token_amount)
                    .checked_add(swaped_amount)
                    .unwrap(),
            );
            swap_stake_info
                .token_locked
                .push(&U128::from(swaped_amount));
            swap_stake_info
                .unlocked_at
                .push(&(env::block_timestamp() / 1000000 + 365 * 86400 * 3 * 1000));
            swap_stake_info
                .claimed_token_at
                .push(&(env::block_timestamp() / 1000000));

            swap_farm_info
                .stake_infos
                .insert(&sender_id, &swap_stake_info);
            self.swap_farms.insert(&ft_token_id, &swap_farm_info);
        } else {
            assert!(self.farm_infos.len() > farm_id.into(), "Invalid Farm ID");
            let mut farm_info = self.farm_infos.get(farm_id.into()).unwrap();

            if ft_token_id == farm_info.token_id {
                self.update_claim_amounts(farm_id, 0);
                farm_info.total_token_amount = U128::from(
                    u128::from(farm_info.total_token_amount)
                        .checked_add(u128::from(amount))
                        .unwrap(),
                );
                let info: Option<StakeInfo> = farm_info.stake_infos.get(&sender_id);
                let mut stake_info = if let Some(info) = info {
                    info
                } else {
                    let empty_value = StakeInfo {
                        owner_id: sender_id.clone(),
                        token_amount: U128(0),
                        lp_share_amount: U128(0),
                        reward_token_to_claim: U128(0),
                        reward_lp_to_claim: U128(0),
                        token_locked: Vector::new(StorageKey::LockInfos {
                            farm_id,
                            account_id: sender_id.clone(),
                            lock_info_type: 0,
                        }),
                        lp_share_locked: Vector::new(StorageKey::LockInfos {
                            farm_id,
                            account_id: sender_id.clone(),
                            lock_info_type: 1,
                        }),
                        unlocked_at: Vector::new(StorageKey::LockInfos {
                            farm_id,
                            account_id: sender_id.clone(),
                            lock_info_type: 2,
                        }),
                        staking_duration: Vector::new(StorageKey::LockInfos {
                            farm_id,
                            account_id: sender_id.clone(),
                            lock_info_type: 3,
                        }),
                        created_at: env::block_timestamp() / 1000000,
                        claimed_token_at: env::block_timestamp() / 1000000,
                        claimed_lp_at: env::block_timestamp() / 1000000,
                    };
                    empty_value
                };
                stake_info.token_locked.push(&amount);
                stake_info.lp_share_locked.push(&U128(0));
                stake_info
                    .unlocked_at
                    .push(&(env::block_timestamp() / 1000000 + lock_duration));
                stake_info.staking_duration.push(&lock_duration);
                stake_info.token_amount = U128::from(
                    u128::from(stake_info.token_amount)
                        .checked_add(u128::from(amount))
                        .unwrap(),
                );
                stake_info.claimed_token_at = env::block_timestamp() / 1000000;
                farm_info.stake_infos.insert(&sender_id, &stake_info);
                self.farm_infos.replace(farm_id.into(), &farm_info);
            }
        }
        PromiseOrValue::Value(U128(0))
    }
}
