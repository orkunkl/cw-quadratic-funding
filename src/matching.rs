use crate::error::ContractError;
use cosmwasm_std::CanonicalAddr;
use integer_sqrt::IntegerSquareRoot;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QFAlgorithm {
    CapitalConstrainedLiberalRadicalism {},
}

pub fn calculate_clr(
    grants: Vec<(CanonicalAddr, Vec<u128>)>,
    budget: Option<u128>,
) -> Result<(Vec<(CanonicalAddr, u128)>, u128), ContractError> {
    // clr algorithm works with budget constrain
    if let Some(budget) = budget {
        // calculate matches sum
        let matched = calculate_matched_sum(grants);

        // constraint the grants by budget
        let constrained = constrain_by_budget(matched, budget);

        // calculate leftover
        let constrained_sum: u128 = constrained.iter().map(|c| c.1).sum();
        // shouldn't be used with tokens with > 10 decimal points
        // will cause overflow and panic on the during execution.
        let leftover = budget - constrained_sum;
        Ok((constrained, leftover))
    } else {
        Err(ContractError::CLRConstrainRequired {})
    }
}

// takes square root of each fund, sums, then squares and returns u128
fn calculate_matched_sum(grants: Vec<(CanonicalAddr, Vec<u128>)>) -> Vec<(CanonicalAddr, u128)> {
    grants
        .into_iter()
        .map(|g| {
            let (proposal, votes) = g;
            let sum_sqrts: u128 = votes.into_iter().map(|v| v.integer_sqrt()).sum();
            (proposal, sum_sqrts * sum_sqrts)
        })
        .collect()
}

// takes square root of each fund, sums, then squares and returns u128
fn constrain_by_budget(
    grants: Vec<(CanonicalAddr, u128)>,
    budget: u128,
) -> Vec<(CanonicalAddr, u128)> {
    let raw_total: u128 = grants.iter().map(|g| g.1).sum();
    grants
        .into_iter()
        .map(|g| {
            let (proposal, grant) = g;
            (proposal, (grant * budget) / raw_total)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::matching::calculate_clr;
    use crate::state::Proposal;
    use cosmwasm_std::CanonicalAddr;

    #[test]
    fn test_clr_1() {
        let proposal1 = Proposal {
            fund_address: CanonicalAddr(b"proposal1".to_vec().into()),
            ..Default::default()
        };
        let proposal2 = Proposal {
            fund_address: CanonicalAddr(b"proposal2".to_vec().into()),
            ..Default::default()
        };
        let proposal3 = Proposal {
            fund_address: CanonicalAddr(b"proposal3".to_vec().into()),
            ..Default::default()
        };
        let proposal4 = Proposal {
            fund_address: CanonicalAddr(b"proposal4".to_vec().into()),
            ..Default::default()
        };
        let votes1 = vec![7200u128];
        let votes2 = vec![12345u128];
        let votes3 = vec![4456u128];
        let votes4 = vec![60000u128];

        let grants = vec![
            (proposal1.fund_address.clone(), votes1),
            (proposal2.fund_address.clone(), votes2),
            (proposal3.fund_address.clone(), votes3),
            (proposal4.fund_address.clone(), votes4),
        ];
        let expected = vec![
            (proposal1.fund_address, 84737u128),
            (proposal2.fund_address, 147966u128),
            (proposal3.fund_address, 52312u128),
            (proposal4.fund_address, 714983u128),
        ];
        let res = calculate_clr(grants, Some(1000000u128));
        match res {
            Ok(o) => {
                assert_eq!(o.0, expected);
                assert_eq!(o.1, 2)
            }
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
        let proposal1 = Proposal {
            fund_address: CanonicalAddr(b"proposal1".to_vec().into()),
            ..Default::default()
        };
        let proposal2 = Proposal {
            fund_address: CanonicalAddr(b"proposal2".to_vec().into()),
            ..Default::default()
        };
        let proposal3 = Proposal {
            fund_address: CanonicalAddr(b"proposal3".to_vec().into()),
            ..Default::default()
        };
        let proposal4 = Proposal {
            fund_address: CanonicalAddr(b"proposal4".to_vec().into()),
            ..Default::default()
        };
        let votes1 = vec![1200u128, 44999u128, 33u128];
        let votes2 = vec![30000u128, 58999u128];
        let votes3 = vec![230000u128, 100u128];
        let votes4 = vec![100000u128, 5u128];

        let grants = vec![
            (proposal1.fund_address.clone(), votes1),
            (proposal2.fund_address.clone(), votes2),
            (proposal3.fund_address.clone(), votes3),
            (proposal4.fund_address.clone(), votes4),
        ];
        let expected = vec![
            (proposal1.fund_address, 60212u128),
            (proposal2.fund_address, 164602u128),
            (proposal3.fund_address, 228537u128),
            (proposal4.fund_address, 96648u128),
        ];
        let res = calculate_clr(grants, Some(550000u128));
        match res {
            Ok(o) => {
                assert_eq!(o.0, expected);
                assert_eq!(o.1, 1)
            }
            e => panic!("unexpected error, got {}", e.unwrap_err()),
        }
    }
}
