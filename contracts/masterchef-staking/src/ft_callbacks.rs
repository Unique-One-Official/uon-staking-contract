use crate::*;

/// callbacks from FT Contracts
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct StakingArgs {
    pub stake_type: u8,
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
            stake_type,
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
                env::panic_str("Invalid Pool ID");
            }
        } else {
            env::panic_str("Invalid Pool ID");
        }
        let now = env::block_timestamp() / 1000000;

        self.update_claim_amounts(farm_id, 1 , now);
        let mut farm_info = self.farm_infos.get(farm_id.into()).unwrap();

        assert!(farm_info.farm_type == 1 || farm_info.farm_type == 2, "LP Pool does not exist in this Farm");


        if pool_id == farm_info.pool_id {

            let info: Option<StakeInfo> = farm_info.stake_infos.get(&sender_id);

            let mut stake_info = if let Some(info) = info {
                info
            } else {
                let empty_value = StakeInfo {
                    owner_id: sender_id.clone(),
                    token_amount: 0,
                    lp_share_amount: 0,
                    reward_amount: 0,
                    reward_token_to_claim: 0,
                    reward_lp_to_claim: 0,
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
                    reward_locked: Vector::new(StorageKey::LockInfos {
                        farm_id,
                        account_id: sender_id.clone(),
                        lock_info_type: 2,
                    }),
                    unlocked_at: Vector::new(StorageKey::LockInfos {
                        farm_id,
                        account_id: sender_id.clone(),
                        lock_info_type: 3,
                    }),
                    staking_duration: Vector::new(StorageKey::LockInfos {
                        farm_id,
                        account_id: sender_id.clone(),
                        lock_info_type: 4,
                    }),
                    lp_share_weight: 0,
                    token_weight: 0,
                    created_at: now,
                    claimed_token_at: now,
                    claimed_lp_at: now,
                    claimed_reward_at: now,
                };
                empty_value
            };

            stake_info.token_locked.push(&U128(0));
            stake_info.lp_share_locked.push(&amount);
            stake_info.reward_locked.push(&U128(0));
            stake_info
                .unlocked_at
                .push(&(now + lock_duration));
            stake_info.staking_duration.push(&lock_duration);
            stake_info.claimed_lp_at = now;
            stake_info.lp_share_amount = stake_info.lp_share_amount.checked_add(u128::from(amount)).unwrap();
            farm_info.total_lp_share_amount = farm_info.total_lp_share_amount
                    .checked_add(u128::from(amount))
                    .unwrap();
            let reward_weight = u128::from(amount).checked_mul(100000000).unwrap().checked_div(farm_info.total_lp_share_amount).unwrap().checked_mul(100000000000000000000).unwrap().checked_add(u128::from(amount).checked_mul(farm_info.pool_weight_rate.into()).unwrap().checked_div(10000).unwrap().checked_div(u128::from(farm_info.max_token_vesting_duration)).unwrap().checked_mul(u128::from(lock_duration)).unwrap()).unwrap();
            stake_info.lp_share_weight = stake_info.lp_share_weight.checked_add(reward_weight).unwrap();
            farm_info.total_lp_share_weight = farm_info.total_lp_share_weight.checked_add(u128::from(reward_weight)).unwrap();

            farm_info.stake_infos.insert(&sender_id, &stake_info);

            let lp_staked = self.lp_stake_info.get(&pool_id).unwrap_or(0) + u128::from(amount);
            self.lp_stake_info.insert(&pool_id, &lp_staked);
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
            stake_type,
            farm_id,
            lock_duration,
        } = near_sdk::serde_json::from_str(&msg).expect("Invalid PurchaseArgs");

        let ft_token_id = env::predecessor_account_id();
        assert!(amount.0 > 0, "Amount must be greater than 0");

        let swap_tokens = self.swap_farms.keys_as_vector().to_vec();
        let now = env::block_timestamp() / 1000000;

        if swap_tokens.contains(&ft_token_id) && stake_type == 1 {
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
                    created_at: now,
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
                .push(&(now + swap_farm_info.max_lock_time));
            swap_stake_info
                .claimed_token_at
                .push(&(now));

            swap_farm_info
                .stake_infos
                .insert(&sender_id, &swap_stake_info);
            self.swap_farms.insert(&ft_token_id, &swap_farm_info);
        } else {
            assert!(self.farm_infos.len() > farm_id.into(), "Invalid Farm ID");
            self.update_claim_amounts(farm_id, 0, now);
            let mut farm_info = self.farm_infos.get(farm_id.into()).unwrap();

            assert!(farm_info.farm_type == 0 || farm_info.farm_type == 2, "Token Pool does not exist in this Farm");

            if ft_token_id == farm_info.token_id {
                let info: Option<StakeInfo> = farm_info.stake_infos.get(&sender_id);
                let mut stake_info = if let Some(info) = info {
                    info
                } else {
                    let empty_value = StakeInfo {
                        owner_id: sender_id.clone(),
                        token_amount: 0,
                        lp_share_amount: 0,
                        reward_amount: 0,
                        token_weight: 0,
                        lp_share_weight: 0,
                        reward_token_to_claim: 0,
                        reward_lp_to_claim: 0,
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
                        reward_locked: Vector::new(StorageKey::LockInfos {
                            farm_id,
                            account_id: sender_id.clone(),
                            lock_info_type: 2,
                        }),
                        unlocked_at: Vector::new(StorageKey::LockInfos {
                            farm_id,
                            account_id: sender_id.clone(),
                            lock_info_type: 3,
                        }),
                        staking_duration: Vector::new(StorageKey::LockInfos {
                            farm_id,
                            account_id: sender_id.clone(),
                            lock_info_type: 4,
                        }),
                        created_at: now,
                        claimed_token_at: now,
                        claimed_lp_at: now,
                        claimed_reward_at: now,
                    };
                    empty_value
                };
                stake_info.token_locked.push(&amount);
                stake_info.lp_share_locked.push(&U128(0));
                stake_info.reward_locked.push(&U128(0));
                stake_info.staking_duration.push(&lock_duration);
                stake_info
                    .unlocked_at
                    .push(&(now + lock_duration));
                stake_info.claimed_token_at = now;
                stake_info.token_amount = stake_info.token_amount.checked_add(u128::from(amount)).unwrap();
                farm_info.total_token_amount = farm_info.total_token_amount.checked_add(u128::from(amount)).unwrap();
                let multiplier: u128 = 10;
                let reward_weight = u128::from(amount).checked_mul(100000000).unwrap().checked_div(farm_info.total_token_amount).unwrap().checked_mul(100000000000000000000).unwrap().checked_add(u128::from(amount).checked_mul(farm_info.token_weight_rate.into()).unwrap().checked_div(10000).unwrap().checked_div(u128::from(farm_info.max_token_vesting_duration)).unwrap().checked_mul(u128::from(lock_duration)).unwrap().checked_mul(multiplier.pow((24-farm_info.token_decimal).try_into().unwrap())).unwrap()).unwrap();
                stake_info.token_weight = stake_info.token_weight.checked_add(reward_weight).unwrap();
                farm_info.total_token_weight = farm_info.total_token_weight.checked_add(u128::from(reward_weight)).unwrap();
                farm_info.stake_infos.insert(&sender_id, &stake_info);
                self.farm_infos.replace(farm_id.into(), &farm_info);

                let token_staked = self.token_stake_info.get(&ft_token_id).unwrap_or(0) + u128::from(amount);
                self.token_stake_info.insert(&ft_token_id, &token_staked);
            }
        }
        PromiseOrValue::Value(U128(0))
    }
}
