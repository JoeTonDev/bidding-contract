use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
    pub denom: String,
    pub commission: Option<u128>,
}

#[cw_serde]
pub enum ExecuteMsg {
    Bid {},
    Close {},
    Retract { receiver: Option<String> },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Uint128)]
    TotalBids { address: String },
    #[returns(BidResponse)]
    HighestBid {},
    #[returns(bool)]
    BiddingCompleted {},
    #[returns(BidResponse)]
    WinningBid {},
}

#[cw_serde]
pub struct BidResponse {
    pub address: Addr,
    pub amount: Uint128,
}
