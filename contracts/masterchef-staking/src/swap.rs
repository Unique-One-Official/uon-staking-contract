use crate::*;

extern crate chrono;
// use chrono::prelude::*;
// use chrono::{DateTime, Local, NaiveDateTime, Utc};
// use std::time::SystemTime;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct SwapFarmInfo {
    pub total_token_amount: U128,
    pub min_lock_time: u64,
    pub token_decimal: u32,
    pub swap_rate: U128,
    pub stake_infos: UnorderedMap<AccountId, SwapStakeInfo>,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct EditingSwapFarmInfoData {
    pub swap_rate: U128,
    pub min_lock_time: u64,
    pub confirmed_admins: UnorderedSet<AccountId>,
}

impl EditingSwapFarmInfoData {
    pub fn new(token_id: AccountId, swap_rate: U128, lock_time: u64, type_id: u64) -> Self {
        Self {
            swap_rate: swap_rate,
            min_lock_time: lock_time,
            confirmed_admins: UnorderedSet::new(StorageKey::EditingSwapConfirmedAdmin {
                token_id: token_id,
                type_id: type_id,
            }),
        }
    }
}

impl SwapFarmInfo {
    pub fn new(token_id: AccountId, decimal: u32, lock_time: u64, swap_rate: U128) -> Self {
        Self {
            total_token_amount: U128(0),
            token_decimal: decimal,
            min_lock_time: lock_time,
            swap_rate: swap_rate,
            stake_infos: UnorderedMap::new(StorageKey::SwapStakeInfo {
                token_id: token_id.clone(),
            }),
        }
    }
}

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn add_swapfarm(
        &mut self,
        token_id: AccountId,
        decimal: u32,
        lock_time: u64,
        swap_rate: U128,
    ) {
        self.assert_owner();
        assert_one_yocto();
        self.swap_farms.insert(
            &token_id,
            &SwapFarmInfo::new(token_id.clone(), decimal, lock_time, swap_rate),
        );
    }
}
