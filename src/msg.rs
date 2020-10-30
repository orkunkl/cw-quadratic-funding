use crate::error::ContractError;
use crate::helper::extract_funding_coin;
use crate::state::Proposal;
use cosmwasm_std::{Binary, Env, HumanAddr, MessageInfo};
use cw0::Expiration;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InitMsg {
    pub admin: HumanAddr,
    pub create_proposal_whitelist: Option<Vec<HumanAddr>>,
    pub vote_proposal_whitelist: Option<Vec<HumanAddr>>,
    pub voting_period: Expiration,
    pub proposal_period: Expiration,
}

impl InitMsg {
    pub fn validate(&self, env: Env, info: &MessageInfo) -> Result<(), ContractError> {
        // check if proposal period is expired
        if self.proposal_period.is_expired(&env.block) {
            return Err(ContractError::ProposalPeriodExpired {});
        }
        // check if voting period is expired
        if self.voting_period.is_expired(&env.block) {
            return Err(ContractError::VotingPeriodExpired {});
        }

        extract_funding_coin(&info.sent_funds)?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    CreateProposal {
        title: String,
        description: String,
        metadata: Option<Binary>,
        fund_address: HumanAddr,
    },
    VoteProposal {
        proposal_id: u64,
    },
    TriggerDistribution {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    ProposalByID { id: u64 },
    AllProposals {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AllProposalsResponse {
    pub proposals: Vec<Proposal>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::coin;
    use cosmwasm_std::testing::{mock_env, mock_info};

    #[test]
    fn validate_init_msg() {
        let mut env = mock_env();
        let denom = String::from("denom");
        let info = mock_info("creator", &[coin(4, denom.as_str())]);

        env.block.height = 30;
        let msg = InitMsg {
            ..Default::default()
        };

        let mut msg1 = msg.clone();
        msg1.voting_period = Expiration::AtHeight(15);
        match msg1.validate(env.clone(), &info) {
            Ok(_) => panic!("expected error"),
            Err(ContractError::VotingPeriodExpired {}) => {}
            Err(err) => println!("{:?}", err),
        }

        let mut msg2 = msg.clone();
        msg2.proposal_period = Expiration::AtHeight(15);
        match msg2.validate(env.clone(), &info) {
            Ok(_) => panic!("expected error"),
            Err(ContractError::ProposalPeriodExpired {}) => {}
            Err(err) => println!("{:?}", err),
        }

        let msg3 = msg.clone();
        match msg3.validate(env, &info) {
            Ok(_) => panic!("expected error"),
            Err(ContractError::MultipleCoinsSent {}) => {}
            Err(err) => println!("{:?}", err),
        }
    }
}
