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

#[near_bindgen]
impl Contract {
    /// views
    pub fn get_supply_staking_informations(&self, farm_id: u16) -> U64 {
        assert!(self.farm_infos.len() > farm_id.into(), "Invalid Farm ID");

        let farm_info: FarmInfo = self.farm_infos.get(farm_id.into()).unwrap();

        U64(farm_info.stake_infos.len())
    }

    // pub fn get_staking_informations_by_owner_id(
    //     &self,
    //     farm_id: u16,
    //     account_id: AccountId
    // ) -> StakeInfo {
    //     self.staking_informations_by_owner_id(farm_id, account_id)
    // }

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
        let mut farm_info = self.farm_infos.get(farm_id.into()).unwrap();
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
            let mut stake_info = StakeSkeletonInfo {
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
}
