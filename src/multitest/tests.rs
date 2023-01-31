use cosmwasm_std::{coins, Addr, Uint128};
use cw_multi_test::App;

use crate::error::ContractError;
use crate::msg::BidResponse;
use crate::state::{Config, State, Status, CONFIG, STATE};

use super::contract::BiddingContract;

const ATOM: &str = "atom";

#[test]
fn bidding_with_owner() {
    let owner = Addr::unchecked("owner");
    let sender = Addr::unchecked("sender");

    let mut app = App::default();

    let code_id = BiddingContract::store_code(&mut app);
    let contract = BiddingContract::instantiate(
        &mut app,
        code_id,
        &sender,
        "Bidding contract",
        &owner,
        ATOM,
        1_000_000,
    )
    .unwrap();

    let state = STATE.query(&app.wrap(), contract.addr().clone()).unwrap();

    assert_eq!(
        state,
        State {
            current_status: Status::Open,
            highest_bid: None,
        }
    );

    let config = CONFIG.query(&app.wrap(), contract.addr().clone()).unwrap();

    assert_eq!(
        config,
        Config {
            denom: ATOM.to_string(),
            owner,
            commission: 1_000_000,
        }
    );
}

#[test]
fn bidding_with_sender() {
    let owner = Addr::unchecked("owner");

    let mut app = App::default();
    let code_id = BiddingContract::store_code(&mut app);

    let contract = BiddingContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "Bidding contract",
        None,
        ATOM,
        1_000_000,
    )
    .unwrap();

    let state = STATE.query(&app.wrap(), contract.addr().clone()).unwrap();

    assert_eq!(
        state,
        State {
            current_status: Status::Open,
            highest_bid: None,
        }
    );

    let config = CONFIG.query(&app.wrap(), contract.addr().clone()).unwrap();

    assert_eq!(
        config,
        Config {
            denom: ATOM.to_string(),
            owner,
            commission: 1_000_000,
        }
    );
}

#[test]
fn unauthorized_bid() {
    let owner = Addr::unchecked("owner");
    let sender = Addr::unchecked("sender");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &owner, coins(10_000_000, ATOM))
            .unwrap();
    });

    let code_id = BiddingContract::store_code(&mut app);
    let contract = BiddingContract::instantiate(
        &mut app,
        code_id,
        &sender,
        "Bidding contract",
        &owner,
        ATOM,
        1_000_000,
    )
    .unwrap();

    let err = contract
        .bid(&mut app, &owner, &coins(1_000_000, ATOM))
        .unwrap_err();

    assert_eq!(
        err,
        ContractError::UnauthorizedBid {
            owner: owner.to_string()
        }
    );

    assert_eq!(
        app.wrap().query_all_balances(owner).unwrap(),
        coins(10_000_000, ATOM)
    );

    assert_eq!(app.wrap().query_all_balances(contract.addr()).unwrap(), &[]);
}

#[test]
fn invalid_funds() {
    let owner = Addr::unchecked("owner");
    let sender = Addr::unchecked("sender");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender, coins(10_000_000, ATOM))
            .unwrap();
    });

    let code_id = BiddingContract::store_code(&mut app);
    let contract = BiddingContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "Bidding contract",
        None,
        ATOM,
        1_000_000,
    )
    .unwrap();

    let err = contract
        .bid(&mut app, &sender, &coins(900_000, ATOM))
        .unwrap_err();

    assert_eq!(
        err,
        ContractError::InvalidCommission {
            funds: Uint128::new(900_000),
            commission: Uint128::new(1_000_000)
        }
    );

    assert_eq!(
        app.wrap().query_all_balances(sender).unwrap(),
        coins(10_000_000, ATOM)
    );

    assert_eq!(app.wrap().query_all_balances(owner).unwrap(), &[]);

    assert_eq!(app.wrap().query_all_balances(contract.addr()).unwrap(), &[]);
}

#[test]
fn bidding() {
    let owner = Addr::unchecked("owner");
    let sender = Addr::unchecked("sender");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender, coins(15_000_000, ATOM))
            .unwrap();
    });

    let code_id = BiddingContract::store_code(&mut app);
    let contract = BiddingContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "Bidding contract",
        None,
        ATOM,
        1_000_000,
    )
    .unwrap();

    contract
        .bid(&mut app, &sender, &coins(14_000_000, ATOM))
        .unwrap();

    assert_eq!(
        app.wrap().query_all_balances(contract.addr()).unwrap(),
        coins(14_000_000, ATOM)
    );
}

#[test]
fn close_bidding() {
    let owner = Addr::unchecked("owner");
    let sender = Addr::unchecked("sender");

    let mut app = App::default();

    let code_id = BiddingContract::store_code(&mut app);
    let contract = BiddingContract::instantiate(
        &mut app,
        code_id,
        &sender,
        "Bidding contract",
        &owner,
        ATOM,
        1_000_000,
    )
    .unwrap();

    let state = STATE.query(&app.wrap(), contract.addr().clone()).unwrap();
    assert_eq!(
        state,
        State {
            current_status: Status::Open,
            highest_bid: None
        }
    );

    contract.close(&mut app, &owner).unwrap();

    let state = STATE.query(&app.wrap(), contract.addr().clone()).unwrap();
    assert_eq!(
        state,
        State {
            current_status: Status::Closed,
            highest_bid: None
        }
    );
}

#[test]
fn unauthorized_closed() {
    let owner = Addr::unchecked("owner");
    let sender = Addr::unchecked("sender");

    let mut app = App::default();

    let code_id = BiddingContract::store_code(&mut app);
    let contract = BiddingContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "Bidding contract",
        None,
        ATOM,
        1_000_000,
    )
    .unwrap();

    let state = STATE.query(&app.wrap(), contract.addr().clone()).unwrap();
    assert_eq!(
        state,
        State {
            current_status: Status::Open,
            highest_bid: None
        }
    );

    let err = contract.close(&mut app, &sender).unwrap_err();
    assert_eq!(
        err,
        ContractError::Unauthorized {
            owner: owner.to_string()
        }
    );

    let state = STATE.query(&app.wrap(), contract.addr().clone()).unwrap();
    assert_eq!(
        state,
        State {
            current_status: Status::Open,
            highest_bid: None
        }
    );
}

#[test]
fn query_total_bids() {
    let owner = Addr::unchecked("owner");
    let sender = Addr::unchecked("sender");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender, coins(10_000_000, ATOM))
            .unwrap();
    });

    let code_id = BiddingContract::store_code(&mut app);
    let contract = BiddingContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "Bidding contract",
        None,
        ATOM,
        1_000_000,
    )
    .unwrap();

    contract
        .bid(&mut app, &sender, &coins(4_000_000, ATOM))
        .unwrap();

    contract
        .bid(&mut app, &sender, &coins(4_000_000, ATOM))
        .unwrap();

    let resp = contract.query_total_bids(&app, &sender).unwrap();
    assert_eq!(resp, Uint128::new(8_000_000));
}

#[test]
fn query_highest_bid() {
    let owner = Addr::unchecked("owner");
    let sender1 = Addr::unchecked("sender1");
    let sender2 = Addr::unchecked("sender2");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender1, coins(5_000_000, ATOM))
            .unwrap();
        router
            .bank
            .init_balance(storage, &sender2, coins(6_000_000, ATOM))
            .unwrap();
    });

    let code_id = BiddingContract::store_code(&mut app);
    let contract = BiddingContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "Bidding contract",
        None,
        ATOM,
        1_000_000,
    )
    .unwrap();

    contract
        .bid(&mut app, &sender1, &coins(4_000_000, ATOM))
        .unwrap();

    contract
        .bid(&mut app, &sender2, &coins(5_000_000, ATOM))
        .unwrap();

    let resp = contract.query_highest_bid(&app).unwrap();
    assert_eq!(
        resp,
        BidResponse {
            address: sender2,
            amount: Uint128::new(5_000_000)
        }
    );
}

#[test]
fn query_winning_bid() {
    let owner = Addr::unchecked("owner");
    let sender = Addr::unchecked("sender");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender, coins(10_000_000, ATOM))
            .unwrap();
    });

    let code_id = BiddingContract::store_code(&mut app);
    let contract = BiddingContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "Bidding contract",
        None,
        ATOM,
        1_000_000,
    )
    .unwrap();

    contract
        .bid(&mut app, &sender, &coins(9_000_000, ATOM))
        .unwrap();

    contract.close(&mut app, &owner).unwrap();

    let resp = contract.query_bidding_completed(&app).unwrap();
    assert_eq!(resp, true);

    let resp = contract.query_winning_bid(&app).unwrap();
    assert_eq!(
        resp,
        BidResponse {
            address: sender,
            amount: Uint128::new(9_000_000)
        }
    );
}
