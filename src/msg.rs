use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{HumanAddr, Env, MessageInfo};
use cw0::Expiration;
use crate::error::ContractError;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InitMsg {
    pub create_proposal_whitelist: Option<Vec<HumanAddr>>,
    pub vote_proposal_whitelist: Option<Vec<HumanAddr>>,
    pub voting_period: Expiration,
    pub proposal_period: Expiration,
    pub coin_denom: String,
}

impl InitMsg {
    pub fn validate(self, env: Env, info: MessageInfo) -> Result<(),ContractError>{
        // check if proposal period is expired
        if self.proposal_period.is_expired(&env.block) {
            Err(ContractError::ProposalPeriodExpired {})
        }
        // check if voting period is expired
        if self.voting_period.is_expired(&env.block) {
            Err(ContractError::VotingPeriodExpired {})
        }

        // check of funding coin_denom matches sent_funds
        // maybe throw error when unexpected coin is found?
        if !info.sent_funds.iter().any(|coin| {
            coin.denom == self.coin_denom
        }) {
            Err(ContractError::ExpectedCoinNotSent{ coin_denom })
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    /*
    CreateProposal {
        description: String,
        metadata: String,
        fund_address: HumanAddr
    },
    VoteProposal {
        proposal_id: u32,
    },
    TriggerDistribution {
        proposal_id: u32
    },
     */
    Increment {},
    Reset { count: i32 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetCount {},
    ProposalByID {
        id: u64,
    },
    ProposalByFundAddress {
        fund_address: HumanAddr
    },
    AllProposals {},
}
