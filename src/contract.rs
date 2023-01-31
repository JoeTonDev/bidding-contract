#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    coins, to_binary, BankMsg, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Uint128,
};
use cosmwasm_std::{entry_point, StdError};
use cw2::set_contract_version;
use std::ops::Mul;

use crate::error::ContractError;
use crate::msg::{BidResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, State, Status, BIDS, CONFIG, STATE};

const COMMISSION: u128 = 0.05 as u128; // 5%

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let commission = msg.commission.unwrap_or_default();

    STATE.save(
        deps.storage,
        &State {
            current_status: Status::Open,
            highest_bid: None,
        },
    )?;

    let owner = match msg.owner {
        Some(owner) => deps.api.addr_validate(&owner)?,
        None => info.sender,
    };

    CONFIG.save(
        deps.storage,
        &Config {
            owner,
            denom: msg.denom,
            commission,
        },
    )?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    use ExecuteMsg::{Bid, Close, Retract};

    match msg {
        Bid {} => bid(deps, info),
        Close {} => close(deps, info),
        Retract { receiver } => retract(deps, info, receiver),
    }
}

pub fn bid(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let owner = CONFIG.load(deps.storage)?.owner;
    let denom = CONFIG.load(deps.storage)?.denom;
    let commission = Uint128::new(CONFIG.load(deps.storage)?.commission);

    let mut state = STATE.load(deps.storage)?;
    let highest_bid = state.highest_bid;

    let mut resp = Response::new();

    if state.current_status == Status::Closed {
        return Err(ContractError::BiddingClosed);
    }

    if owner == info.sender {
        return Err(ContractError::UnauthorizedBid {
            owner: owner.to_string(),
        });
    }

    let highest_bid_amount = match highest_bid {
        Some(highest_bid) => highest_bid.1,
        None => Uint128::new(0),
    };

    let funds = match info.funds.iter().find(|coin| coin.denom == denom) {
        Some(funds) => funds.amount,
        None => return Err(ContractError::InvalidFunds),
    };

    if !commission.is_zero() && funds < commission {
        return Err(ContractError::InvalidCommission { funds, commission });
    }

    let comm = Uint128::from(COMMISSION);
    let commission = funds.mul(comm);

    let net_bid = funds - commission;

    let existing_bid = match BIDS.may_load(deps.storage, &info.sender)? {
        Some(existing_bid) => existing_bid,
        None => Uint128::new(0),
    };

    let new_bid = net_bid + existing_bid;
    if new_bid <= highest_bid_amount {
        return Err(ContractError::InvalidBid {
            existing: existing_bid,
            funds,
            new_bid: net_bid,
            max_bid: highest_bid_amount,
        });
    }

    state.highest_bid = Some((info.sender.clone(), new_bid));

    if !commission.is_zero() {
        let funds: Vec<_> = coins(commission.u128(), denom);

        let bank_msg = BankMsg::Send {
            to_address: owner.into_string(),
            amount: funds,
        };

        resp = resp
            .add_message(bank_msg)
            .add_attribute("commission_to_owner", info.sender.as_str());
    }

    BIDS.update(deps.storage, &info.sender, |_| -> StdResult<_> {
        Ok(new_bid)
    })?;

    STATE.save(deps.storage, &state)?;

    resp = resp
        .add_attribute("action", "bid")
        .add_attribute("sender", info.sender.as_str())
        .add_attribute("current_highest_bid", new_bid);

    Ok(resp)
}

pub fn close(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;
    let owner = CONFIG.load(deps.storage)?.owner;
    let denom = CONFIG.load(deps.storage)?.denom;

    let mut resp = Response::new();

    if state.current_status == Status::Closed {
        return Err(ContractError::BiddingClosed);
    }

    if owner != info.sender {
        return Err(ContractError::Unauthorized {
            owner: owner.to_string(),
        });
    }

    if let Some(highest_bid) = state.highest_bid {
        let funds: Vec<_> = coins(highest_bid.1.u128(), denom);

        let bank_msg = BankMsg::Send {
            to_address: owner.into_string(),
            amount: funds,
        };

        BIDS.remove(deps.storage, &highest_bid.0);

        resp = resp
            .add_message(bank_msg)
            .add_attribute("Highest_bid", highest_bid.0.as_str());
    }

    STATE.update(deps.storage, |mut state| -> StdResult<_> {
        state.current_status = Status::Closed;
        Ok(state)
    })?;

    resp = resp
        .add_attribute("action", "close")
        .add_attribute("sender", info.sender.as_str());

    Ok(resp)
}

pub fn retract(
    deps: DepsMut,
    info: MessageInfo,
    receiver: Option<String>,
) -> Result<Response, ContractError> {
    let status = STATE.load(deps.storage)?.current_status;

    if status == Status::Open {
        return Err(ContractError::BiddingActive);
    }

    let funds = match BIDS.load(deps.storage, &info.sender) {
        Ok(amount) => coins(amount.u128(), amount),
        _ => return Err(ContractError::InvalidRetract),
    };

    let receiver = match receiver {
        Some(receiver) => deps.api.addr_validate(&receiver)?,
        None => info.sender.clone(),
    };

    let bank_msg = BankMsg::Send {
        to_address: receiver.to_string(),
        amount: funds,
    };

    BIDS.remove(deps.storage, &receiver);

    Ok(Response::new().add_message(bank_msg).add_attributes(vec![
        ("action", "retract"),
        ("sender", info.sender.as_str()),
        ("retract_funds_to_receiver", receiver.as_str()),
    ]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::TotalBids { address } => to_binary(&self::total_bids(deps, address)?),
        QueryMsg::HighestBid {} => to_binary(&self::highest_bid(deps)?),
        QueryMsg::BiddingCompleted {} => to_binary(&self::bidding_completed(deps)?),
        QueryMsg::WinningBid {} => to_binary(&self::winning_bid(deps)?),
    }
}

pub fn total_bids(deps: Deps, address: String) -> StdResult<Uint128> {
    let address = deps.api.addr_validate(&address)?;

    BIDS.load(deps.storage, &address)
}

pub fn highest_bid(deps: Deps) -> StdResult<BidResponse> {
    match STATE.load(deps.storage)?.highest_bid {
        Some((address, amount)) => Ok(BidResponse { address, amount }),
        None => Err(StdError::not_found("Auction has no bid")),
    }
}

pub fn bidding_completed(deps: Deps) -> StdResult<bool> {
    match STATE.load(deps.storage)?.current_status {
        Status::Closed => Ok(true),
        _ => Ok(false),
    }
}

pub fn winning_bid(deps: Deps) -> StdResult<BidResponse> {
    if STATE.load(deps.storage)?.current_status == Status::Open {
        return Err(StdError::generic_err("Auction is closed"));
    }
    match STATE.load(deps.storage)?.highest_bid {
        Some((address, amount)) => Ok(BidResponse { address, amount }),
        None => Err(StdError::not_found("Auction has no bid")),
    }
}
