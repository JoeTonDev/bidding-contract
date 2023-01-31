use cosmwasm_std::{Addr, Coin, StdResult, Uint128};
use cw_multi_test::{App, ContractWrapper, Executor};

use crate::contract::{execute, instantiate, query};
use crate::error::ContractError;
use crate::msg::{BidResponse, ExecuteMsg, InstantiateMsg, QueryMsg};

pub struct BiddingContract(Addr);

impl BiddingContract {
    pub fn addr(&self) -> &Addr {
        &self.0
    }

    pub fn store_code(app: &mut App) -> u64 {
        let contract = ContractWrapper::new(execute, instantiate, query);
        app.store_code(Box::new(contract))
    }

    #[track_caller]
    pub fn instantiate<'a>(
        app: &mut App,
        code_id: u64,
        sender: &Addr,
        label: &str,
        owner: impl Into<Option<&'a Addr>>,
        denom: &str,
        commission: impl Into<Option<u128>>,
    ) -> StdResult<Self> {
        let owner = owner.into();
        let commission = commission.into();

        app.instantiate_contract(
            code_id,
            sender.clone(),
            &InstantiateMsg {
                denom: denom.to_string(),
                owner: owner.map(Addr::to_string),
                commission,
            },
            &[],
            label,
            None,
        )
        .map(BiddingContract)
        .map_err(|err| err.downcast().unwrap())
    }

    #[track_caller]
    pub fn bid(&self, app: &mut App, sender: &Addr, funds: &[Coin]) -> Result<(), ContractError> {
        app.execute_contract(sender.clone(), self.0.clone(), &ExecuteMsg::Bid {}, funds)
            .map_err(|err| err.downcast().unwrap())
            .map(|_| ())
    }

    #[track_caller]
    pub fn close(&self, app: &mut App, sender: &Addr) -> Result<(), ContractError> {
        app.execute_contract(sender.clone(), self.0.clone(), &ExecuteMsg::Close {}, &[])
            .map_err(|err| err.downcast().unwrap())
            .map(|_| ())
    }

    #[track_caller]
    pub fn retract<'a>(
        &self,
        app: &mut App,
        sender: &Addr,
        receiver: impl Into<Option<&'a Addr>>,
    ) -> Result<(), ContractError> {
        let receiver = receiver.into();

        app.execute_contract(
            sender.clone(),
            self.0.clone(),
            &ExecuteMsg::Retract {
                receiver: receiver.map(Addr::to_string),
            },
            &[],
        )
        .map_err(|err| err.downcast().unwrap())
        .map(|_| ())
    }

    pub fn query_total_bids(&self, app: &App, address: &Addr) -> StdResult<Uint128> {
        app.wrap().query_wasm_smart(
            self.0.clone(),
            &QueryMsg::TotalBids {
                address: address.to_string(),
            },
        )
    }

    pub fn query_highest_bid(&self, app: &App) -> StdResult<BidResponse> {
        app.wrap()
            .query_wasm_smart(self.0.clone(), &QueryMsg::HighestBid {})
    }

    pub fn query_bidding_completed(&self, app: &App) -> StdResult<bool> {
        app.wrap()
            .query_wasm_smart(self.0.clone(), &QueryMsg::BiddingCompleted {})
    }

    pub fn query_winning_bid(&self, app: &App) -> StdResult<BidResponse> {
        app.wrap()
            .query_wasm_smart(self.0.clone(), &QueryMsg::WinningBid {})
    }
}

impl From<BiddingContract> for Addr {
    fn from(contract: BiddingContract) -> Self {
        contract.0
    }
}
