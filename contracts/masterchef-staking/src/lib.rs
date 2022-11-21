use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap, UnorderedSet, Vector};
use near_sdk::json_types::{U128, U64};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    assert_one_yocto, env, ext_contract, near_bindgen, AccountId, Balance, BorshStorageKey, Gas, PanicOnDefault, Promise, PromiseOrValue
};

use std::convert::TryInto;
use std::cmp::min;

use crate::external::*;
use crate::farm_info::*;
use crate::stake_info::*;
use crate::swap::*;

use near_sdk::env::STORAGE_PRICE_PER_BYTE;

mod external;
mod farm_info;
mod ft_callbacks;
mod internal;
mod stake_info;
mod swap;
mod view;

// TODO check seller supports storage_deposit at ft_token_id they want to post sale in

const GAS_FOR_FT_TRANSFER: Gas = Gas(5_000_000_000_000);
const STORAGE_PER_SALE: u128 = 1000 * STORAGE_PRICE_PER_BYTE;
const TOKEN_CONTRACT_ID: &str = "unet.testnet";
const REF_CONTRACT_ID: &str = "ref-finance-101.testnet";

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct StorageBalanceBounds {
    pub min: U128,
    pub max: Option<U128>,
}

// #[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
// pub struct ContractV1 {
//     pub owner_id: AccountId,
//     pub farm_infos: Vector<FarmInfo>,
//     pub storage_deposits: LookupMap<AccountId, Balance>,
//     pub admin_ids: UnorderedSet<AccountId>,
//     pub editing_new_farm: EditingFarmInfo,
//     pub confirmed_admins_for_new_farm: UnorderedSet<AccountId>,
//     pub swap_rate_info: UnorderedMap<AccountId, U128>,
//     pub swap_farms: UnorderedMap<AccountId, SwapFarmInfo>,
// }

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub owner_id: AccountId,
    pub farm_infos: Vector<FarmInfo>,
    pub storage_deposits: LookupMap<AccountId, Balance>,
    pub admin_ids: UnorderedSet<AccountId>,
    pub editing_new_farm: EditingFarmInfo,
    pub confirmed_admins_for_new_farm: UnorderedSet<AccountId>,
    pub swap_farms: UnorderedMap<AccountId, SwapFarmInfo>,
    pub editing_swap_farm_infos: UnorderedMap<AccountId, EditingSwapFarmInfoData>,
    pub token_stake_info: LookupMap<AccountId, u128>,
    pub lp_stake_info: LookupMap<u64, u128>,
}

/// Helper structure to for keys of the persistent collections.
#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKey {
    FarmInfos,
    StakeInfos {
        farm_id: u16,
    },
    LockInfos {
        farm_id: u16,
        account_id: AccountId,
        lock_info_type: u8,
    },
    StorageDeposits,
    AdminIds,
    EditingNewFarm,
    ConfirmedAdminsForNewFarm,
    SwapFarms,
    SwapStakeInfo {
        token_id: AccountId,
    },
    SwapStakeLockInfos {
        token_id: AccountId,
        account_id: AccountId,
        lock_info_type: u8,
    },
    EditingSwapFarmInfo,
    EditingSwapConfirmedAdmin {
        token_id: AccountId,
        type_id: u64,
    },
    TokenStakeInfo,
    LPStakeInfo,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        let this = Self {
            owner_id: owner_id.into(),
            farm_infos: Vector::new(StorageKey::FarmInfos),
            storage_deposits: LookupMap::new(StorageKey::StorageDeposits),
            admin_ids: UnorderedSet::new(StorageKey::AdminIds),
            editing_new_farm: EditingFarmInfo {
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
            },
            confirmed_admins_for_new_farm: UnorderedSet::new(StorageKey::ConfirmedAdminsForNewFarm),
            swap_farms: UnorderedMap::new(StorageKey::SwapFarms),
            editing_swap_farm_infos: UnorderedMap::new(StorageKey::EditingSwapFarmInfo),
            token_stake_info: LookupMap::new(StorageKey::TokenStakeInfo),
            lp_stake_info: LookupMap::new(StorageKey::LPStakeInfo),
        };
        this
    }

    // #[init(ignore_state)]
    // pub fn migrate() -> Self {
    //     let prev: ContractV1 = env::state_read().expect("ERR_NOT_INITIALIZED");
    //     assert_eq!(
    //         env::predecessor_account_id(),
    //         prev.owner_id,
    //         "Owner's method"
    //     );
    //     let this = Contract {
    //         owner_id: prev.owner_id,
    //         farm_infos: prev.farm_infos,
    //         storage_deposits: prev.storage_deposits,
    //         admin_ids: prev.admin_ids,
    //         editing_new_farm: prev.editing_new_farm,
    //         confirmed_admins_for_new_farm: prev.confirmed_admins_for_new_farm,
    //         swap_farms: UnorderedMap::new(StorageKey::SwapFarms),
    //         editing_swap_farm_infos: UnorderedMap::new(StorageKey::EditingSwapFarmInfo),
    //     };
    //     this
    // }

    #[payable]
    pub fn add_admin(&mut self, account_id: AccountId) {
        self.assert_owner();
        self.admin_ids.insert(&account_id);
    }

    #[payable]
    pub fn remove_admin(&mut self, account_id: AccountId) {
        self.assert_owner();
        self.admin_ids.remove(&account_id);
        let confirmed_admins_for_new_farm = self.confirmed_admins_for_new_farm.as_vector().to_vec();
        if confirmed_admins_for_new_farm.contains(&account_id) {
            self.confirmed_admins_for_new_farm.remove(&account_id);
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

        let swap_token_len = self.editing_swap_farm_infos.keys_as_vector().to_vec().len();
        if swap_token_len > 0 {
            let swap_tokens = self.editing_swap_farm_infos.keys_as_vector().to_vec();
            for idx in 0..swap_token_len {
                let swap_token = swap_tokens.get(idx).unwrap();
                let mut editing_swap_farm_info =
                    self.editing_swap_farm_infos.get(&swap_token).unwrap();
                if editing_swap_farm_info
                    .confirmed_admins
                    .contains(&account_id)
                {
                    editing_swap_farm_info.confirmed_admins.remove(&account_id);
                    if self.admin_ids.len() <= editing_swap_farm_info.confirmed_admins.len() {
                        let mut swap_farm_data = self.swap_farms.get(&swap_token).unwrap();
                        swap_farm_data.min_lock_time = editing_swap_farm_info.min_lock_time;
                        swap_farm_data.max_lock_time = editing_swap_farm_info.max_lock_time;
                        swap_farm_data.swap_rate = editing_swap_farm_info.swap_rate;
                        self.swap_farms.insert(&swap_token, &swap_farm_data);

                        editing_swap_farm_info.confirmed_admins.clear();
                        self.editing_swap_farm_infos
                            .insert(&swap_token, &editing_swap_farm_info);
                    } else {
                        self.editing_swap_farm_infos
                            .insert(&swap_token, &editing_swap_farm_info);
                    }
                }
            }
        }
    }

    pub fn get_adminlist(&self) -> Vec<AccountId> {
        self.admin_ids.to_vec()
    }


    #[payable]
    pub fn storage_deposit(&mut self, account_id: Option<AccountId>) {
        let storage_account_id = account_id
            .map(|a| a.into())
            .unwrap_or_else(env::predecessor_account_id);
        let deposit = env::attached_deposit();
        assert!(
            deposit >= STORAGE_PER_SALE,
            "Requires minimum deposit of {}",
            STORAGE_PER_SALE
        );
        let mut balance: u128 = self.storage_deposits.get(&storage_account_id).unwrap_or(0);
        balance += deposit;
        self.storage_deposits.insert(&storage_account_id, &balance);
    }

    #[payable]
    pub fn storage_withdraw(&mut self) {
        assert_one_yocto();
        let owner_id = env::predecessor_account_id();
        let amount = self.storage_deposits.remove(&owner_id).unwrap_or(0);
        Promise::new(owner_id.clone()).transfer(amount);
    }

    /// views
    pub fn storage_balance_bounds(&self) -> StorageBalanceBounds {
        StorageBalanceBounds {
            min: U128(STORAGE_PER_SALE),
            max: None,
        }
    }

    pub fn storage_minimum_balance(&self) -> U128 {
        U128(STORAGE_PER_SALE)
    }

    pub fn storage_balance_of(&self, account_id: AccountId) -> U128 {
        U128(self.storage_deposits.get(&AccountId::new_unchecked(account_id.to_string())).unwrap_or(0))
    }

    /// deprecated
    pub fn storage_paid(&self, account_id: AccountId) -> U128 {
        U128(self.storage_deposits.get(&AccountId::new_unchecked(account_id.to_string())).unwrap_or(0))
    }

    pub fn storage_amount(&self) -> U128 {
        U128(STORAGE_PER_SALE)
    }
}
