use crate::state::{Proposal, Vote, PROPOSALS};
use cosmwasm_std::{Api, Coin, Extern, HumanAddr, Order, Querier, StdResult, Storage, Uint128};
use num_integer::Roots;

trait QuadraticFundingMatchingAlgorithm {
    fn distribute(
        &self,
        grants: Vec<(Proposal, Vec<Vote>)>,
        budget: Uint128,
    ) -> StdResult<Vec<(HumanAddr, Coin)>>;
}

struct CLR;

impl QuadraticFundingMatchingAlgorithm for CLR {
    // takes (proposal, votes) tuple vector returns (fund address, coin) to be executed
    fn distribute(
        &self,
        grants: Vec<(Proposal, Vec<Vote>)>,
        budget: Uint128,
    ) -> StdResult<Vec<(HumanAddr, Coin)>> {
        // aggregate grant funds
        let grant_funds = aggregate_funding_round_grants(grants);

        // sum funding round grants
        let funding_round_sum: Uint128 = sum_total_round_grants(grant_funds);

        // calculate liberal match for each proposal
        /*
                let lr_matches: Vec<(Proposal, u128)> = grant_funds
                    .iter()
                    .map(|g| {
                        let (proposal, votes_sum) = g;
                        let sum_sqrts = votes_sum.u128().sqrt();
                        let squared = sum_sqrts * sum_sqrts;
                        return (proposal, squared);
                    })
                    .collect();

                // constrain matches to budget
                let raw_total: u128 = lr_matches.iter().map(|lr| lr.1).sum();
        */
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

fn aggregate_funding_round_grants(grants: Vec<(Proposal, Vec<Vote>)>) -> Vec<(Proposal, Uint128)> {
    grants
        .iter()
        .map(|g| {
            let (proposal, votes) = g;

            let votes_sum: Uint128 = votes.iter().map(|v| v.fund.amount).sum();
            (proposal.clone(), votes_sum.clone())
        })
        .collect::<Vec<_>>()
}

fn sum_total_round_grants(grants: Vec<(Proposal, Uint128)>) -> Uint128 {
    grants.iter().map(|g| g.1).sum()
}

fn calculate_liberal_matches(grants: Vec<(Proposal, Vec<Vote>)>) -> Vec<(Proposal, u128)> {
    /*
    let lr_matches: Vec<(Proposal, u128)> = grant_funds
        .iter()
        .map(|g| {
            let (proposal, votes_sum) = g;
            let sum_sqrts = votes_sum.u128().sqrt();
            let squared = sum_sqrts * sum_sqrts;
            return (proposal, squared);
        })
        .collect();

     */
    grants
        .iter()
        .map(|g| {
            let (proposal, votes) = g;
            let sum_sqrts = votes.iter().map(|v| v.fund.amount.u128().sqrt()).sum();
            let squared = sum_sqrts * sum_sqrts;
            (proposal.clone(), squared)
        })
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use crate::matching::{aggregate_funding_round_grants, sum_total_round_grants, calculate_liberal_matches};
    use crate::state::{Proposal, Vote};
    use cosmwasm_std::{Coin, HumanAddr, Uint128};

    #[test]
    fn test_aggregate_funding_round_grants() {
        let proposal1 = Proposal {
            title: String::from("proposal 1"),
            description: String::from("desc"),
            metadata: String::from(""),
            fund_address: HumanAddr::from("fund_address1"),
        };
        let votes1 = vec![
            Vote {
                proposal_id: 0,
                voter: HumanAddr::from("address1"),
                fund: Coin {
                    denom: String::from("ucosm"),
                    amount: Uint128(1000),
                },
            },
            Vote {
                proposal_id: 0,
                voter: HumanAddr::from("address2"),
                fund: Coin {
                    denom: String::from("ucosm"),
                    amount: Uint128(2000),
                },
            },
            Vote {
                proposal_id: 0,
                voter: HumanAddr::from("address3"),
                fund: Coin {
                    denom: String::from("ucosm"),
                    amount: Uint128(3000),
                },
            },
        ];
        let proposal2 = Proposal {
            title: String::from("proposal 2"),
            description: String::from("desc"),
            metadata: String::from(""),
            fund_address: HumanAddr::from("fund_address2"),
        };
        let votes2 = vec![
            Vote {
                proposal_id: 0,
                voter: HumanAddr::from("address4"),
                fund: Coin {
                    denom: String::from("ucosm"),
                    amount: Uint128(4000),
                },
            },
            Vote {
                proposal_id: 0,
                voter: HumanAddr::from("address5"),
                fund: Coin {
                    denom: String::from("ucosm"),
                    amount: Uint128(5000),
                },
            },
            Vote {
                proposal_id: 0,
                voter: HumanAddr::from("address6"),
                fund: Coin {
                    denom: String::from("ucosm"),
                    amount: Uint128(5000),
                },
            },
        ];
        let grants = vec![(proposal1.clone(), votes1), (proposal2.clone(), votes2)];
        let expected = vec![(proposal1, Uint128(6000)), (proposal2, Uint128(14000))];
        let sum = aggregate_funding_round_grants(grants);
        assert_eq!(sum, expected);
    }

    #[test]
    fn test_sum_funding_round_grants() {
        let proposal1 = Proposal {
            title: String::from("proposal 1"),
            description: String::from("desc"),
            metadata: String::from(""),
            fund_address: HumanAddr::from("fund_address1"),
        };
        let total_grant1 = Uint128(12000);
        let proposal2 = Proposal {
            title: String::from("proposal 1"),
            description: String::from("desc"),
            metadata: String::from(""),
            fund_address: HumanAddr::from("fund_address1"),
        };
        let total_grant2 = Uint128(12000);
        let expected = total_grant1 + total_grant2;
        let sum =
            sum_total_round_grants(vec![(proposal1, total_grant1), (proposal2, total_grant2)]);
        assert_eq!(sum, expected)
    }

    #[test]
    fn test_calculate_liberal_matches() {
        let proposal1 = Proposal {
            title: String::from("proposal 1"),
            description: String::from("desc"),
            metadata: String::from(""),
            fund_address: HumanAddr::from("fund_address1"),
        };
        let proposal2 = Proposal {
            title: String::from("proposal 1"),
            description: String::from("desc"),
            metadata: String::from(""),
            fund_address: HumanAddr::from("fund_address1"),
        };
        let grants = vec![
            (proposal1.clone(), Uint128(3000)),
                (proposal2.clone(), Uint128(1000))
        ];
        let expected = vec![
            (proposal1, 129),
            (proposal2, 129),
        ];
        let lm = calculate_liberal_matches(grants);
        assert_eq!(lm, expected)
    }
}

/*
def constrain_by_budget(matches, budget):
raw_total = sum(matches.values())
constrained = {key:value/raw_total * budget for key, value in matches.items()}
return constrained

def calc_lr_matches(grants):
matches = {}
for project in grants:
    sum_sqrts = sum([i**(1/2) for i in grants[project].values()])
    squared = sum_sqrts**2
    matches[project] = squared
return matches

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
