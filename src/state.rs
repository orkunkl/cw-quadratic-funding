use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, Coin, HumanAddr, StdResult, Storage};
use cosmwasm_storage::{nextval, singleton, singleton_read, ReadonlySingleton, Singleton};
use cw0::Expiration;
use cw_storage_plus::{Item, Map};

pub static STATE_KEY: &[u8] = b"state";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub count: i32,
    pub owner: CanonicalAddr,
}

pub fn state<S: Storage>(storage: &mut S) -> Singleton<S, State> {
    singleton(storage, STATE_KEY)
}

pub fn state_read<S: Storage>(storage: &S) -> ReadonlySingleton<S, State> {
    singleton_read(storage, STATE_KEY)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    // set admin as single address, multisig or contract sig could be used
    pub admin: HumanAddr,
    pub create_proposal_whitelist: Option<Vec<HumanAddr>>,
    pub vote_proposal_whitelist: Option<Vec<HumanAddr>>,
    pub voting_period: Expiration,
    pub proposal_period: Expiration,
    pub coin_denom: String,
}

pub const CONFIG: Item<Config> = Item::new(b"config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Proposal {
    pub title: String,
    pub description: String,
    pub metadata: String,
    pub fund_address: HumanAddr,
}

pub const PROPOSALS: Map<&[u8], Proposal> = Map::new(b"proposal");
pub const PROPOSAL_SEQ: &[u8] = b"proposal_seq";

pub fn proposal_seq<S: Storage>(storage: &mut S) -> Singleton<S, u64> {
    singleton(storage, PROPOSAL_SEQ)
}

pub fn create_proposal<S: Storage>(storage: &mut S, p: &Proposal) -> StdResult<u64> {
    let next_id = nextval(&mut proposal_seq(storage))?;
    PROPOSALS.save(storage, &next_id.to_be_bytes(), p)?;
    Ok(next_id)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Vote {
    pub proposal_id: u64,
    pub voter: HumanAddr,
    pub fund: Coin,
}

pub const VOTES: Map<&[u8], Vote> = Map::new(b"votes");
