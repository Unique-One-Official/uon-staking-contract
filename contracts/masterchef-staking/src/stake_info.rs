use crate::*;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct StakeInfo {
    pub owner_id: AccountId,
    pub token_amount: u128,
    pub lp_share_amount: u128,
    pub reward_amount: u128,
    pub token_weight: u128,
    pub lp_share_weight: u128,
    pub reward_token_to_claim: u128,
    pub reward_lp_to_claim: u128,
    pub token_locked: Vector<U128>,
    pub lp_share_locked: Vector<U128>,
    pub reward_locked: Vector<U128>,
    pub unlocked_at: Vector<u64>,
    pub staking_duration: Vector<u64>,
    pub created_at: u64,
    pub claimed_token_at: u64,
    pub claimed_lp_at: u64,
    pub claimed_reward_at: u64,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct SwapStakeInfo {
    pub owner_id: AccountId,
    pub token_amount: U128,
    pub token_locked: Vector<U128>,
    pub unlocked_at: Vector<u64>,
    pub created_at: u64,
    pub claimed_token_at: Vector<u64>,
}

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn claim_reward(&mut self, farm_id: u16, token_type: u8) {
        assert_one_yocto();
        let account_id = env::predecessor_account_id();
        let mut claim_amount = 0;
        let now = env::block_timestamp() / 1000000;

        let mut farm_info = self.farm_infos.get(farm_id.into()).unwrap();
        if token_type == 0 && farm_info.farm_type == 1 {
            env::panic_str("Token Pool does not exist in this Farm");
        } else if token_type == 1 && farm_info.farm_type == 0 {
            env::panic_str("LP Pool does not exist in this Farm");
        }

        if token_type == 0 {
            self.update_claim_amounts(farm_id, 0, now);
            self.update_claim_amounts(farm_id, 2, now);
        }
        else if token_type == 1 {
            self.update_claim_amounts(farm_id, 1, now);
            self.update_claim_amounts(farm_id, 2, now);
        } else if token_type == 2 {
            self.update_claim_amounts(farm_id, 2, now);
            return;
        }
        farm_info = self.farm_infos.get(farm_id.into()).unwrap();
        let mut stake_info = farm_info.stake_infos.get(&account_id).unwrap();
        let total_reward_amount = farm_info.total_reward_amount;


        if token_type == 0 {
            claim_amount = self.claim_amount(farm_id, account_id.clone(), 0, now);
            stake_info.reward_token_to_claim = 0;
            stake_info.claimed_token_at = now;
            stake_info.token_locked.push(&U128(0));
            stake_info.lp_share_locked.push(&U128(0));
            stake_info.reward_locked.push(&U128(claim_amount));
            stake_info
                .unlocked_at
                .push(&(now + farm_info.max_reward_vesting_duration));
            stake_info.staking_duration.push(&farm_info.max_reward_vesting_duration);

            stake_info.reward_amount = stake_info.reward_amount.checked_add(claim_amount)
                    .unwrap();
            farm_info.total_reward_amount = farm_info.total_reward_amount.checked_add(claim_amount).unwrap();
        }
        else if token_type == 1 {
            claim_amount = self.claim_amount(farm_id, account_id.clone(), 1, now);
            stake_info.reward_lp_to_claim = 0;
            stake_info.claimed_lp_at = now;
            stake_info.token_locked.push(&U128(0));
            stake_info.lp_share_locked.push(&U128(0));
            stake_info.reward_locked.push(&U128(claim_amount));
            stake_info
                .unlocked_at
                .push(&(now + farm_info.max_reward_vesting_duration));
            stake_info.staking_duration.push(&farm_info.max_reward_vesting_duration);

            stake_info.reward_amount = stake_info.reward_amount.checked_add(claim_amount)
                    .unwrap();
            farm_info.total_reward_amount = farm_info.total_reward_amount.checked_add(claim_amount).unwrap();
        }

        farm_info.stake_infos.insert(&account_id, &stake_info);
        self.farm_infos.replace(farm_id.into(), &farm_info);
    }

    #[payable]
    pub fn unstake(&mut self, farm_id: u16, token_type: u8, amount: U128) {
        assert_one_yocto();
        let account_id = env::predecessor_account_id();
        assert!(self.farm_infos.len() > farm_id.into(), "Invalid Farm ID");
        let mut farm_info = self.farm_infos.get(farm_id.into()).unwrap();

        let mut stake_info = farm_info.stake_infos.get(&account_id).unwrap();

        if token_type == 0 && farm_info.farm_type == 1 {
            env::panic_str("Token Pool does not exist in this Farm");
        } else if token_type == 1 && farm_info.farm_type == 0 {
            env::panic_str("LP Pool does not exist in this Farm");
        }

        if token_type == 0 {
            assert!(
                amount.0
                    <= (stake_info.token_amount
                        - self.token_locked_amount(farm_id, account_id.clone())),
                "Amount must be smaller than available amount"
            );
        } else if token_type == 1 {
            assert!(
                amount.0
                    <= (stake_info.lp_share_amount
                        - self.lp_locked_amount(farm_id, account_id.clone())),
                "Amount must be smaller than available amount"
            );
        } else if token_type == 2 {
            assert!(
                amount.0
                    <= (stake_info.reward_amount
                        - self.reward_locked_amount(farm_id, account_id.clone())),
                "Amount must be smaller than available amount"
            );
        }

        let now = env::block_timestamp() / 1000000;

        self.update_claim_amounts(farm_id, token_type, now);

        farm_info = self.farm_infos.get(farm_id.into()).unwrap();
        stake_info = farm_info.stake_infos.get(&account_id).unwrap();

        if token_type == 0 {
            ext_transfer::ext(AccountId::new_unchecked(farm_info.token_id.to_string()))
                .with_attached_deposit(1)
                .with_static_gas(GAS_FOR_FT_TRANSFER.into())
                .ft_transfer(
                    account_id.clone().try_into().unwrap(), 
                    amount,
                    None,
                );
            let multiplier: u128 = 10;
            let reward_weight = stake_info.token_weight
                .checked_div(10_000_000_000_000_000).unwrap()
                .checked_mul(u128::from(amount)).unwrap()
                .checked_div(stake_info.token_amount).unwrap()
                .checked_mul(10_000_000_000_000_000).unwrap();
            stake_info.token_weight = stake_info.token_weight.checked_sub(reward_weight).unwrap();
            farm_info.total_token_weight = farm_info.total_token_weight.checked_sub(reward_weight).unwrap();
            stake_info.token_amount = stake_info.token_amount.checked_sub(u128::from(amount))
                    .unwrap();
            farm_info.total_token_amount = farm_info.total_token_amount.checked_sub(u128::from(amount)).unwrap();
        } else if token_type == 1 {
            let reward_weight = stake_info.lp_share_weight.checked_div(10_000_000_000_000_000).unwrap().checked_mul(u128::from(amount)).unwrap().checked_div(stake_info.lp_share_amount).unwrap().checked_mul(10_000_000_000_000_000).unwrap();
            stake_info.lp_share_weight = stake_info.lp_share_weight.checked_sub(reward_weight).unwrap();
            farm_info.total_lp_share_weight = farm_info.total_lp_share_weight.checked_sub(reward_weight).unwrap();
            stake_info.lp_share_amount = stake_info.lp_share_amount.checked_sub(u128::from(amount)).unwrap();
            farm_info.total_lp_share_amount = farm_info.total_lp_share_amount.checked_sub(u128::from(amount)).unwrap();
            ext_transfer::ext(AccountId::new_unchecked(REF_CONTRACT_ID.to_string()))
            .with_attached_deposit(1)
            .with_static_gas(GAS_FOR_FT_TRANSFER.into())
            .mft_transfer(
                ":".to_owned() + &farm_info.pool_id.to_string(),
                account_id.clone().try_into().unwrap(),
                amount,
                None,
            );
        } else if token_type == 2 {
            ext_transfer::ext(AccountId::new_unchecked(farm_info.token_id.to_string()))
                .with_attached_deposit(1)
                .with_static_gas(GAS_FOR_FT_TRANSFER.into())
                .ft_transfer(
                    account_id.clone().try_into().unwrap(), 
                    amount,
                    None,
                );
            stake_info.reward_amount = stake_info.reward_amount.checked_sub(u128::from(amount)).unwrap();
            farm_info.total_reward_amount = farm_info.total_reward_amount.checked_sub(u128::from(amount)).unwrap();
        }
        farm_info.stake_infos.insert(&account_id, &stake_info);
        self.farm_infos.replace(farm_id.into(), &farm_info);
    }
}
