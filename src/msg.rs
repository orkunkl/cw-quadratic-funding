use crate::error::ContractError;
use cosmwasm_std::{Env, HumanAddr, MessageInfo};
use cw0::Expiration;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
    pub fn validate(&self, env: Env, info: MessageInfo) -> Result<(), ContractError> {
        // check if proposal period is expired
        if self.proposal_period.is_expired(&env.block) {
            return Err(ContractError::ProposalPeriodExpired {});
        }
        // check if voting period is expired
        if self.voting_period.is_expired(&env.block) {
            return Err(ContractError::VotingPeriodExpired {});
        }

        // check of funding coin_denom matches sent_funds
        // maybe throw error when unexpected coin is found?
        if !info
            .sent_funds
            .iter()
            .any(|coin| coin.denom == self.coin_denom)
        {
            return Err(ContractError::ExpectedCoinNotSent {
                coin_denom: self.coin_denom.clone(),
            });
        }

        Ok(())
    }
}

impl Default for InitMsg {
    fn default() -> Self {
        InitMsg {
            create_proposal_whitelist: None,
            vote_proposal_whitelist: None,
            voting_period: Default::default(),
            proposal_period: Default::default(),
            coin_denom: "ucosm".to_string(),
        }
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
    ProposalByID { id: u64 },
    ProposalByFundAddress { fund_address: HumanAddr },
    AllProposals {},
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::coins;
    use cosmwasm_std::testing::{mock_env, mock_info};

    #[test]
    fn validate_init_msg() {
        let mut env = mock_env();
        let info = mock_info("creator", &coins(4, "ucosm"));

        env.block.height = 30;
        let mut msg = InitMsg {
            ..Default::default()
        };
        let mut msg1 = msg.clone();
        msg1.voting_period = Expiration::AtHeight(15);
        match msg1.validate(env.clone(), info.clone()) {
            Ok(_) => panic!("expected error"),
            Err(ContractError::VotingPeriodExpired {}) => {}
            Err(err) => println!("{:?}", err),
        }

        msg.proposal_period = Expiration::AtHeight(15);
        match msg.validate(env, info) {
            Ok(_) => panic!("expected error"),
            Err(ContractError::ProposalPeriodExpired{}) => {}
            Err(err) => println!("{:?}", err),
        }
    }
}
