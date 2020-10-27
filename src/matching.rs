use crate::state::Proposal;
use cosmwasm_std::{coin, Coin, HumanAddr, StdResult, Uint128};
use num_integer::Roots; // this adds magic powers to u128

trait QuadraticFundingMatchingAlgorithm {
    fn distribute(
        &self,
        grants: Vec<(Proposal, Vec<Uint128>)>,
        denom: String,
        budget: Uint128,
    ) -> StdResult<Vec<(HumanAddr, Coin)>>;
}

struct CLR;

impl QuadraticFundingMatchingAlgorithm for CLR {
    // takes (proposal, votes) tuple vector returns (fund address, coin) to be executed
    fn distribute(
        &self,
        grants: Vec<(Proposal, Vec<Uint128>)>,
        denom: String,
        budget: Uint128,
    ) -> StdResult<Vec<(HumanAddr, Coin)>> {
        // calculate matches sum
        let summed = CLR::calculate_matched_sum(grants.clone());

        // setup a divisor based on available match
        let divisor = budget.u128() / summed;

        let final_match = CLR::mul_matches_divisor(grants, divisor);

        let res = CLR::sanitize_result(denom, final_match);
        Ok(res)
    }
}

impl CLR {
    // takes square root of each fund, sums, then squares and returns u128
    fn calculate_matched_sum(grants: Vec<(Proposal, Vec<Uint128>)>) -> u128 {
        let mut sum = 0u128;
        for g in grants {
            for vote in g.1 {
                sum += vote.u128().sqrt()
            }
        }
        sum * sum
    }

    // multiply matched values with divisor to get match amount in range of available funds
    fn mul_matches_divisor(
        grants: Vec<(Proposal, Vec<Uint128>)>,
        divisor: u128,
    ) -> Vec<(Proposal, u128)> {
        grants
            .iter()
            .map(|g| {
                let (p, vs) = g;
                let proposal_fund: u128 = vs.iter().map(|v| v.u128() * divisor).sum();
                (p.clone(), proposal_fund)
            })
            .collect()
    }

    // sanitize result for handler to process.
    fn sanitize_result(
        denom: String,
        final_match: Vec<(Proposal, u128)>,
    ) -> Vec<(HumanAddr, Coin)> {
        final_match
            .iter()
            .map(|g| {
                let (p, f) = g;
                let fund_addr = p.clone().fund_address;
                let c = coin(*f, denom.as_str());
                (fund_addr, c)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::matching::CLR;
    use crate::state::Proposal;
    use cosmwasm_std::Uint128;

    #[test]
    fn test_calculate_liberal_matches() {
        let proposal1 = Proposal {
            ..Default::default()
        };
        let proposal2 = Proposal {
            ..Default::default()
        };
        let grants = vec![
            (proposal1, vec![Uint128(30000000)]),
            (proposal2, vec![Uint128(1000)]),
        ];
        let lm = CLR::calculate_matched_sum(grants);
        assert_eq!(lm, 30338064u128)
    }
    /*
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


    */
}
