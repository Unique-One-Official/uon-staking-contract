use crate::*;

extern crate chrono;
// use chrono::prelude::*;
// use chrono::{DateTime, Local, NaiveDateTime, Utc};
// use std::time::SystemTime;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct SwapFarmInfo {
    pub total_token_amount: U128,
    pub max_lock_time: u64,
    pub min_lock_time: u64,
    pub token_decimal: u32,
    pub swap_rate: U128,
    pub stake_infos: UnorderedMap<AccountId, SwapStakeInfo>,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct EditingSwapFarmInfoData {
    pub swap_rate: U128,
    pub min_lock_time: u64,
    pub max_lock_time: u64,
    pub confirmed_admins: UnorderedSet<AccountId>,
}

impl EditingSwapFarmInfoData {
    pub fn new(token_id: AccountId, swap_rate: U128, min_lock_time: u64, max_lock_time: u64, type_id: u64) -> Self {
        Self {
            swap_rate: swap_rate,
            min_lock_time: min_lock_time,
            max_lock_time: max_lock_time,
            confirmed_admins: UnorderedSet::new(StorageKey::EditingSwapConfirmedAdmin {
                token_id: token_id,
                type_id: type_id,
            }),
        }
    }
}

impl SwapFarmInfo {
    pub fn new(token_id: AccountId, decimal: u32, min_lock_time: u64, max_lock_time: u64, swap_rate: U128) -> Self {
        Self {
            total_token_amount: U128(0),
            token_decimal: decimal,
            min_lock_time: min_lock_time,
            max_lock_time: max_lock_time,
            swap_rate: swap_rate,
            stake_infos: UnorderedMap::new(StorageKey::SwapStakeInfo {
                token_id: token_id.clone(),
            }),
        }
    }
}

#[near_bindgen]
impl Contract {
    // #[payable]
    // pub fn add_swapfarm(
    //     &mut self,
    //     token_id: AccountId,
    //     decimal: u32,
    //     min_lock_time: u64,
    //     max_lock_time: u64,
    //     swap_rate: U128,
    // ) {
    //     self.assert_owner();
    //     assert_one_yocto();
    //     self.swap_farms.insert(
    //         &token_id,
    //         &SwapFarmInfo::new(token_id.clone(), decimal, min_lock_time, max_lock_time, swap_rate),
    //     );
    // }

    #[payable]
    pub fn confirm_editing_swap_farm_info(&mut self, token_id: AccountId) {
        self.assert_admin();
        assert_one_yocto();
        let swap_tokens = self.editing_swap_farm_infos.keys_as_vector().to_vec();
        if swap_tokens.contains(&token_id) {
            let mut editing_swap_farm_data = self.editing_swap_farm_infos.get(&token_id).unwrap();
            editing_swap_farm_data
                .confirmed_admins
                .insert(&env::predecessor_account_id());
            if self.admin_ids.len() <= editing_swap_farm_data.confirmed_admins.len() {
                editing_swap_farm_data.confirmed_admins.clear();
                let mut swap_farm_data = self.swap_farms.get(&token_id).unwrap();
                swap_farm_data.min_lock_time = editing_swap_farm_data.min_lock_time;
                swap_farm_data.max_lock_time = editing_swap_farm_data.max_lock_time;
                swap_farm_data.swap_rate = editing_swap_farm_data.swap_rate;
                self.swap_farms.insert(&token_id, &swap_farm_data);
            }
            self.editing_swap_farm_infos
                .insert(&token_id, &editing_swap_farm_data);
        } else {
            env::panic_str("Invalid token id");
        }
    }

    #[payable]
    pub fn save_editing_swap_farm_info(
        &mut self,
        token_id: AccountId,
        swap_rate: U128,
        min_lock_time: u64,
        max_lock_time: u64,
    ) {
        self.assert_admin();
        assert_one_yocto();
        let swap_tokens = self.editing_swap_farm_infos.keys_as_vector().to_vec();
        if swap_tokens.contains(&token_id) {
            let mut editing_swap_farm_info = self.editing_swap_farm_infos.get(&token_id).unwrap();
            editing_swap_farm_info.min_lock_time = min_lock_time;
            editing_swap_farm_info.max_lock_time = max_lock_time;
            editing_swap_farm_info.swap_rate = swap_rate;
            editing_swap_farm_info.confirmed_admins.clear();
            editing_swap_farm_info
                .confirmed_admins
                .insert(&env::predecessor_account_id());

            self.editing_swap_farm_infos
                .insert(&token_id, &editing_swap_farm_info);

            if self.admin_ids.as_vector().len()
                == editing_swap_farm_info.confirmed_admins.as_vector().len()
            {
                let mut swap_farm_data = self.swap_farms.get(&token_id).unwrap();
                swap_farm_data.min_lock_time = min_lock_time;
                swap_farm_data.max_lock_time = max_lock_time;
                swap_farm_data.swap_rate = swap_rate;
                self.swap_farms.insert(&token_id, &swap_farm_data);
                editing_swap_farm_info.confirmed_admins.clear();
                self.editing_swap_farm_infos
                    .insert(&token_id, &editing_swap_farm_info);
            }
        } else {
            self.editing_swap_farm_infos.insert(
                &token_id,
                &EditingSwapFarmInfoData::new(
                    token_id.clone(),
                    swap_rate,
                    min_lock_time,
                    max_lock_time,
                    self.editing_swap_farm_infos.len(),
                ),
            );
        }
    }

    #[payable]
    pub fn save_swap_farm(
        &mut self,
        token_id: AccountId,
        swap_rate: U128,
        decimal: u32,
        min_lock_time: u64,
        max_lock_time: u64,
    ) {
        self.assert_owner();
        assert_one_yocto();
        let mut swap_tokens = self.swap_farms.keys_as_vector().to_vec();

        if swap_tokens.contains(&token_id) {
            let mut swap_farm_info = self.swap_farms.get(&token_id).unwrap();
            swap_farm_info.min_lock_time = min_lock_time;
            swap_farm_info.max_lock_time = max_lock_time;
            swap_farm_info.token_decimal = decimal;
            swap_farm_info.swap_rate = swap_rate;
            self.swap_farms.insert(&token_id, &swap_farm_info);
        } else {
            self.swap_farms.insert(
                &token_id,
                &SwapFarmInfo::new(token_id.clone(), decimal, min_lock_time, max_lock_time, swap_rate),
            );
        }

        swap_tokens = self.editing_swap_farm_infos.keys_as_vector().to_vec();
        if swap_tokens.contains(&token_id) {
            let mut editing_swap_farm_info = self.editing_swap_farm_infos.get(&token_id).unwrap();
            editing_swap_farm_info.min_lock_time = min_lock_time;            editing_swap_farm_info.max_lock_time = max_lock_time;
            editing_swap_farm_info.swap_rate = swap_rate;
            editing_swap_farm_info.confirmed_admins.clear();
            self.editing_swap_farm_infos
                .insert(&token_id, &editing_swap_farm_info);
        } else {
            self.editing_swap_farm_infos.insert(
                &token_id,
                &EditingSwapFarmInfoData::new(
                    token_id.clone(),
                    swap_rate,
                    min_lock_time,
                    max_lock_time,
                    self.editing_swap_farm_infos.len(),
                ),
            );
        }
    }

    #[payable]
    pub fn remove_swap_farm(&mut self, token_id: AccountId) {
        self.assert_owner();
        assert_one_yocto();
        self.swap_farms.remove(&token_id);
        self.editing_swap_farm_infos.remove(&token_id);
    }

    #[payable]
    pub fn unstake_swap(
        &mut self,
        token_id: AccountId,
        account_id: AccountId,
        id: u64,
        amount_2_unstake: U128,
    ) {
        let amount = self.get_swap_unstake_amount_by_id(token_id.clone(), account_id.clone(), id);
        assert!(
            u128::from(amount) >= u128::from(amount_2_unstake),
            "Invalid UNET unstake amount"
        );

        let swap_farm_data = self.swap_farms.get(&token_id.clone()).unwrap();
        let mut stake_info = swap_farm_data.stake_infos.get(&account_id.clone()).unwrap();
        stake_info.token_amount = U128::from(
            u128::from(stake_info.token_amount)
                .checked_sub(u128::from(amount_2_unstake))
                .unwrap(),
        );
        stake_info.token_locked.replace(
            id,
            &U128::from(
                u128::from(stake_info.token_locked.get(id).unwrap())
                    .checked_sub(u128::from(amount_2_unstake))
                    .unwrap(),
            ),
        );
        stake_info
            .claimed_token_at
            .replace(id, &(env::block_timestamp() / 1000000));

        // ext_transfer::ft_transfer(
        //     account_id.clone().try_into().unwrap(),
        //     amount_2_unstake,
        //     None,
        //     &AccountId::new_unchecked(TOKEN_CONTRACT_ID.to_string()),
        //     1,
        //     GAS_FOR_FT_TRANSFER,
        // );
        ext_transfer::ext(AccountId::new_unchecked(TOKEN_CONTRACT_ID.to_string()))
            .with_attached_deposit(1)
            .with_static_gas(GAS_FOR_FT_TRANSFER)
            .ft_transfer(
                account_id.clone().try_into().unwrap(),
                amount_2_unstake,
                None,
            );
    }

    pub fn get_swap_unstake_amount(&self, token_id: AccountId, account_id: AccountId) -> Vec<U128> {
        let swap_farm_data = self.swap_farms.get(&token_id).unwrap();
        let stake_info = swap_farm_data.stake_infos.get(&account_id).unwrap();
        let mut unstake_amount = vec![];
        let start_idx: u64 = 0;
        let end_idx = stake_info.token_locked.len();
        for idx in start_idx..end_idx {
            if stake_info.unlocked_at.get(idx).unwrap() < env::block_timestamp() / 1000000 {
                unstake_amount.push(stake_info.token_locked.get(idx).unwrap());
            } else if u64::from(stake_info.unlocked_at.get(idx).unwrap())
                .checked_sub(swap_farm_data.max_lock_time)
                .unwrap()
                .checked_add(u64::from(swap_farm_data.min_lock_time))
                .unwrap()
                > env::block_timestamp() / 1000000
            {
                unstake_amount.push(U128(0));
            } else if stake_info.unlocked_at.get(idx).unwrap() >= env::block_timestamp() / 1000000 {
                unstake_amount.push(U128::from(
                    u128::from(stake_info.token_locked.get(idx).unwrap())
                        .checked_div(10000000000)
                        .unwrap()
                        .checked_mul(u128::from(
                            u128::from(env::block_timestamp() / 1000000)
                                .checked_sub(u128::from(
                                    stake_info.claimed_token_at.get(idx).unwrap(),
                                ))
                                .unwrap(),
                        ))
                        .unwrap()
                        .checked_div(u128::from(
                            u128::from(stake_info.unlocked_at.get(idx).unwrap())
                                .checked_sub(u128::from(
                                    stake_info.claimed_token_at.get(idx).unwrap(),
                                ))
                                .unwrap(),
                        ))
                        .unwrap()
                        .checked_mul(10000000000)
                        .unwrap(),
                ));
            }
        }
        unstake_amount
    }

    pub fn get_swap_unstake_amount_by_id(
        &self,
        token_id: AccountId,
        account_id: AccountId,
        id: u64,
    ) -> U128 {
        let swap_farm_data = self.swap_farms.get(&token_id).unwrap();
        let stake_info = swap_farm_data.stake_infos.get(&account_id).unwrap();
        assert!(stake_info.token_locked.len() >= id, "Invalid staking id");
        let mut unstake_amount = U128(0);
        if stake_info.unlocked_at.get(id).unwrap() < env::block_timestamp() / 1000000 {
            unstake_amount = U128::from(u128::from(stake_info.token_locked.get(id).unwrap()));
        } else if u64::from(stake_info.unlocked_at.get(id).unwrap())
            .checked_sub(swap_farm_data.max_lock_time)
            .unwrap()
            .checked_add(u64::from(swap_farm_data.min_lock_time))
            .unwrap()
            > env::block_timestamp() / 1000000
        {
            unstake_amount = U128(0);
        } else {
            unstake_amount = U128::from(
                u128::from(stake_info.token_locked.get(id).unwrap())
                    .checked_div(10000000000)
                    .unwrap()
                    .checked_mul(u128::from(
                        u128::from(env::block_timestamp() / 1000000)
                            .checked_sub(u128::from(stake_info.claimed_token_at.get(id).unwrap()))
                            .unwrap(),
                    ))
                    .unwrap()
                    .checked_div(u128::from(
                        u128::from(stake_info.unlocked_at.get(id).unwrap())
                            .checked_sub(u128::from(stake_info.claimed_token_at.get(id).unwrap()))
                            .unwrap(),
                    ))
                    .unwrap()
                    .checked_mul(10000000000)
                    .unwrap(),
            );
        };
        unstake_amount
    }

}
