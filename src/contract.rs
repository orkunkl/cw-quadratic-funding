use cosmwasm_std::{
    Api, Binary, Env, Extern, HandleResponse, InitResponse, MessageInfo, Querier, StdResult,
    Storage,
};

use crate::error::ContractError;
use crate::msg::{HandleMsg, InitMsg, QueryMsg};
use crate::state::{config, state, Config};

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
    config(&mut deps.storage).save(&cfg)?;

    Ok(InitResponse::default())
}

// And declare a custom Error variant for the ones where you will want to make use of it
pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    info: MessageInfo,
    msg: HandleMsg,
) -> Result<HandleResponse, ContractError> {
    match msg {
        HandleMsg::Increment {} => try_increment(deps),
        HandleMsg::Reset { count } => try_reset(deps, info, count),
    }
}

pub fn try_increment<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
) -> Result<HandleResponse, ContractError> {
    state(&mut deps.storage).update(|mut state| -> Result<_, ContractError> {
        state.count += 1;
        Ok(state)
    })?;

    Ok(HandleResponse::default())
}

pub fn try_reset<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    info: MessageInfo,
    count: i32,
) -> Result<HandleResponse, ContractError> {
    let api = &deps.api;
    state(&mut deps.storage).update(|mut state| -> Result<_, ContractError> {
        if api.canonical_address(&info.sender)? != state.owner {
            return Err(ContractError::Unauthorized {});
        }
        state.count = count;
        Ok(state)
    })?;
    Ok(HandleResponse::default())
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    _deps: &Extern<S, A, Q>,
    _env: Env,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCount { .. } => {}
        QueryMsg::ProposalByID { .. } => {}
        QueryMsg::ProposalByFundAddress { .. } => {}
        QueryMsg::AllProposals { .. } => {}
    }
    return Ok(Binary::from(b"1"));
}

#[cfg(test)]
mod tests {}
