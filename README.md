# CosmWasm Quadratic Funding

A CosmWasm smart contract, written in Rust, that instantiates a “Quadratic Funding Round”, and has the following functionality:

- Coins sent with contract instantiation goes into the common funding pool, to be reallocated based on the Quadratic Funding formula.
- Contract parameterized by a list of whitelisted addresses who can make proposals, and a list of whitelisted addresses who can vote on proposals. Empty list means permissionless proposals/voting.
- Proposers can make text based proposals via contract function calls, and set the address that funds should be sent to.
- Proposal periods / voting periods are either defined in advance by contract parameters, or are explicitly triggered via function calls from contract creator / admin.
    If periods are triggered via function calls, minimum proposal periods / voting periods should be set upon contract instantiation.
- Voters vote on proposals by sending coins to the contract in a function call referencing a proposal.
- Once voting period ends, contract creator / admin triggers the distribution of funds to proposals according to a quadratic funding formula.
- A web based user interface (using CosmJS) with the following functionality:
- Allows instantiation of a new contract / creation of new proposal round
- Enables sending of proposals, and voting on proposals
- Enables viewing all proposals and votes for a given contract / Quadratic Funding Round
- Video demo showcasing functionality of your Quadratic Funding dApp!

## Bonus Points

- [ ] Support for alternative funding formulas (besides the standard quadratic funding formula)
- [ ] Support for structured proposal metadata
- [ ] Support for multiple funding rounds per contract
- [ ] Variable proposal periods / voting periods
- [ ] Support for more fine grained queries like “get proposal text/metadata by proposal ID”
- [ ] Regen Network / OpenTEAM logos & branding represented in the UI
- [ ] Deploy your contract to the CosmWasm coral testnet, and share a working link to your dApp

First iteration will only support single type of native coin.

## Messages

```rust
pub struct InitMsg {
    create_proposal_whitelist: Option<Vec<HumanAddr>>,
    vote_proposal_whitelist: Option<Vec<HumanAddr>>,
    voting_period: Expiration,
    proposal_period: Expiration,
    coin_denom: String,
}

enum HandleMsg {
    CreateProposal {
        description: String,
        metadata: String,
        fund_address: HumanAddr
    },
    VoteProposal {
        proposal_id: u32,
    },
    TriggerDistribution {
        proposal_id: u32
    },
}
```

### State

```rust
pub struct Proposal {
    id: u32,
    title: String,
    metadata: String,
    fund_address: HumanAddr,
}

pub struct Vote {
    id: u32,
    proposal_id: u32,
    voter: HumanAddr,
    fund: Coin,
}
```

### Queries

```rust
enum QueryMsg {
    ProposalByID {
        id: u64,
    },
    ProposalByFundAddress {
        fund_address: HumanAddr
    },
    AllProposals {},
}
```

## Iteration 2

Support CW20
