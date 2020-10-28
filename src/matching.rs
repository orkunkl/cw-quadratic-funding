use crate::error::ContractError;
use crate::state::Proposal;
use cosmwasm_std::{coin, Coin, HumanAddr};
use num_integer::Roots;

trait QFAlgorithm {
    // takes grantes denom and budget, returns "fund_address" -> "grant" vector and left over tokens to return
    fn distribute(
        &self,
        grants: Vec<(Proposal, Vec<u128>)>,
        denom: String,
        budget: Option<u128>,
    ) -> Result<(Vec<(HumanAddr, Coin)>, Coin), ContractError>;
}

struct CLR;

impl QFAlgorithm for CLR {
    // takes (proposal, votes) tuple vector returns (fund address, coin) to be executed
    fn distribute(
        &self,
        grants: Vec<(Proposal, Vec<u128>)>,
        denom: String,
        budget: Option<u128>,
    ) -> Result<(Vec<(HumanAddr, Coin)>, Coin), ContractError> {
        // clr algorithm works with budget constrain
        if budget.is_none() {
            return Err(ContractError::CLRConstrainRequired {});
        }

        // calculate matches sum
        let matched = CLR::calculate_matched_sum(grants.clone());

        // constraint the grants by budget
        let constrained = CLR::constrain_by_budget(matched, budget.unwrap());

        // calculate leftover
        let constrained_sum: u128 = constrained.iter().map(|c| c.1).sum();
        let leftover = coin(budget.unwrap() - constrained_sum, denom.as_str());
        // sanitize result
        let res = CLR::sanitize_result(denom, constrained);
        Ok((res, leftover))
    }
}

impl CLR {
    // takes square root of each fund, sums, then squares and returns u128
    fn calculate_matched_sum(grants: Vec<(Proposal, Vec<u128>)>) -> Vec<(Proposal, u128)> {
        grants.iter()
            .map(|g| {
                let (proposal, votes) = g;
                let sum_sqrts:u128 = votes.iter().map(|v| v.sqrt()).sum();
                (proposal.clone(), sum_sqrts * sum_sqrts)
            }).collect()
    }

    // takes square root of each fund, sums, then squares and returns u128
    fn constrain_by_budget(grants: Vec<(Proposal, u128)>, budget: u128) -> Vec<(Proposal, u128)>{
        let raw_total:u128 = grants.iter().map(|g| g.1).sum();
        grants.iter()
            .map(|g| {
                let (proposal, grant) = g;
                (proposal.clone(), (grant * budget) / raw_total)
            }).collect()
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
    fn test_clr_1() {
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
            (proposal1.fund_address, coin(84737u128, "ucosm")),
            (proposal2.fund_address, coin(147966u128, "ucosm")),
            (proposal3.fund_address, coin(52312u128, "ucosm")),
            (proposal4.fund_address, coin(714983u128, "ucosm")),
        ];
        let res = algo.distribute(grants, String::from("ucosm"), Some(1000000u128));
        match res {
            Ok(o) => {
            assert_eq!(o.0, expected);
            assert_eq!(o.1, coin(1, "ucosm"))
            },
            e => panic!("unexpected error, got {}", e.unwrap_err()),
        }
    }

    // values got from https://wtfisqf.com/?grant=1200,44999,33&grant=30000,58999&grant=230000,100&grant=100000,5&match=550000
    //        expected   got
    // grant1 60673.38   60212
    // grant2 164749.05  164602
    // grant3 228074.05  228537
    // grant4 96503.53   96648
    #[test]
    fn test_clr_2() {
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
        let votes1 = vec![1200u128, 44999u128, 33u128];
        let votes2 = vec![30000u128, 58999u128];
        let votes3 = vec![230000u128, 100u128];
        let votes4 = vec![100000u128, 5u128];

        let grants = vec![
            (proposal1.clone(), votes1),
            (proposal2.clone(), votes2),
            (proposal3.clone(), votes3),
            (proposal4.clone(), votes4),
        ];
        let expected = vec![
            (proposal1.fund_address, coin(60212u128, "ucosm")),
            (proposal2.fund_address, coin(164602u128, "ucosm")),
            (proposal3.fund_address, coin(228537u128, "ucosm")),
            (proposal4.fund_address, coin(96648u128, "ucosm")),
        ];
        let res = algo.distribute(grants, String::from("ucosm"), Some(550000u128));
        match res {
            Ok(o) => {
                assert_eq!(o.0, expected);
                assert_eq!(o.1, coin(1, "ucosm"))
            }
            e => panic!("unexpected error, got {}", e.unwrap_err()),
        }
    }
}
