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

    pub(crate) fn staking_informations_by_owner_id(
        &self,
        farm_id: u16,
        account_id: AccountId
    ) -> StakeInfo {
        assert!(self.farm_infos.len() > farm_id.into(), "Invalid Farm ID");
        let mut farm_info = self.farm_infos.get(farm_id.into()).unwrap();
        farm_info.stake_infos.get(&account_id).unwrap()
    }

    pub(crate) fn token_locked_amount(&self, farm_id: u16, account_id: AccountId) -> u128 {
        assert!(self.farm_infos.len() > farm_id.into(), "Invalid Farm ID");
        let farm_info = self.farm_infos.get(farm_id.into()).unwrap();
        let stake_info =
            self.staking_informations_by_owner_id(farm_id, account_id);
        let mut now = env::block_timestamp() / 1000000;

        let mut locked_amount:u128 = 0;

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
        let stake_info =
            self.staking_informations_by_owner_id(farm_id, account_id);
        let mut now = env::block_timestamp() / 1000000;

        let mut locked_amount:u128 = 0;

        let start: u64 = 0;
        let end = stake_info.unlocked_at.len();
        for i in start..end {
            if stake_info.unlocked_at.get(i).unwrap() > now {
                locked_amount += u128::from(stake_info.lp_share_locked.get(i).unwrap());
            }
        }

        locked_amount
    }

    pub(crate) fn claim_amount(&self, farm_id: u16, account_id: AccountId, token_type: u8) -> U128 {
        assert!(self.farm_infos.len() > farm_id.into(), "Invalid Farm ID");
        let farm_info = self.farm_infos.get(farm_id.into()).unwrap();
        let stake_info =
            self.staking_informations_by_owner_id(farm_id, account_id);
        let mut now = env::block_timestamp() / 1000000;

        if (now < farm_info.starting_at){
            return (U128(0));
        } else if (stake_info.claimed_token_at >= farm_info.ending_at && token_type == 0) {
            return (stake_info.reward_token_to_claim);
        } else if (stake_info.claimed_lp_at >= farm_info.ending_at && token_type == 1) {
            return (stake_info.reward_lp_to_claim);
        } else if (now > farm_info.ending_at) {
            now = farm_info.ending_at
        }

        let mut claim_amount = U128(0);
        
        if (stake_info.token_amount != U128(0) && token_type == 0) {
            if (farm_info.total_token_amount == U128(0)) {
                claim_amount = U128(0);
            } else {
                claim_amount = U128::from(u128::from(stake_info.reward_token_to_claim).checked_add(u128::from(stake_info.token_amount).checked_div(10000000000).unwrap().checked_mul(u128::from(farm_info.token_reward_rate)).unwrap().checked_div(u128::from(farm_info.total_token_amount)).unwrap().checked_mul(u128::from((now - stake_info.claimed_token_at)/1000)).unwrap().checked_mul(10000000000).unwrap()).unwrap());
            }
        }

        if (stake_info.lp_share_amount != U128(0) && token_type == 1) {
            if (farm_info.total_token_amount == U128(0)) {
                claim_amount = U128(0);
            } else {
                claim_amount = U128::from(u128::from(stake_info.reward_lp_to_claim).checked_add(u128::from(stake_info.lp_share_amount).checked_div(10000000000000000).unwrap().checked_mul(u128::from(farm_info.pool_reward_rate)).unwrap().checked_div(u128::from(farm_info.total_lp_share_amount)).unwrap().checked_mul(u128::from((now - stake_info.claimed_lp_at)/1000)).unwrap().checked_mul(10000000000000000).unwrap()).unwrap());
            }
        }

        claim_amount
    }

    pub(crate) fn update_claim_amounts(&mut self, farm_id: u16, token_type: u8) {
        let mut farm_info = self.farm_infos.get(farm_id.into()).unwrap();
        let keys_as_vector = farm_info.stake_infos.keys_as_vector();
        let keys_vec = keys_as_vector.to_vec();
        for key in keys_vec.iter() {
            let mut stake_info: StakeInfo = farm_info.stake_infos
                .get(&key)
                .unwrap();
            let now_time = env::block_timestamp() / 1000000;
            if stake_info.token_amount != U128(0) {
                stake_info.reward_token_to_claim = U128::from(u128::from(stake_info.reward_token_to_claim).checked_add(u128::from(stake_info.token_amount).checked_div(10000000000).unwrap().checked_mul(u128::from(farm_info.token_reward_rate)).unwrap().checked_div(u128::from(farm_info.total_token_amount)).unwrap().checked_mul(u128::from((now_time - stake_info.claimed_token_at)/1000)).unwrap().checked_mul(10000000000).unwrap()).unwrap());
            }
            if stake_info.lp_share_amount != U128(0) {
                stake_info.reward_lp_to_claim = U128::from(u128::from(stake_info.reward_lp_to_claim).checked_add(u128::from(stake_info.lp_share_amount).checked_div(10000000000000000).unwrap().checked_mul(u128::from(farm_info.pool_reward_rate)).unwrap().checked_div(u128::from(farm_info.total_lp_share_amount)).unwrap().checked_mul(u128::from((now_time - stake_info.claimed_lp_at)/1000)).unwrap().checked_mul(10000000000000000).unwrap()).unwrap());
            }
            stake_info.claimed_token_at = now_time;
            stake_info.claimed_lp_at = now_time;
            farm_info.stake_infos
                .insert(&key, &stake_info);
        }
        self.farm_infos.replace(farm_id.into(), &farm_info);
    }
}
