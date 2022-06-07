use crate::*;

extern crate chrono;
// use chrono::prelude::*;
// use chrono::{DateTime, Local, NaiveDateTime, Utc};
// use std::time::SystemTime;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct FarmInfo {
    pub farm_id: u16,
    pub token_id: AccountId,
    pub pool_id: u64,
    pub reward_token_id: AccountId,
    pub total_token_amount: U128,
    pub total_lp_share_amount: U128,
    pub token_reward_rate: U128,
    pub pool_reward_rate: U128,
    pub starting_at: u64,
    pub ending_at: u64,
    pub stake_infos: UnorderedMap<AccountId, StakeInfo>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct EditingFarmInfo {
    pub farm_id: u16,
    pub token_id: AccountId,
    pub pool_id: u64,
    pub reward_token_id: AccountId,
    pub token_reward_rate: U128,
    pub pool_reward_rate: U128,
    pub starting_at: u64,
    pub ending_at: u64,
}

impl FarmInfo {
    pub fn new(
        farm_id: u16,
        token_id: AccountId,
        pool_id: u64,
        reward_token_id: AccountId,
        token_reward_rate: U128,
        pool_reward_rate: U128,
        starting_at: u64,
        ending_at: u64,
    ) -> Self {
        Self {
            farm_id,
            token_id,
            pool_id,
            reward_token_id,
            total_token_amount: U128(0),
            total_lp_share_amount: U128(0),
            token_reward_rate,
            pool_reward_rate,
            starting_at,
            ending_at,
            stake_infos: UnorderedMap::new(StorageKey::StakeInfos { farm_id }),
        }
    }
}

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn add_farm(
        &mut self,
        token_id: AccountId,
        pool_id: u64,
        reward_token_id: AccountId,
        token_reward_rate: U128,
        pool_reward_rate: U128,
        starting_at: u64,
        ending_at: u64,
    ) {
        self.assert_owner();
        assert_one_yocto();
        self.farm_infos.push(&FarmInfo::new(
            self.farm_infos.len() as u16,
            token_id,
            pool_id,
            reward_token_id,
            token_reward_rate,
            pool_reward_rate,
            starting_at,
            ending_at,
        ));
    }
}
