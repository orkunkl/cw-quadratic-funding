use crate::error::ContractError;
use cosmwasm_std::Coin;

pub fn extract_funding_coin(sent_funds: &[Coin]) -> Result<Coin, ContractError> {
    if sent_funds.len() != 1 {
        return Err(ContractError::MultipleCoinsSent {});
    }
    Ok(sent_funds[0].clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::helper::extract_funding_coin;
    use cosmwasm_std::coin;
    use cosmwasm_std::testing::mock_info;

    #[test]
    fn test_extract_funding_coin() {
        let denom = "denom";
        let c = &[coin(4, denom)];
        let info = mock_info("creator", c);

        let res = extract_funding_coin(&info.sent_funds);
        match res {
            Ok(cc) => assert_eq!(c, &[cc]),
            Err(err) => println!("{:?}", err),
        }
        let info = mock_info("creator", &[coin(4, denom)]);

        match extract_funding_coin(&info.clone().sent_funds) {
            Ok(_) => panic!("expected error"),
            Err(ContractError::MultipleCoinsSent { .. }) => {}
            Err(err) => println!("{:?}", err),
        }
    }
}
