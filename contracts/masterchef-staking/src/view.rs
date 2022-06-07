use crate::*;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct FarmSkeletonInfo {
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
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct StakeSkeletonInfo {
    pub farm_id: u16,
    pub owner_id: AccountId,
    pub token_amount: U128,
    pub lp_share_amount: U128,
    pub reward_token_to_claim: U128,
    pub reward_lp_to_claim: U128,
    pub created_at: u64,
    pub claimed_token_at: u64,
    pub claimed_lp_at: u64,
    pub token_locked: Vec<U128>,
    pub lp_share_locked: Vec<U128>,
    pub unlocked_at: Vec<u64>,
    pub staking_duration: Vec<u64>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SwapStakeSkeletonInfo {
    pub owner_id: AccountId,
    pub token_id: AccountId,
    pub token_amount: U128,
    pub token_locked: Vec<U128>,
    pub unlocked_at: Vec<u64>,
    pub created_at: u64,
    pub claimed_token_at: Vec<u64>,
    pub unstake_amount: Vec<U128>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct EditingSwapFarmViewInfo {
    pub token_id: AccountId,
    pub swap_rate: U128,
    pub min_lock_time: u64,
    pub confirmed_admins: Vec<AccountId>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SwapFarmViewInfo {
    pub token_id: AccountId,
    pub swap_rate: U128,
    pub min_lock_time: u64,
}

#[near_bindgen]
impl Contract {
    /// views
    pub fn get_supply_staking_informations(&self, farm_id: u16) -> U64 {
        assert!(self.farm_infos.len() > farm_id.into(), "Invalid Farm ID");

        let farm_info: FarmInfo = self.farm_infos.get(farm_id.into()).unwrap();

        U64(farm_info.stake_infos.len())
    }

    // pub fn get_staking_informations_by_owner_id(xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx

    pub fn get_token_claim_amount(&self, farm_id: u16, account_id: AccountId) -> U128 {
        self.claim_amount(farm_id, account_id, 0)
    }

    pub fn get_token_locked_amount(&self, farm_id: u16, account_id: AccountId) -> U128 {
        U128(self.token_locked_amount(farm_id, account_id))
    }

    pub fn get_lp_claim_amount(&self, farm_id: u16, account_id: AccountId) -> U128 {
        self.claim_amount(farm_id, account_id, 1)
    }

    pub fn get_lp_locked_amount(&self, farm_id: u16, account_id: AccountId) -> U128 {
        U128(self.lp_locked_amount(farm_id, account_id))
    }

    pub fn get_farm_length(&self) -> U64 {
        U64(self.farm_infos.len())
    }

    pub fn get_farm_info(&self, farm_id: u16) -> FarmSkeletonInfo {
        let farm_info = self.farm_infos.get(farm_id.into()).unwrap();
        FarmSkeletonInfo {
            farm_id: farm_info.farm_id,
            token_id: farm_info.token_id,
            pool_id: farm_info.pool_id,
            reward_token_id: farm_info.reward_token_id,
            total_token_amount: farm_info.total_token_amount,
            total_lp_share_amount: farm_info.total_lp_share_amount,
            token_reward_rate: farm_info.token_reward_rate,
            pool_reward_rate: farm_info.pool_reward_rate,
            starting_at: farm_info.starting_at,
            ending_at: farm_info.ending_at,
        }
    }

    pub fn get_stake_infos_by_ownerid(
        &self,
        farm_id: u16,
        account_id: AccountId,
    ) -> Vec<StakeSkeletonInfo> {
        // let account_id = env::predecessor_account_id();

        let mut tmp = vec![];
        // let start = u64::from(self.farm_infos.len()-1);
        let stake_infos_tmp = self.farm_infos.get(farm_id.into()).unwrap().stake_infos;
        let info: Option<StakeInfo> = stake_infos_tmp.get(&account_id);
        if let Some(info) = info {
            let stake_info = StakeSkeletonInfo {
                farm_id: farm_id,
                owner_id: info.owner_id,
                token_amount: info.token_amount,
                lp_share_amount: info.lp_share_amount,
                reward_token_to_claim: info.reward_token_to_claim,
                reward_lp_to_claim: info.reward_lp_to_claim,
                created_at: info.created_at,
                claimed_token_at: info.claimed_token_at,
                claimed_lp_at: info.claimed_lp_at,
                token_locked: info.token_locked.to_vec(),
                lp_share_locked: info.lp_share_locked.to_vec(),
                unlocked_at: info.unlocked_at.to_vec(),
                staking_duration: info.staking_duration.to_vec(),
            };
            tmp.push(stake_info);
            tmp
        } else {
            vec![]
        }
        // tmp
    }

    pub fn get_swap_stake_info_by_userid(
        &self,
        account_id: AccountId,
    ) -> Vec<SwapStakeSkeletonInfo> {
        let mut stake_info_vec = vec![];
        for i in 0..self.swap_farms.len() {
            let swap_token = self.swap_farms.keys_as_vector().get(i).unwrap();
            let swap_farm_data = self.swap_farms.get(&swap_token).unwrap();
            let info = swap_farm_data.stake_infos.get(&account_id);
            if let Some(info) = info {
                let swap_unstake_amount =
                    self.get_swap_unstake_amount(swap_token.clone(), info.owner_id.clone());
                stake_info_vec.push(SwapStakeSkeletonInfo {
                    owner_id: info.owner_id.clone(),
                    token_id: swap_token.clone(),
                    token_amount: info.token_amount,
                    token_locked: info.token_locked.to_vec(),
                    unlocked_at: info.unlocked_at.to_vec(),
                    claimed_token_at: info.claimed_token_at.to_vec(),
                    created_at: info.created_at,
                    unstake_amount: swap_unstake_amount,
                });
            } else {
            }
        }
        stake_info_vec
    }

    pub fn get_swap_tokens(&self) -> Vec<AccountId> {
        self.swap_farms.keys_as_vector().to_vec()
    }

    pub fn get_swap_farm_info(&self) -> Vec<SwapFarmViewInfo> {
        let swap_tokens = self.swap_farms.keys_as_vector();
        let start: u64 = 0;
        let end = swap_tokens.len();
        let mut swap_info = vec![];
        for i in start..end {
            let swap_token = swap_tokens.get(i).unwrap();
            swap_info.push(SwapFarmViewInfo {
                token_id: swap_token.clone(),
                swap_rate: self.swap_farms.get(&swap_token).unwrap().swap_rate,
                min_lock_time: self.swap_farms.get(&swap_token).unwrap().min_lock_time,
            });
        }
        swap_info
    }

    pub fn get_editing_swap_farm_info(&self) -> Vec<EditingSwapFarmViewInfo> {
        let mut result = vec![];
        let swap_tokens = self.editing_swap_farm_infos.keys_as_vector();
        for i in 0..self.editing_swap_farm_infos.len() {
            let swap_token = swap_tokens.get(i).unwrap();
            let editing_swap_farm_info = self.editing_swap_farm_infos.get(&swap_token).unwrap();
            result.push(EditingSwapFarmViewInfo {
                token_id: swap_token,
                swap_rate: editing_swap_farm_info.swap_rate,
                min_lock_time: editing_swap_farm_info.min_lock_time,
                confirmed_admins: editing_swap_farm_info.confirmed_admins.as_vector().to_vec(),
            });
        }
        result
    }
}
