use crate::*;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct StakeInfo {
    pub owner_id: AccountId,
    pub token_amount: U128,
    pub lp_share_amount: U128,
    pub reward_token_to_claim: U128,
    pub reward_lp_to_claim: U128,
    pub token_locked: Vector<U128>,
    pub lp_share_locked: Vector<U128>,
    pub unlocked_at: Vector<u64>,
    pub staking_duration: Vector<u64>,
    pub created_at: u64,
    pub claimed_token_at: u64,
    pub claimed_lp_at: u64,
}
