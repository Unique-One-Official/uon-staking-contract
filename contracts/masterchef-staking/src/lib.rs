use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap, UnorderedSet, Vector};
use near_sdk::json_types::{ValidAccountId, U128, U64};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    assert_one_yocto, env, ext_contract, near_bindgen, AccountId, Balance, BorshStorageKey,
    Gas, PanicOnDefault, Promise,
};
use near_sdk::PromiseOrValue;
use std::convert::TryInto;

use crate::external::*;
use crate::farm_info::*;
use crate::stake_info::*;

use near_sdk::env::STORAGE_PRICE_PER_BYTE;

mod external;
mod internal;
mod ft_callbacks;
mod farm_info;
mod stake_info;
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

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub owner_id: AccountId,
    pub farm_infos: Vector<FarmInfo>,
    pub storage_deposits: LookupMap<AccountId, Balance>,
}

/// Helper structure to for keys of the persistent collections.
#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKey {
    FarmInfos,
    StakeInfos { farm_id: u16 },
    LockInfos {
        farm_id: u16,
        account_id: AccountId,
        lock_info_type: u8,
    },
    StorageDeposits,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(
        owner_id: ValidAccountId
    ) -> Self {
        let this = Self {
            owner_id: owner_id.into(),
            farm_infos: Vector::new(StorageKey::FarmInfos),
            storage_deposits: LookupMap::new(StorageKey::StorageDeposits),
        };
        this
    }

    #[payable]
    pub fn claim_reward(
        &mut self,
        farm_id: u16,
        token_type: u8
    ) {
        assert_one_yocto();
        let account_id = env::predecessor_account_id();
        let mut claim_amount = U128(0);

        let mut farm_info = self.farm_infos.get(farm_id.into()).unwrap();
        let mut stake_info = farm_info.stake_infos.get(&account_id).unwrap();
        if(token_type == 0) {
            claim_amount = self.claim_amount(farm_id, account_id.clone(), 0);
            stake_info.reward_token_to_claim = U128(0);
            stake_info.token_amount = U128::from(u128::from(stake_info.token_amount).checked_add(u128::from(claim_amount)).unwrap());
            farm_info.total_token_amount = U128::from(u128::from(farm_info.total_token_amount).checked_add(u128::from(claim_amount)).unwrap());
            stake_info.claimed_token_at = env::block_timestamp() / 1000000;
        } 
        if(token_type == 1) {
            claim_amount = self.claim_amount(farm_id, account_id.clone(), 1);
            stake_info.reward_lp_to_claim = U128(0);
            stake_info.token_amount = U128::from(u128::from(stake_info.token_amount).checked_add(u128::from(claim_amount)).unwrap());
            farm_info.total_token_amount = U128::from(u128::from(farm_info.total_token_amount).checked_add(u128::from(claim_amount)).unwrap());
            stake_info.claimed_lp_at = env::block_timestamp() / 1000000;
        }
       
        farm_info.stake_infos.insert(
            &account_id,
            &stake_info
        );
        self.farm_infos.replace(farm_id.into(), &farm_info);
    }

    #[payable]
    pub fn unstake(
        &mut self,
        farm_id: u16,
        token_type: u8,
        amount: U128
    ) {
        assert_one_yocto();
        let account_id = env::predecessor_account_id();
        assert!(self.farm_infos.len() > farm_id.into(), "Invalid Farm ID");
        let mut farm_info = self.farm_infos.get(farm_id.into()).unwrap();

        let mut stake_info = farm_info.stake_infos.get(&account_id).unwrap();

        if (token_type == 0){
            assert!(amount.0 <= (u128::from(stake_info.token_amount)-self.token_locked_amount(farm_id, account_id.clone())), "Amount must be smaller than available amount");
        } else if (token_type == 1){
            assert!(amount.0 <= (u128::from(stake_info.lp_share_amount)-self.lp_locked_amount(farm_id, account_id.clone())), "Amount must be smaller than available amount");
        }

        self.claim_reward(farm_id, token_type);

        if (token_type == 0){
            ext_contract::ft_transfer(
                account_id.clone().try_into().unwrap(),
                amount,
                None,
                &AccountId::from(TOKEN_CONTRACT_ID),
                1,
                GAS_FOR_FT_TRANSFER,
            );
        } else if (token_type == 1){
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

        if (token_type == 0){
            stake_info.token_amount = U128::from(u128::from(stake_info.token_amount).checked_sub(u128::from(amount)).unwrap());
            farm_info.total_token_amount = U128::from(u128::from(farm_info.total_token_amount).checked_sub(u128::from(amount)).unwrap());
        } else if (token_type == 1){
            stake_info.lp_share_amount = U128::from(u128::from(stake_info.lp_share_amount).checked_sub(u128::from(amount)).unwrap());
            farm_info.total_lp_share_amount = U128::from(u128::from(farm_info.total_lp_share_amount).checked_sub(u128::from(amount)).unwrap());
        }
        farm_info.stake_infos.insert(
            &account_id,
            &stake_info
        );
        self.farm_infos.replace(farm_id.into(), &farm_info);
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
        let mut amount = self.storage_deposits.remove(&owner_id).unwrap_or(0);
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
