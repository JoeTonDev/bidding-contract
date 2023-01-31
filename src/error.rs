use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized { owner: String },

    #[error("Unauthorized bid")]
    UnauthorizedBid { owner: String },

    #[error("Invalid retract")]
    InvalidRetract,

    #[error("Bidding closed")]
    BiddingClosed,

    #[error("Bidding is active")]
    BiddingActive,

    #[error("Invalid commission")]
    InvalidCommission { funds: Uint128, commission: Uint128 },

    #[error("Invalid funds")]
    InvalidFunds,

    #[error("Invalid bid")]
    InvalidBid {
        existing: Uint128,
        funds: Uint128,
        new_bid: Uint128,
        max_bid: Uint128,
    },
}
