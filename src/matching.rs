use crate::error::ContractError;
use crate::state::Proposal;
use cosmwasm_std::{coin, Coin, HumanAddr};
use num_integer::Roots;

trait QFAlgorithm {
    fn distribute(
        &self,
        grants: Vec<(Proposal, Vec<u128>)>,
        denom: String,
        budget: Option<u128>,
    ) -> Result<Vec<(HumanAddr, Coin)>, ContractError>;
}

struct CLR;

impl QFAlgorithm for CLR {
    // takes (proposal, votes) tuple vector returns (fund address, coin) to be executed
    fn distribute(
        &self,
        grants: Vec<(Proposal, Vec<u128>)>,
        denom: String,
        budget: Option<u128>,
    ) -> Result<Vec<(HumanAddr, Coin)>, ContractError> {
        // clr algorithm works with budget constrain
        if budget.is_none() {
            return Err(ContractError::CLRConstrainRequired {});
        }

        // calculate matches sum
        let matched = CLR::calculate_matched_sum(grants.clone());
        println!("summed {}", matched);
        // setup a divisor based on available match
        println!("budget {} matched {}", budget.unwrap(), matched);
        let divisor = budget.unwrap() / matched;
        println!("divisor {}", divisor);

        let final_match = CLR::mul_matches_divisor(grants, divisor);
        println!("final match {}", divisor);

        let res = CLR::sanitize_result(denom, final_match);
        Ok(res)
    }
}

impl CLR {
    // takes square root of each fund, sums, then squares and returns u128
    fn calculate_matched_sum(grants: Vec<(Proposal, Vec<u128>)>) -> u128 {
        let mut sum = 0u128;
        for g in grants {
            for vote in g.1 {
                sum += vote.sqrt()
            }
        }
        sum * sum
    }

    // multiply matched values with divisor to get match amount in range of available funds
    fn mul_matches_divisor(
        grants: Vec<(Proposal, Vec<u128>)>,
        divisor: u128,
    ) -> Vec<(Proposal, u128)> {
        grants
            .iter()
            .map(|g| {
                let (p, vs) = g;
                let proposal_fund: u128 = vs.iter().map(|v| v * divisor).sum();
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
    use crate::matching::{QFAlgorithm, CLR};
    use crate::state::Proposal;
    use cosmwasm_std::{coin, HumanAddr};

    #[test]
    fn test_clr() {
        let algo = CLR {};
        let proposal1 = Proposal {
            fund_address: HumanAddr::from("proposal1"),
            ..Default::default()
        };
        let proposal2 = Proposal {
            fund_address: HumanAddr::from("proposal2"),
            ..Default::default()
        };
        let proposal3 = Proposal {
            fund_address: HumanAddr::from("proposal3"),
            ..Default::default()
        };
        let proposal4 = Proposal {
            fund_address: HumanAddr::from("proposal4"),
            ..Default::default()
        };
        let votes1 = vec![7200u128];
        let votes2 = vec![12345u128];
        let votes3 = vec![4456u128];
        let votes4 = vec![60000u128];

        let grants = vec![
            (proposal1.clone(), votes1),
            (proposal2.clone(), votes2),
            (proposal3.clone(), votes3),
            (proposal4.clone(), votes4),
        ];
        let expected = vec![
            (proposal1.fund_address, coin(4838677u128, "ucosm")),
            (proposal2.fund_address, coin(829632u128, "ucosm")),
            (proposal3.fund_address, coin(299460u128, "ucosm")),
            (proposal4.fund_address, coin(40322317u128, "ucosm")),
        ];
        let res = algo.distribute(grants, String::from("ucosm"), Some(100000u128));
        match res {
            Ok(o) => assert_eq!(o, expected),
            e => panic!("unexpected error, got {}", e.unwrap_err()),
        }
    }
    #[test]
    fn test_calculate_liberal_matches() {
        let proposal1 = Proposal {
            ..Default::default()
        };
        let proposal2 = Proposal {
            ..Default::default()
        };
        let grants = vec![
            (proposal1, vec![30000000u128]),
            (proposal2.clone(), vec![40000000u128]),
        ];
        let lm = CLR::calculate_matched_sum(grants);
        assert_eq!(lm, 157452304u128)
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
