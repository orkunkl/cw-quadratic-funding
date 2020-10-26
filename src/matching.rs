use cosmwasm_std::{Extern, Querier, Api, Storage, Order, HumanAddr, Coin, StdResult, Uint128};
use crate::state::{PROPOSALS, Proposal, Vote};

trait QuadraticFundingMatchingAlgorithm {
    fn distribute(&self, grants: &[(Proposal, &[Vote])]) -> StdResult<Vec<(HumanAddr, Coin)>>;
}

struct CLR;

impl QuadraticFundingMatchingAlgorithm for CLR {
    // takes (proposal, votes) tuple vector returns (fund address, coin) to be executed
    fn distribute(&self, grants: Vec<(Proposal, Vec<Vote>)>) -> StdResult<Vec<(HumanAddr, Coin)>>{
        
        // sum funding round grants
        let sum:Uint128 = grants.iter()
            .map(|g| {
                g.1.iter().map(|x| x.fund)
                    .collect();
            }).collect();


        // calculate liberal match for each proposal

        // constrain matches to budget


        /*
        raw_grants = process_raw_data(projects, backers, contribution_amounts)
        grants = aggregate(raw_grants)
        project_grant_sums = project_grant_sum(grants)
        lr_matches = calc_lr_matches(grants)
        clr = constrain_by_budget(lr_matches, budget)

        return grants, lr_matches, clr

         */
        unimplemented!();
    }
}

/*

def process_raw_data(projects, backers, contributions):
    hex_backers = check_addresses(backers)
    chiDAI_contribs = to_chiDAI(contributions)
    return list(zip(projects, hex_backers, chiDAI_contribs))

def aggregate(grants):
    aggregated = {}
    for project, backer, contribution in grants:
        if project not in aggregated:
            aggregated[project] = {}
        aggregated[project][backer] = aggregated[project].get(backer, 0) + contribution
    return aggregated
 */
