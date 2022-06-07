use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap, UnorderedSet, Vector};
use near_sdk::json_types::{ValidAccountId, U128, U64};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::PromiseOrValue;
use near_sdk::{
    assert_one_yocto, env, ext_contract, near_bindgen, AccountId, Balance, BorshStorageKey, Gas,
    PanicOnDefault, Promise,
};
use std::convert::TryInto;

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

near_sdk::setup_alloc!();

//I already adjusted this contract and it has basic info for staking.

// TODO check seller supports storage_deposit at ft_token_id they want to post sale in

const GAS_FOR_FT_TRANSFER: Gas = 5_000_000_000_000;
const STORAGE_PER_SALE: u128 = 1000 * STORAGE_PRICE_PER_BYTE;
const TOKEN_CONTRACT_ID: &str = "unet.testnet";
const REF_CONTRACT_ID: &str = "ref-finance-101.testnet";

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct StorageBalanceBounds {
    pub min: U128,
    pub max: Option<U128>,
}

#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct ContractV1 {
    pub owner_id: AccountId,
    pub farm_infos: Vector<FarmInfo>,
    pub storage_deposits: LookupMap<AccountId, Balance>,
    pub admin_ids: UnorderedSet<AccountId>,
    pub editing_new_farm: EditingFarmInfo,
    pub confirmed_admins_for_new_farm: UnorderedSet<AccountId>,
    pub swap_rate_info: UnorderedMap<AccountId, U128>,
    pub swap_farms: UnorderedMap<AccountId, SwapFarmInfo>,
}

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
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner_id: ValidAccountId) -> Self {
        let this = Self {
            owner_id: owner_id.into(),
            farm_infos: Vector::new(StorageKey::FarmInfos),
            storage_deposits: LookupMap::new(StorageKey::StorageDeposits),
            admin_ids: UnorderedSet::new(StorageKey::AdminIds),
            editing_new_farm: EditingFarmInfo {
                farm_id: 0,
                token_id: "unet.testnet".to_owned(),
                pool_id: 382,
                reward_token_id: "unet.testnet".to_owned(),
                token_reward_rate: U128(0),
                pool_reward_rate: U128(0),
                starting_at: 0,
                ending_at: 0,
            },
            confirmed_admins_for_new_farm: UnorderedSet::new(StorageKey::ConfirmedAdminsForNewFarm),
            swap_farms: UnorderedMap::new(StorageKey::SwapFarms),
            editing_swap_farm_infos: UnorderedMap::new(StorageKey::EditingSwapFarmInfo),
        };
        this
    }

    #[init(ignore_state)]
    pub fn migrate() -> Self {
        let prev: ContractV1 = env::state_read().expect("ERR_NOT_INITIALIZED");
        assert_eq!(
            env::predecessor_account_id(),
            prev.owner_id,
            "Owner's method"
        );
        let this = Contract {
            owner_id: prev.owner_id,
            farm_infos: prev.farm_infos,
            storage_deposits: prev.storage_deposits,
            admin_ids: prev.admin_ids,
            editing_new_farm: prev.editing_new_farm,
            confirmed_admins_for_new_farm: prev.confirmed_admins_for_new_farm,
            swap_farms: UnorderedMap::new(StorageKey::SwapFarms),
            editing_swap_farm_infos: UnorderedMap::new(StorageKey::EditingSwapFarmInfo),
        };
        this
    }

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
                swap_farm_data.swap_rate = editing_swap_farm_data.swap_rate;
                self.swap_farms.insert(&token_id, &swap_farm_data);
            }
            self.editing_swap_farm_infos
                .insert(&token_id, &editing_swap_farm_data);
        } else {
            env::panic(b"Invalid token id");
        }
    }

    #[payable]
    pub fn save_editing_swap_farm_info(
        &mut self,
        token_id: AccountId,
        swap_rate: U128,
        lock_time: u64,
    ) {
        self.assert_admin();
        assert_one_yocto();
        let swap_tokens = self.editing_swap_farm_infos.keys_as_vector().to_vec();
        if swap_tokens.contains(&token_id) {
            let mut editing_swap_farm_info = self.editing_swap_farm_infos.get(&token_id).unwrap();
            editing_swap_farm_info.min_lock_time = lock_time;
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
                swap_farm_data.min_lock_time = lock_time;
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
                    lock_time,
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
        lock_time: u64,
    ) {
        self.assert_owner();
        assert_one_yocto();
        let mut swap_tokens = self.swap_farms.keys_as_vector().to_vec();

        if swap_tokens.contains(&token_id) {
            let mut swap_farm_info = self.swap_farms.get(&token_id).unwrap();
            swap_farm_info.min_lock_time = lock_time;
            swap_farm_info.token_decimal = decimal;
            swap_farm_info.swap_rate = swap_rate;
            self.swap_farms.insert(&token_id, &swap_farm_info);
        } else {
            self.swap_farms.insert(
                &token_id,
                &SwapFarmInfo::new(token_id.clone(), decimal, lock_time, swap_rate),
            );
        }

        swap_tokens = self.editing_swap_farm_infos.keys_as_vector().to_vec();
        if swap_tokens.contains(&token_id) {
            let mut editing_swap_farm_info = self.editing_swap_farm_infos.get(&token_id).unwrap();
            editing_swap_farm_info.min_lock_time = lock_time;
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
                    lock_time,
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

    pub fn get_editing_farm_info(&self) -> EditingFarmInfo {
        // (&self.editing_new_farm).into()
        let data = EditingFarmInfo {
            farm_id: self.editing_new_farm.farm_id,
            ending_at: self.editing_new_farm.ending_at,
            pool_id: self.editing_new_farm.pool_id,
            pool_reward_rate: self.editing_new_farm.pool_reward_rate,
            reward_token_id: self.editing_new_farm.reward_token_id.clone(),
            starting_at: self.editing_new_farm.starting_at,
            token_id: self.editing_new_farm.token_id.clone(),
            token_reward_rate: self.editing_new_farm.token_reward_rate,
        };
        data
        // data
    }

    #[payable]
    pub fn save_editing_farm_info(
        &mut self,
        token_id: AccountId,
        pool_id: u64,
        reward_token_id: AccountId,
        token_reward_rate: U128,
        pool_reward_rate: U128,
        starting_at: u64,
        ending_at: u64,
    ) {
        self.assert_admin();
        assert_one_yocto();
        for elem in self.farm_infos.to_vec() {
            if elem.starting_at < starting_at && starting_at < elem.ending_at
                || elem.starting_at < ending_at && ending_at < elem.ending_at
                || starting_at < elem.starting_at && elem.starting_at < ending_at
                || starting_at < elem.ending_at && elem.ending_at < ending_at
            {
                env::panic(b"Farm Time Overlaps. Plz input valid farm time");
            }
        }
        self.editing_new_farm = EditingFarmInfo {
            farm_id: 1,
            token_id: token_id,
            pool_id: pool_id,
            token_reward_rate: token_reward_rate,
            pool_reward_rate: pool_reward_rate,
            reward_token_id: reward_token_id,
            starting_at: starting_at,
            ending_at: ending_at,
        };
        self.confirmed_admins_for_new_farm.clear();
        self.confirmed_admins_for_new_farm
            .insert(&env::predecessor_account_id());

        if self.confirmed_admins_for_new_farm.len() == self.admin_ids.len() {
            self.farm_infos.push(&FarmInfo::new(
                self.farm_infos.len() as u16,
                self.editing_new_farm.token_id.clone(),
                self.editing_new_farm.pool_id,
                self.editing_new_farm.reward_token_id.clone(),
                self.editing_new_farm.token_reward_rate,
                self.editing_new_farm.pool_reward_rate,
                self.editing_new_farm.starting_at,
                self.editing_new_farm.ending_at,
            ));
            self.confirmed_admins_for_new_farm.clear();
            self.editing_new_farm = EditingFarmInfo {
                farm_id: 0,
                token_id: "unet.testnet".to_owned(),
                pool_id: 382,
                reward_token_id: "unet.testnet".to_owned(),
                token_reward_rate: U128(0),
                pool_reward_rate: U128(0),
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
                self.editing_new_farm.token_id.clone(),
                self.editing_new_farm.pool_id,
                self.editing_new_farm.reward_token_id.clone(),
                self.editing_new_farm.token_reward_rate,
                self.editing_new_farm.pool_reward_rate,
                self.editing_new_farm.starting_at,
                self.editing_new_farm.ending_at,
            ));
            self.confirmed_admins_for_new_farm.clear();
            self.editing_new_farm = EditingFarmInfo {
                farm_id: 0,
                token_id: "unet.testnet".to_owned(),
                pool_id: 382,
                reward_token_id: "unet.testnet".to_owned(),
                token_reward_rate: U128(0),
                pool_reward_rate: U128(0),
                starting_at: 0,
                ending_at: 0,
            };
        }
    }

    pub fn get_confirmed_admins_for_new_farm(&self) -> Vec<AccountId> {
        self.confirmed_admins_for_new_farm.to_vec()
    }

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
                    self.editing_new_farm.token_id.clone(),
                    self.editing_new_farm.pool_id,
                    self.editing_new_farm.reward_token_id.clone(),
                    self.editing_new_farm.token_reward_rate,
                    self.editing_new_farm.pool_reward_rate,
                    self.editing_new_farm.starting_at,
                    self.editing_new_farm.ending_at,
                ));
                self.confirmed_admins_for_new_farm.clear();
                self.editing_new_farm = EditingFarmInfo {
                    farm_id: 0,
                    token_id: "unet.testnet".to_owned(),
                    pool_id: 382,
                    reward_token_id: "unet.testnet".to_owned(),
                    token_reward_rate: U128(0),
                    pool_reward_rate: U128(0),
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
    pub fn claim_reward(&mut self, farm_id: u16, token_type: u8) {
        assert_one_yocto();
        let account_id = env::predecessor_account_id();
        let mut claim_amount = U128(0);

        let mut farm_info = self.farm_infos.get(farm_id.into()).unwrap();
        let mut stake_info = farm_info.stake_infos.get(&account_id).unwrap();
        if token_type == 0 {
            claim_amount = self.claim_amount(farm_id, account_id.clone(), 0);
            stake_info.reward_token_to_claim = U128(0);
            stake_info.token_amount = U128::from(
                u128::from(stake_info.token_amount)
                    .checked_add(u128::from(claim_amount))
                    .unwrap(),
            );
            farm_info.total_token_amount = U128::from(
                u128::from(farm_info.total_token_amount)
                    .checked_add(u128::from(claim_amount))
                    .unwrap(),
            );
            stake_info.claimed_token_at = env::block_timestamp() / 1000000;
        }
        if token_type == 1 {
            claim_amount = self.claim_amount(farm_id, account_id.clone(), 1);
            stake_info.reward_lp_to_claim = U128(0);
            stake_info.token_amount = U128::from(
                u128::from(stake_info.token_amount)
                    .checked_add(u128::from(claim_amount))
                    .unwrap(),
            );
            farm_info.total_token_amount = U128::from(
                u128::from(farm_info.total_token_amount)
                    .checked_add(u128::from(claim_amount))
                    .unwrap(),
            );
            stake_info.claimed_lp_at = env::block_timestamp() / 1000000;
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

        if token_type == 0 {
            assert!(
                amount.0
                    <= (u128::from(stake_info.token_amount)
                        - self.token_locked_amount(farm_id, account_id.clone())),
                "Amount must be smaller than available amount"
            );
        } else if token_type == 1 {
            assert!(
                amount.0
                    <= (u128::from(stake_info.lp_share_amount)
                        - self.lp_locked_amount(farm_id, account_id.clone())),
                "Amount must be smaller than available amount"
            );
        }

        self.claim_reward(farm_id, token_type);

        if token_type == 0 {
            ext_contract::ft_transfer(
                account_id.clone().try_into().unwrap(),
                amount,
                None,
                &AccountId::from(TOKEN_CONTRACT_ID),
                1,
                GAS_FOR_FT_TRANSFER,
            );
        } else if token_type == 1 {
            ext_contract::mft_transfer(
                ":".to_owned() + &farm_info.pool_id.to_string(),
                account_id.clone().try_into().unwrap(),
                amount,
                None,
                &AccountId::from(REF_CONTRACT_ID),
                1,
                GAS_FOR_FT_TRANSFER,
            );
        }

        if token_type == 0 {
            stake_info.token_amount = U128::from(
                u128::from(stake_info.token_amount)
                    .checked_sub(u128::from(amount))
                    .unwrap(),
            );
            farm_info.total_token_amount = U128::from(
                u128::from(farm_info.total_token_amount)
                    .checked_sub(u128::from(amount))
                    .unwrap(),
            );
        } else if token_type == 1 {
            stake_info.lp_share_amount = U128::from(
                u128::from(stake_info.lp_share_amount)
                    .checked_sub(u128::from(amount))
                    .unwrap(),
            );
            farm_info.total_lp_share_amount = U128::from(
                u128::from(farm_info.total_lp_share_amount)
                    .checked_sub(u128::from(amount))
                    .unwrap(),
            );
        }
        farm_info.stake_infos.insert(&account_id, &stake_info);
        self.farm_infos.replace(farm_id.into(), &farm_info);
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
            .checked_sub(86400 * 365 * 3 * 1000)
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

    // pub fn get_swap_unstake_amount(&self, token_id: AccountId, account_id: AccountId) -> Vec<U128> {
    //     let swap_farm_data = self.swap_farms.get(&token_id).unwrap();
    //     let stake_info = swap_farm_data.stake_infos.get(&account_id).unwrap();
    //     let mut unstake_amount_vec = vec![];

    //     for id in 0..stake_info.token_locked.len() {
    //         let mut unstake_amount = U128(0);
    //         if stake_info.unlocked_at.get(id).unwrap() < env::block_timestamp() / 1000000 {
    //             unstake_amount = U128::from(u128::from(stake_info.token_locked.get(id).unwrap()));
    //         } else {
    //             unstake_amount = U128::from(
    //                 u128::from(stake_info.token_locked.get(id).unwrap())
    //                     .checked_div(10000000000)
    //                     .unwrap()
    //                     .checked_mul(u128::from(
    //                         (env::block_timestamp() / 1000000
    //                             - stake_info.claimed_token_at.get(id).unwrap())
    //                             / (stake_info.unlocked_at.get(id).unwrap()
    //                                 - stake_info.claimed_token_at.get(id).unwrap()),
    //                     ))
    //                     .unwrap()
    //                     .checked_mul(10000000000)
    //                     .unwrap(),
    //             );
    //         };
    //         unstake_amount_vec.push(unstake_amount);
    //     }
    //     unstake_amount_vec
    // }
    pub fn get_swap_unstake_amount(&self, token_id: AccountId, account_id: AccountId) -> Vec<U128> {
        let swap_farm_data = self.swap_farms.get(&token_id).unwrap();
        let stake_info = swap_farm_data.stake_infos.get(&account_id).unwrap();
        let mut unstake_amount = vec![];
        let startIdx: u64 = 0;
        let endIdx = stake_info.token_locked.len();
        for idx in startIdx..endIdx {
            if stake_info.unlocked_at.get(idx).unwrap() < env::block_timestamp() / 1000000 {
                unstake_amount.push(stake_info.token_locked.get(idx).unwrap());
            } else if u64::from(stake_info.unlocked_at.get(idx).unwrap())
                .checked_sub(86400 * 365 * 3 * 1000)
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

        ext_contract::ft_transfer(
            account_id.clone().try_into().unwrap(),
            amount_2_unstake,
            None,
            &AccountId::from(TOKEN_CONTRACT_ID),
            1,
            GAS_FOR_FT_TRANSFER,
        );
    }

    #[payable]
    pub fn storage_deposit(&mut self, account_id: Option<ValidAccountId>) {
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

    pub fn storage_balance_of(&self, account_id: ValidAccountId) -> U128 {
        U128(self.storage_deposits.get(account_id.as_ref()).unwrap_or(0))
    }

    /// deprecated
    pub fn storage_paid(&self, account_id: ValidAccountId) -> U128 {
        U128(self.storage_deposits.get(account_id.as_ref()).unwrap_or(0))
    }

    pub fn storage_amount(&self) -> U128 {
        U128(STORAGE_PER_SALE)
    }
}
