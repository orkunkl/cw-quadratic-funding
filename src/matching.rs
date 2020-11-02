use crate::error::ContractError;
use cosmwasm_std::CanonicalAddr;
use integer_sqrt::IntegerSquareRoot;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QuadraticFundingAlgorithm {
    CapitalConstrainedLiberalRadicalism { parameter: String },
}

type CollectedVoteFunds = u128;
type Fund = u128;
type CLRAdjustedDistr = u128;

struct RawGrant {
    addr: CanonicalAddr,
    funds: Vec<u128>,
    collected_vote_funds: u128
}

struct CalculatedGrant {
    addr: CanonicalAddr,
    grant: u128,
    collected_vote_funds: u128
}

type LeftOver = u128;

// TODO rename u128 to something meaningful
pub fn calculate_clr(
    grants: Vec<RawGrant>,
    budget: Option<u128>,
) -> Result<(Vec<SummedGrant>, LeftOver), ContractError> {
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
fn calculate_matched_sum(
    grants: Vec<RawGrant>,
) -> Vec<SummedGrant> {
    grants
        .into_iter()
        .map(|g| {
            let sum_sqrts: u128 = g.votes.into_iter().map(|v| v.integer_sqrt()).sum();
            CalculatedGrant{
                addr: g.addr,
                grant:  sum_sqrts * sum_sqrts,
                collected_vote_funds: collected_fund
            }
        })
        .collect()
}

// takes square root of each fund, sums, then squares and returns u128
fn constrain_by_budget(
    grants: Vec<(CanonicalAddr, Fund, CollectedVoteFunds)>,
    budget: u128,
) -> Vec<(CanonicalAddr, Fund, CollectedVoteFunds)> {
    let raw_total: u128 = grants.iter().map(|g| g.1).sum();
    grants
        .into_iter()
        .map(|g| {
            let (proposal, grant, collected_fund) = g;
            (proposal, (grant * budget) / raw_total, collected_fund)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::matching::{calculate_clr, RawGrant, CalculatedGrant};
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
            CalculatedGrant{
                addr: proposal1.fund_address.clone(),
                grant:
                collected_vote_funds: 0,
            }
            (proposal1.fund_address.clone(), votes1, 7200u128),
            (proposal2.fund_address.clone(), votes2, 12345u128),
            (proposal3.fund_address.clone(), votes3, 4456u128),
            (proposal4.fund_address.clone(), votes4, 60000u128),
        ];
        let expected = vec![
            (proposal1.fund_address, 84737u128, 7200u128),
            (proposal2.fund_address, 147966u128, 12345u128),
            (proposal3.fund_address, 52312u128, 4456u128),
            (proposal4.fund_address, 714983u128, 60000u128),
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
            (
                proposal1.fund_address.clone(),
                votes1.clone(),
                votes1.clone().iter().sum(),
            ),
            (
                proposal2.fund_address.clone(),
                votes2.clone(),
                votes2.clone().iter().sum(),
            ),
            (
                proposal3.fund_address.clone(),
                votes3.clone(),
                votes3.clone().iter().sum(),
            ),
            (
                proposal4.fund_address.clone(),
                votes4.clone(),
                votes4.clone().iter().sum(),
            ),
        ];
        let expected = vec![
            (proposal1.fund_address, 60212u128, votes1.iter().sum()),
            (proposal2.fund_address, 164602u128, votes2.iter().sum()),
            (proposal3.fund_address, 228537u128, votes3.iter().sum()),
            (proposal4.fund_address, 96648u128, votes4.iter().sum()),
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
