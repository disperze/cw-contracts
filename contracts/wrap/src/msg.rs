use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, Uint128};
use cw20::Expiration;

#[cw_serde]
pub struct InstantiateMsg {
    /// native coin to wrap
    pub native_coin: String,

    // CW20
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Deposit token and get wrapped cw20 token
    Deposit {},
    /// Withdraw your wrapped token previous allowance
    Withdraw { amount: Uint128 },

    // Implements CW20
    /// CW20. Transfer is a base message to move tokens to another account without triggering actions
    Transfer { recipient: String, amount: Uint128 },
    /// CW20. Burn is a base message to destroy tokens forever
    Burn { amount: Uint128 },
    /// CW20. Send is a base message to transfer tokens to a contract and trigger an action
    /// on the receiving contract.
    Send {
        contract: String,
        amount: Uint128,
        msg: Binary,
    },
    /// CW20 "approval" extension. Allows spender to access an additional amount tokens
    /// from the owner's (env.sender) account. If expires is Some(), overwrites current allowance
    /// expiration with this one.
    IncreaseAllowance {
        spender: String,
        amount: Uint128,
        expires: Option<Expiration>,
    },
    /// CW20 "approval" extension. Lowers the spender's access of tokens
    /// from the owner's (env.sender) account by amount. If expires is Some(), overwrites current
    /// allowance expiration with this one.
    DecreaseAllowance {
        spender: String,
        amount: Uint128,
        expires: Option<Expiration>,
    },
    /// CW20 "approval" extension. Transfers amount tokens from owner -> recipient
    /// if `env.sender` has sufficient pre-approval.
    TransferFrom {
        owner: String,
        recipient: String,
        amount: Uint128,
    },
    /// CW20 "approval" extension. Sends amount tokens from owner -> contract
    /// if `env.sender` has sufficient pre-approval.
    SendFrom {
        owner: String,
        contract: String,
        amount: Uint128,
        msg: Binary,
    },
    /// CW20 "approval" extension. Destroys tokens forever
    BurnFrom { owner: String, amount: Uint128 },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // Implements CW20
    /// CW20. Returns the current balance of the given address, 0 if unset.
    #[returns(cw20::BalanceResponse)]
    Balance { address: String },
    /// CW20. Returns metadata on the contract - name, decimals, supply, etc.
    #[returns(cw20::TokenInfoResponse)]
    TokenInfo {},
    /// CW20. Only with "mintable" extension.
    /// Returns who can mint and how much.
    #[returns(cw20::MinterResponse)]
    Minter {},
    /// CW20 "allowance" extension.
    /// Returns how much spender can use from owner account, 0 if unset.
    #[returns(cw20::AllowanceResponse)]
    Allowance { owner: String, spender: String },
    /// CW20. Only with "enumerable" extension (and "allowances")
    /// Returns all allowances this owner has approved. Supports pagination.
    #[returns(cw20::AllAllowancesResponse)]
    AllAllowances {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// CW20. Only with "enumerable" extension
    /// Returns all accounts that have balances. Supports pagination.
    #[returns(cw20::AllAccountsResponse)]
    AllAccounts {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}
