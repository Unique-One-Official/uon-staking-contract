use crate::*;

// You can define internal functions here you can see here all the functions are declared like "pub(crate)"

impl Contract {
    pub(crate) fn assert_owner(&self) {
        assert_eq!(
            &env::predecessor_account_id(),
            &self.owner_id,
            "Owner's method"
        );
    }

    pub(crate) fn assert_admin(&self) {
        assert!(
            self.admin_ids.contains(&env::predecessor_account_id()),
            "Admin's method"
        );
    }

    pub(crate) fn staking_informations_by_owner_id(
        &self,
        farm_id: u16,
        account_id: AccountId,
    ) -> StakeInfo {
        assert!(self.farm_infos.len() > farm_id.into(), "Invalid Farm ID");
        let farm_info = self.farm_infos.get(farm_id.into()).unwrap();
        farm_info.stake_infos.get(&account_id).unwrap()
    }

    pub(crate) fn token_locked_amount(&self, farm_id: u16, account_id: AccountId) -> u128 {
        assert!(self.farm_infos.len() > farm_id.into(), "Invalid Farm ID");
        let farm_info = self.farm_infos.get(farm_id.into()).unwrap();
        if farm_info.farm_type == 1 {
            return 0;
        }
        let stake_info = self.staking_informations_by_owner_id(farm_id, account_id);
        let now = env::block_timestamp() / 1000000;

        let mut locked_amount: u128 = 0;

        let start: u64 = 0;
        let end = stake_info.unlocked_at.len();
        for i in start..end {
            if stake_info.unlocked_at.get(i).unwrap() > now {
                locked_amount += u128::from(stake_info.token_locked.get(i).unwrap());
            }
        }

        locked_amount
    }

    pub(crate) fn lp_locked_amount(&self, farm_id: u16, account_id: AccountId) -> u128 {
        assert!(self.farm_infos.len() > farm_id.into(), "Invalid Farm ID");
        let farm_info = self.farm_infos.get(farm_id.into()).unwrap();
        if farm_info.farm_type == 0 {
            return 0;
        }
        let stake_info = self.staking_informations_by_owner_id(farm_id, account_id);
        let now = env::block_timestamp() / 1000000;

        let mut locked_amount: u128 = 0;

        let start: u64 = 0;
        let end = stake_info.unlocked_at.len();
        for i in start..end {
            if stake_info.unlocked_at.get(i).unwrap() > now {
                locked_amount += u128::from(stake_info.lp_share_locked.get(i).unwrap());
            }
        }

        locked_amount
    }

    pub(crate) fn reward_locked_amount(&self, farm_id: u16, account_id: AccountId) -> u128 {
        assert!(self.farm_infos.len() > farm_id.into(), "Invalid Farm ID");
        let farm_info = self.farm_infos.get(farm_id.into()).unwrap();
        let stake_info = self.staking_informations_by_owner_id(farm_id, account_id);
        let now = env::block_timestamp() / 1000000;

        let mut locked_amount: u128 = 0;

        let start: u64 = 0;
        let end = stake_info.unlocked_at.len();
        for i in start..end {
            if stake_info.unlocked_at.get(i).unwrap() > now {
                locked_amount += u128::from(stake_info.reward_locked.get(i).unwrap());
            }
        }

        locked_amount
    }

    pub(crate) fn claim_amount(&self, farm_id: u16, account_id: AccountId, token_type: u8, time: u64) -> u128 {
        assert!(self.farm_infos.len() > farm_id.into(), "Invalid Farm ID");
        let farm_info = self.farm_infos.get(farm_id.into()).unwrap();
        let stake_info = self.staking_informations_by_owner_id(farm_id, account_id);
        let mut now = time;

        if now < farm_info.starting_at {
            return 0;
        } else if now > farm_info.ending_at {
            now = farm_info.ending_at;
        }

        if token_type == 0 && farm_info.farm_type == 1 {
            return 0;
        } else if token_type == 1 && farm_info.farm_type == 0 {
            return 0;
        }
        let mut claim_amount = 0;
        if  token_type == 0 {
            if (now / 1000) <= (stake_info.claimed_token_at / 1000) || stake_info.token_amount != 0 {
                claim_amount = stake_info.reward_token_to_claim;
            } else {
                claim_amount = stake_info.reward_token_to_claim.checked_add(
                    stake_info.token_weight
                        .checked_div(10000000000)
                        .unwrap()
                        .checked_mul(farm_info.token_reward_rate)
                        .unwrap()
                        .checked_div(farm_info.total_token_weight)
                        .unwrap()
                        .checked_mul(((now / 1000) - (stake_info.claimed_token_at / 1000)).try_into().unwrap())
                        .unwrap()
                        .checked_mul(10000000000)
                        .unwrap(),
                )
                .unwrap();
            }
        }

        if  token_type == 1 {
            if (now / 1000) <= (stake_info.claimed_lp_at / 1000) || stake_info.lp_share_amount != 0{
                claim_amount = stake_info.reward_lp_to_claim;
            } else {
                claim_amount = stake_info.reward_lp_to_claim.checked_add(
                    stake_info.lp_share_weight
                        .checked_div(1000000000000)
                        .unwrap()
                        .checked_mul(farm_info.pool_reward_rate)
                        .unwrap()
                        .checked_div(farm_info.total_lp_share_weight)
                        .unwrap()
                        .checked_mul(((now / 1000) - (stake_info.claimed_lp_at / 1000)).try_into().unwrap())
                        .unwrap()
                        .checked_mul(1000000000000)
                        .unwrap(),
                )
                .unwrap();
            }
        }

        if stake_info.reward_amount != 0 && token_type == 2 {
            for index in 0..stake_info.unlocked_at.len() {
                if stake_info.reward_locked.get(index).unwrap() == U128(0) {
                    continue;
                } else {
                    let mut last_time = now;
                    if (stake_info.unlocked_at.get(index).unwrap() / 1000) < (now / 1000) {
                        last_time = stake_info.unlocked_at.get(index).unwrap();
                    }
                    if (last_time / 1000) <= (stake_info.claimed_reward_at / 1000) {
                        continue;
                    }

                    let mut reward_amount: u128 = 0;
                    reward_amount = u128::from(stake_info.reward_locked.get(index).unwrap())
                    .checked_div(10000000000)
                    .unwrap()
                    .checked_mul(farm_info.reward_reward_rate)
                    .unwrap()
                    .checked_div(farm_info.total_reward_amount)
                    .unwrap()
                    .checked_mul(((last_time / 1000) - (stake_info.claimed_reward_at / 1000)).try_into().unwrap())
                    .unwrap()
                    .checked_mul(10000000000)
                    .unwrap();
                    claim_amount = claim_amount.checked_add(reward_amount).unwrap();
                }
            }
        }

        claim_amount
    }

    pub(crate) fn update_claim_amounts(&mut self, farm_id: u16, token_type: u8, now: u64) {
        let mut farm_info = self.farm_infos.get(farm_id.into()).unwrap();
        if token_type == 0 && farm_info.farm_type == 1 {
            return;
        } else if token_type == 1 && farm_info.farm_type == 0 {
            return;
        }
        let keys_as_vector = farm_info.stake_infos.keys_as_vector();
        let keys_vec = keys_as_vector.to_vec();
        let total_reward_amount = farm_info.total_reward_amount;
        for key in keys_vec.iter() {
            let mut stake_info: StakeInfo = farm_info.stake_infos.get(&key).unwrap();
            let mut now_time = now;
            if token_type != 2 {
                if now_time < farm_info.starting_at {
                    return;
                } else if now_time > farm_info.ending_at {
                    now_time = farm_info.ending_at;
                }
            }
    
            if stake_info.token_amount != 0 && token_type == 0 {
                if (now_time / 1000) > (stake_info.claimed_token_at / 1000) {
                    stake_info.reward_token_to_claim = stake_info.reward_token_to_claim.checked_add(
                        stake_info.token_weight
                            .checked_div(10000000000)
                            .unwrap()
                            .checked_mul(farm_info.token_reward_rate)
                            .unwrap()
                            .checked_div(farm_info.total_token_weight)
                            .unwrap()
                            .checked_mul(((now_time / 1000) - (stake_info.claimed_token_at / 1000)).try_into().unwrap())
                            .unwrap()
                            .checked_mul(10000000000)
                            .unwrap(),
                    ).unwrap();
                    stake_info.claimed_token_at = now_time;
                }
            }
            if stake_info.lp_share_amount != 0 && token_type == 1 {
                if (now_time / 1000) > (stake_info.claimed_lp_at / 1000) {
                    stake_info.reward_lp_to_claim = stake_info.reward_lp_to_claim.checked_add(
                        stake_info.lp_share_weight
                            .checked_div(1000000000000)
                            .unwrap()
                            .checked_mul(farm_info.pool_reward_rate)
                            .unwrap()
                            .checked_div(farm_info.total_lp_share_weight)
                            .unwrap()
                            .checked_mul(((now_time / 1000) - (stake_info.claimed_lp_at / 1000)).try_into().unwrap())
                            .unwrap()
                            .checked_mul(1000000000000)
                            .unwrap(),
                    ).unwrap();
                    stake_info.claimed_lp_at = now_time;
                }
            }
            if stake_info.reward_amount != 0 && token_type == 2 {
                let mut sum_reward_amount: u128 = 0;
                for index in 0..stake_info.unlocked_at.len() {
                    if stake_info.reward_locked.get(index).unwrap() == U128(0) {
                        continue;
                    } else {
                        let mut last_time = now_time;
                        if (stake_info.unlocked_at.get(index).unwrap() / 1000) < (now_time / 1000) {
                            last_time = stake_info.unlocked_at.get(index).unwrap();
                        }
                        if (last_time / 1000) <= (stake_info.claimed_reward_at / 1000) {
                            continue;
                        }

                        let mut reward_amount: u128 = 0;
                        reward_amount = u128::from(stake_info.reward_locked.get(index).unwrap())
                        .checked_div(10000000000)
                        .unwrap()
                        .checked_mul(farm_info.reward_reward_rate)
                        .unwrap()
                        .checked_div(total_reward_amount)
                        .unwrap()
                        .checked_mul(((last_time / 1000) - (stake_info.claimed_reward_at / 1000)).try_into().unwrap())
                        .unwrap()
                        .checked_mul(10000000000)
                        .unwrap();
                        sum_reward_amount = sum_reward_amount.checked_add(reward_amount).unwrap();
                        stake_info.reward_locked.replace(index, &U128::from(u128::from(stake_info.reward_locked.get(index).unwrap()).checked_add(reward_amount).unwrap()));
                    }
                }
                stake_info.reward_amount = stake_info.reward_amount.checked_add(sum_reward_amount).unwrap();
                farm_info.total_reward_amount = farm_info.total_reward_amount.checked_add(sum_reward_amount).unwrap();
                stake_info.claimed_reward_at = now_time;
            }
            farm_info.stake_infos.insert(&key, &stake_info);
        }
        self.farm_infos.replace(farm_id.into(), &farm_info);
    }
}
