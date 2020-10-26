use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Proposal period expired")]
    ProposalPeriodExpired {},

    #[error("Voting period expired")]
    VotingPeriodExpired {},

    #[error("Voting period not expired")]
    VotingPeriodNotExpired {},

    #[error("Expected coin not sent (expected: {coin_denom})")]
    ExpectedCoinNotSent { coin_denom: String },
}
