use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub enum Status {
    Open,
    Closed,
}

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub denom: String,
    pub commission: u128,
}

#[cw_serde]
pub struct State {
    pub current_status: Status,
    pub highest_bid: Option<(Addr, Uint128)>,
}

pub const STATE: Item<State> = Item::new("state");
pub const CONFIG: Item<Config> = Item::new("config");
pub const BIDS: Map<&Addr, Uint128> = Map::new("bids");
