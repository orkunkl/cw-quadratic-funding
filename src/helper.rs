use crate::error::ContractError;
use cosmwasm_std::Coin;

pub fn extract_funding_coin(sent_funds: &[Coin], denom: String) -> Result<Coin, ContractError> {
    // check of funding coin_denom matches sent_funds
    // maybe throw error when unexpected coin is found?
    sent_funds
        .iter()
        .map(|x| x.denom.clone())
        .position(|x| x == denom)
        .and_then(|p| sent_funds.get(p))
        .cloned()
        .ok_or(ContractError::ExpectedCoinNotSent { coin_denom: denom })
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

        let res = extract_funding_coin(&info.sent_funds, denom.to_string());
        match res {
            Ok(cc) => assert_eq!(c, &[cc]),
            Err(err) => println!("{:?}", err),
        }
        let info = mock_info("creator", &[coin(4, denom)]);

        match extract_funding_coin(&info.clone().sent_funds, String::from("false")) {
            Ok(_) => panic!("expected error"),
            Err(ContractError::ExpectedCoinNotSent { .. }) => {}
            Err(err) => println!("{:?}", err),
        }
    }
}
