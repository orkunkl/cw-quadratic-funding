use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, Coin, HumanAddr, Storage};
use cosmwasm_storage::{singleton, Singleton};
use cw0::Expiration;
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    // set admin as single address, multisig or contract sig could be used
    pub admin: CanonicalAddr,
    pub create_proposal_whitelist: Option<Vec<HumanAddr>>,
    pub vote_proposal_whitelist: Option<Vec<HumanAddr>>,
    pub voting_period: Expiration,
    pub proposal_period: Expiration,
    pub budget: Coin,
}

pub const CONFIG: Item<Config> = Item::new(b"config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Proposal {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub metadata: String,
    pub fund_address: CanonicalAddr,
}

impl Default for Proposal {
    fn default() -> Self {
        Proposal {
            id: 0,
            title: "title".to_string(),
            description: "desc".to_string(),
            metadata: "dec".to_string(),
            fund_address: Default::default(),
        }
    }
}
pub const PROPOSALS: Map<&[u8], Proposal> = Map::new(b"proposal");
pub const PROPOSAL_SEQ: &[u8] = b"proposal_seq";

pub fn proposal_seq<S: Storage>(storage: &mut S) -> Singleton<S, u64> {
    singleton(storage, PROPOSAL_SEQ)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Vote {
    pub proposal_id: u64,
    pub voter: CanonicalAddr,
    pub fund: Coin,
}

pub const VOTES: Map<(&[u8], &[u8]), Vote> = Map::new(b"votes");
