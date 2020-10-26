use cosmwasm_std::{
    attr, Api, Binary, Env, Extern, HandleResponse, HumanAddr, InitResponse, MessageInfo, Querier,
    StdResult, Storage,
};

use crate::error::ContractError;
use crate::helper::extract_funding_coin;
use crate::msg::{HandleMsg, InitMsg, QueryMsg};
use crate::state::{create_proposal, Config, Proposal, Vote, CONFIG, PROPOSALS, VOTES};

// Note, you can use StdResult in some functions where you do not
// make use of the custom errors
pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    info: MessageInfo,
    msg: InitMsg,
) -> Result<InitResponse, ContractError> {
    msg.validate(env, info)?;

    let cfg = Config {
        create_proposal_whitelist: msg.create_proposal_whitelist,
        vote_proposal_whitelist: msg.vote_proposal_whitelist,
        voting_period: msg.voting_period,
        proposal_period: msg.proposal_period,
        coin_denom: msg.coin_denom,
    };
    CONFIG.save(&mut deps.storage, &cfg)?;

    Ok(InitResponse::default())
}

// And declare a custom Error variant for the ones where you will want to make use of it
pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    info: MessageInfo,
    msg: HandleMsg,
) -> Result<HandleResponse, ContractError> {
    match msg {
        HandleMsg::CreateProposal {
            title,
            description,
            metadata,
            fund_address,
        } => try_create_proposal(deps, env, info, title, description, metadata, fund_address),
        HandleMsg::VoteProposal { proposal_id } => try_vote_proposal(deps, env, info, proposal_id),
    }
}

pub fn try_create_proposal<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    info: MessageInfo,
    title: String,
    description: String,
    metadata: String,
    fund_address: HumanAddr,
) -> Result<HandleResponse, ContractError> {
    let config = CONFIG.load(&deps.storage)?;
    // check whitelist
    if config.create_proposal_whitelist.is_some() {
        let wl = config.create_proposal_whitelist.unwrap();
        if !wl.contains(&info.sender) {
            return Err(ContractError::Unauthorized {});
        }
    }

    // check proposal expiration
    if config.proposal_period.is_expired(&env.block) {
        return Err(ContractError::ProposalPeriodExpired {});
    }

    let proposal = Proposal {
        title,
        description,
        metadata,
        fund_address,
    };
    let proposal_id = create_proposal(&mut deps.storage, &proposal)?;

    let res = HandleResponse {
        messages: vec![],
        attributes: vec![
            attr("action", "create_proposal"),
            attr("proposal_id", proposal_id),
        ],
        data: Some(Binary::from(proposal_id.to_be_bytes())),
    };

    Ok(res)
}

pub fn try_vote_proposal<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    info: MessageInfo,
    proposal_id: u64,
) -> Result<HandleResponse, ContractError> {
    let config = CONFIG.load(&deps.storage)?;
    // check whitelist
    if config.vote_proposal_whitelist.is_some() {
        let wl = config.vote_proposal_whitelist.unwrap();
        if !wl.contains(&info.sender) {
            return Err(ContractError::Unauthorized {});
        }
    }
    // check voting expiration
    if config.voting_period.is_expired(&env.block) {
        return Err(ContractError::VotingPeriodExpired {});
    }

    // validate sent funds and funding denom matches
    let coin = extract_funding_coin(&info.sent_funds, config.coin_denom)?;

    // check proposal exists
    PROPOSALS.load(&deps.storage, &proposal_id.to_be_bytes())?;

    let vote = Vote {
        proposal_id,
        voter: info.sender,
        fund: coin,
    };

    VOTES.save(&mut deps.storage, &proposal_id.to_be_bytes(), &vote)?;

    let res = HandleResponse {
        attributes: vec![
            attr("action", "vote_proposal"),
            attr("proposal_id", proposal_id),
        ],
        ..Default::default()
    };

    Ok(res)
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    _deps: &Extern<S, A, Q>,
    _env: Env,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::ProposalByID { .. } => {}
        QueryMsg::ProposalByFundAddress { .. } => {}
        QueryMsg::AllProposals { .. } => {}
    }
    Ok(Binary::from(b"1"))
}

#[cfg(test)]
mod tests {
    use crate::contract::{handle, init};
    use crate::error::ContractError;
    use crate::msg::{HandleMsg, InitMsg};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coin, Binary, HumanAddr};
    use cw0::Expiration;

    #[test]
    fn create_proposal() {
        let mut env = mock_env();
        let info = mock_info("addr", &[coin(1000, "ucosm")]);
        let mut deps = mock_dependencies(&[]);

        let init_msg = InitMsg {
            create_proposal_whitelist: None,
            vote_proposal_whitelist: None,
            voting_period: Expiration::AtHeight(env.block.height + 15),
            proposal_period: Expiration::AtHeight(env.block.height + 10),
            coin_denom: "ucosm".to_string(),
        };

        init(&mut deps, env.clone(), info.clone(), init_msg.clone()).unwrap();
        let msg = HandleMsg::CreateProposal {
            title: String::from("test"),
            description: String::from("test"),
            metadata: String::from("test"),
            fund_address: HumanAddr::from("fund_address"),
        };

        let res = handle(&mut deps, env.clone(), info.clone(), msg.clone());
        // success case
        match res {
            Ok(seq) => assert_eq!(seq.data.unwrap(), Binary::from(1_u64.to_be_bytes())),
            e => panic!("unexpected error, got {}", e.unwrap_err()),
        }

        // proposal period expired
        env.block.height = env.block.height + 1000;
        let res = handle(&mut deps, env.clone(), info.clone(), msg.clone());

        match res {
            Ok(_) => panic!("expected error"),
            Err(ContractError::ProposalPeriodExpired {}) => {}
            e => panic!("unexpected error, got {}", e.unwrap_err()),
        }

        // unauthorised
        let env = mock_env();
        let info = mock_info("true", &[coin(1000, "ucosm")]);
        let mut deps = mock_dependencies(&[]);
        let init_msg = InitMsg {
            create_proposal_whitelist: Some(vec![HumanAddr::from("false")]),
            ..Default::default()
        };
        init(&mut deps, env.clone(), info.clone(), init_msg.clone()).unwrap();

        let res = handle(&mut deps, env.clone(), info.clone(), msg.clone());

        match res {
            Ok(_) => panic!("expected error"),
            Err(ContractError::Unauthorized {}) => {}
            e => panic!("unexpected error, got {}", e.unwrap_err()),
        }
    }

    #[test]
    fn vote_proposal() {
        let mut env = mock_env();
        let info = mock_info("addr", &[coin(1000, "ucosm")]);
        let mut deps = mock_dependencies(&[]);

        let mut init_msg = InitMsg {
            create_proposal_whitelist: None,
            vote_proposal_whitelist: None,
            voting_period: Expiration::AtHeight(env.block.height + 15),
            proposal_period: Expiration::AtHeight(env.block.height + 10),
            coin_denom: "ucosm".to_string(),
        };
        init(&mut deps, env.clone(), info.clone(), init_msg.clone()).unwrap();

        let create_proposal_msg = HandleMsg::CreateProposal {
            title: String::from("test"),
            description: String::from("test"),
            metadata: String::from("test"),
            fund_address: HumanAddr::from("fund_address"),
        };

        let res = handle(
            &mut deps,
            env.clone(),
            info.clone(),
            create_proposal_msg.clone(),
        );
        match res {
            Ok(seq) => assert_eq!(seq.data.unwrap(), Binary::from(1_u64.to_be_bytes())),
            e => panic!("unexpected error, got {}", e.unwrap_err()),
        }

        let msg = HandleMsg::VoteProposal { proposal_id: 1 };
        let res = handle(&mut deps, env.clone(), info.clone(), msg.clone());
        // success case
        match res {
            Ok(_) => {}
            e => panic!("unexpected error, got {}", e.unwrap_err()),
        }

        // whitelist check
        let mut deps = mock_dependencies(&[]);
        init_msg.vote_proposal_whitelist = Some(vec![HumanAddr::from("admin")]);
        init(&mut deps, env.clone(), info.clone(), init_msg.clone()).unwrap();
        let res = handle(&mut deps, env.clone(), info.clone(), msg.clone());
        match res {
            Ok(_) => panic!("expected error"),
            Err(ContractError::Unauthorized {}) => {}
            e => panic!("unexpected error, got {}", e.unwrap_err()),
        }

        // proposal period expired
        let mut deps = mock_dependencies(&[]);
        init_msg.vote_proposal_whitelist = None;
        init(&mut deps, env.clone(), info.clone(), init_msg.clone()).unwrap();
        env.block.height = env.block.height + 15;
        let res = handle(&mut deps, env.clone(), info.clone(), msg.clone());

        match res {
            Ok(_) => panic!("expected error"),
            Err(ContractError::VotingPeriodExpired {}) => {}
            e => panic!("unexpected error, got {}", e.unwrap_err()),
        }
    }
}
