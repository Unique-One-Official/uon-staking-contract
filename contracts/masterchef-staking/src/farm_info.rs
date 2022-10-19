use crate::*;

extern crate chrono;
// use chrono::prelude::*;
// use chrono::{DateTime, Local, NaiveDateTime, Utc};
// use std::time::SystemTime;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct FarmInfo {
    pub farm_id: u16, 
    pub farm_type: u8,
    pub token_id: AccountId,
    pub token_decimal: u8,
    pub pool_id: u64,
    // pub reward_token_id: AccountId,
    pub total_token_amount: u128,
    pub total_token_weight: u128,
    pub total_lp_share_amount: u128,
    pub total_lp_share_weight: u128,
    pub total_reward_amount: u128,
    pub total_reward_weight: u128,
    pub token_reward_rate: u128,
    pub pool_reward_rate: u128,
    pub reward_reward_rate: u128,
    pub token_weight_rate: u16,
    pub pool_weight_rate: u16,
    pub max_token_vesting_duration: u64,
    pub max_reward_vesting_duration: u64,
    pub starting_at: u64,
    pub ending_at: u64,
    pub stake_infos: UnorderedMap<AccountId, StakeInfo>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct EditingFarmInfo {
    pub farm_id: u16,
    pub farm_type: u8,
    pub token_id: AccountId,
    pub token_decimal: u8,
    pub pool_id: u64,
    // pub reward_token_id: AccountId,
    pub token_reward_rate: U128,
    pub pool_reward_rate: U128,
    pub reward_reward_rate: U128,
    pub token_weight_rate: u16,
    pub pool_weight_rate: u16,
    pub max_token_vesting_duration: u64,
    pub max_reward_vesting_duration: u64,
    pub starting_at: u64,
    pub ending_at: u64,
}

impl FarmInfo {
    pub fn new(
        farm_id: u16,
        farm_type: u8,
        token_id: AccountId,
        token_decimal: u8,
        pool_id: u64,
        token_reward_rate: U128,
        pool_reward_rate: U128,
        reward_reward_rate: U128,
        token_weight_rate: u16,
        pool_weight_rate: u16,
        max_token_vesting_duration: u64,
        max_reward_vesting_duration: u64,
        starting_at: u64,
        ending_at: u64,
    ) -> Self {
        Self {
            farm_id,
            farm_type,
            token_id,
            token_decimal,
            pool_id,
            total_token_amount: 0,
            total_token_weight: 0,
            total_lp_share_amount: 0,
            total_lp_share_weight: 0,
            total_reward_amount: 0,
            total_reward_weight: 0,
            token_reward_rate: u128::from(token_reward_rate),
            pool_reward_rate: u128::from(pool_reward_rate),
            reward_reward_rate: u128::from(reward_reward_rate),
            token_weight_rate,
            pool_weight_rate,
            max_token_vesting_duration,
            max_reward_vesting_duration,
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
        farm_type: u8,
        token_id: AccountId,
        token_decimal: u8,
        pool_id: u64,
        token_reward_rate: U128,
        pool_reward_rate: U128,
        reward_reward_rate: U128,
        token_weight_rate: u16,
        pool_weight_rate: u16,
        max_token_vesting_duration: u64,
        max_reward_vesting_duration: u64,
        starting_at: u64,
        ending_at: u64,
    ) {
        self.assert_owner();
        assert_one_yocto();
        self.farm_infos.push(&FarmInfo::new(
            self.farm_infos.len() as u16,
            farm_type,
            token_id,
            token_decimal,
            pool_id,
            token_reward_rate,
            pool_reward_rate,
            reward_reward_rate,
            token_weight_rate,
            pool_weight_rate,
            max_token_vesting_duration,
            max_reward_vesting_duration,
            starting_at,
            ending_at,
        ));
    }

    pub fn get_editing_farm_info(&self) -> EditingFarmInfo {
        // (&self.editing_new_farm).into()
        let data = EditingFarmInfo {
            farm_id: self.editing_new_farm.farm_id,
            farm_type: self.editing_new_farm.farm_type,
            token_id: self.editing_new_farm.token_id.clone(),
            token_decimal: self.editing_new_farm.token_decimal,
            pool_id: self.editing_new_farm.pool_id,
            token_reward_rate: self.editing_new_farm.token_reward_rate,
            pool_reward_rate: self.editing_new_farm.pool_reward_rate,
            reward_reward_rate: self.editing_new_farm.reward_reward_rate,
            token_weight_rate: self.editing_new_farm.token_weight_rate,
            pool_weight_rate: self.editing_new_farm.pool_weight_rate,
            max_token_vesting_duration: self.editing_new_farm.max_token_vesting_duration,
            max_reward_vesting_duration: self.editing_new_farm.max_reward_vesting_duration,
            starting_at: self.editing_new_farm.starting_at,
            ending_at: self.editing_new_farm.ending_at,
        };
        data
        // data
    }

    #[payable]
    pub fn save_editing_farm_info(
        &mut self,
        farm_type: u8,
        token_id: AccountId,
        token_decimal: u8,
        pool_id: u64,
        token_reward_rate: U128,
        pool_reward_rate: U128,
        reward_reward_rate: U128,
        token_weight_rate: u16,
        pool_weight_rate: u16,
        max_token_vesting_duration: u64,
        max_reward_vesting_duration: u64,
        starting_at: u64,
        ending_at: u64,
    ) {
        self.assert_admin();
        assert_one_yocto();
        for elem in self.farm_infos.to_vec() {
            if farm_type == 0 {
                if (elem.starting_at < starting_at && starting_at < elem.ending_at
                    || elem.starting_at < ending_at && ending_at < elem.ending_at
                    || starting_at < elem.starting_at && elem.starting_at < ending_at
                    || starting_at < elem.ending_at && elem.ending_at < ending_at) &&
                    elem.token_id == token_id && elem.farm_type == farm_type
                {
                    env::panic_str("Farm Time Overlaps. Plz input valid farm time");
                }
            } else if farm_type == 1 {
                if (elem.starting_at < starting_at && starting_at < elem.ending_at
                    || elem.starting_at < ending_at && ending_at < elem.ending_at
                    || starting_at < elem.starting_at && elem.starting_at < ending_at
                    || starting_at < elem.ending_at && elem.ending_at < ending_at) &&
                    elem.pool_id == pool_id && elem.farm_type == farm_type
                {
                    env::panic_str("Farm Time Overlaps. Plz input valid farm time");
                }
            } else if farm_type == 2 {
                if (elem.starting_at < starting_at && starting_at < elem.ending_at
                    || elem.starting_at < ending_at && ending_at < elem.ending_at
                    || starting_at < elem.starting_at && elem.starting_at < ending_at
                    || starting_at < elem.ending_at && elem.ending_at < ending_at) &&
                    elem.token_id == token_id && elem.pool_id == pool_id && elem.farm_type == farm_type
                {
                    env::panic_str("Farm Time Overlaps. Plz input valid farm time");
                }
            }
        }
        self.editing_new_farm = EditingFarmInfo {
            farm_id: 1,
            farm_type: farm_type,
            token_id: token_id,
            token_decimal: token_decimal,
            pool_id: pool_id,
            token_reward_rate: token_reward_rate,
            pool_reward_rate: pool_reward_rate,
            reward_reward_rate: reward_reward_rate,
            token_weight_rate: token_weight_rate,
            pool_weight_rate: pool_weight_rate,
            max_token_vesting_duration: max_token_vesting_duration,
            max_reward_vesting_duration: max_reward_vesting_duration,
            starting_at: starting_at,
            ending_at: ending_at,
        };
        self.confirmed_admins_for_new_farm.clear();
        self.confirmed_admins_for_new_farm
            .insert(&env::predecessor_account_id());

        if self.confirmed_admins_for_new_farm.len() == self.admin_ids.len() {
            self.farm_infos.push(&FarmInfo::new(
                self.farm_infos.len() as u16,
                self.editing_new_farm.farm_type,
                self.editing_new_farm.token_id.clone(),
                self.editing_new_farm.token_decimal,
                self.editing_new_farm.pool_id,
                self.editing_new_farm.token_reward_rate,
                self.editing_new_farm.pool_reward_rate,
                self.editing_new_farm.reward_reward_rate,
                self.editing_new_farm.token_weight_rate,
                self.editing_new_farm.pool_weight_rate,
                self.editing_new_farm.max_token_vesting_duration,
                self.editing_new_farm.max_reward_vesting_duration,
                self.editing_new_farm.starting_at,
                self.editing_new_farm.ending_at,
            ));
            self.confirmed_admins_for_new_farm.clear();
            self.editing_new_farm = EditingFarmInfo {
                farm_id: 0,
                farm_type: 0,
                token_id: AccountId::new_unchecked(TOKEN_CONTRACT_ID.to_string()),
                token_decimal: 0,
                pool_id: 0,
                token_reward_rate: U128(0),
                pool_reward_rate: U128(0),
                reward_reward_rate: U128(0),
                token_weight_rate: 0,
                pool_weight_rate: 0,
                max_token_vesting_duration: 0,
                max_reward_vesting_duration: 0,
                starting_at: 0,
                ending_at: 0,
            };
        }
    }

    #[payable]
    pub fn confirm_new_farm(&mut self) {
        self.assert_admin();
        assert_one_yocto();
        self.confirmed_admins_for_new_farm
            .insert(&env::predecessor_account_id());

        if self.confirmed_admins_for_new_farm.len() == self.admin_ids.len() {
            self.farm_infos.push(&FarmInfo::new(
                self.farm_infos.len() as u16,
                self.editing_new_farm.farm_type,
                self.editing_new_farm.token_id.clone(),
                self.editing_new_farm.token_decimal,
                self.editing_new_farm.pool_id,
                self.editing_new_farm.token_reward_rate,
                self.editing_new_farm.pool_reward_rate,
                self.editing_new_farm.reward_reward_rate,
                self.editing_new_farm.token_weight_rate,
                self.editing_new_farm.pool_weight_rate,
                self.editing_new_farm.max_token_vesting_duration,
                self.editing_new_farm.max_reward_vesting_duration,
                self.editing_new_farm.starting_at,
                self.editing_new_farm.ending_at,
            ));
            self.confirmed_admins_for_new_farm.clear();
            self.editing_new_farm = EditingFarmInfo {
                farm_id: 0,
                farm_type: 0,
                token_id: AccountId::new_unchecked(TOKEN_CONTRACT_ID.to_string()),
                token_decimal: 0,
                pool_id: 0,
                token_reward_rate: U128(0),
                pool_reward_rate: U128(0),
                reward_reward_rate: U128(0),
                token_weight_rate: 0,
                pool_weight_rate: 0,
                max_token_vesting_duration: 0,
                max_reward_vesting_duration: 0,
                starting_at: 0,
                ending_at: 0,
            };
        }
    }

    pub fn get_confirmed_admins_for_new_farm(&self) -> Vec<AccountId> {
        self.confirmed_admins_for_new_farm.to_vec()
    }
}
