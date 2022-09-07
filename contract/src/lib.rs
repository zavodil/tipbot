use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, Balance, Gas, ext_contract, PromiseOrValue, PanicOnDefault, BorshStorageKey,
               log, is_promise_success, Promise, PromiseError, require};
use near_sdk::json_types::U128;
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap};

mod config;
mod tip;
mod deposit;
mod withdraw;
mod claim;
mod whitelisted_token;
mod treasury;
mod ft;
mod legacy;
mod service_account;
mod unclaimed_tips;
mod wrap;
mod views;
mod events;
mod migration;

pub use crate::config::*;
pub use crate::ft::*;
pub use crate::service_account::*;
pub use crate::tip::*;
pub use crate::deposit::*;
pub use crate::withdraw::*;
pub use crate::claim::*;
pub use crate::whitelisted_token::*;
pub use crate::treasury::*;
pub use crate::unclaimed_tips::*;
pub use crate::legacy::*;
pub use crate::wrap::*;
pub use crate::views::*;
pub use crate::migration::*;

pub type TokenAccountId = Option<AccountId>;
pub type ServiceAccountId = u32;
pub type WrappedBalance = U128;

/// All services has common `deposits` on behalf of NEAR account
/// Tips for unknown accounts stores in `unclaimed_tips`
/// How to withdraw `unclaimed_tips`:
/// 1) <Option> convert tokens to NEAR
/// 2) create NEAR account and transfer 0.1 NEAR for gas, the rest transfer to `deposits`

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct NearTips {
    // Deposits/balances for users with NEAR accounts
    deposits: LookupMap<TokenByNearAccount, Balance>,
    // Balances for user without NEAR accounts, tip receivers before the onboarding
    unclaimed_tips: LookupMap<TokenByServiceAccount, Balance>,

    // Social and etc accounts
    service_accounts: LookupMap<ServiceAccount, AccountId>,
    // Service Accounts grouped by NEAR account
    service_accounts_by_near_account: LookupMap<AccountId, Vec<ServiceAccount>>,

    // Tokens to tip. Only tokens with pairs on AMMs (REF Finance, etc) to TipToken
    whitelisted_tokens: UnorderedMap<TokenAccountId, VWhitelistedToken>,

    // part of every tip goes to treasury and may be converted to Tiptoken
    // global object with the sum of unclaimed treasury
    treasury: LookupMap<TokenAccountId, Balance>,
    // sums of claimed tokens
    treasury_claimed: LookupMap<TokenAccountId, Balance>,
    // part of treasure belongs to user and may be claimed by user using market price
    treasury_by_account: LookupMap<TokenByNearAccount, Balance>,
    // tipbot earnings
    service_fees: LookupMap<TokenAccountId, Balance>,

    config: LazyOption<Config>,
    version: u16,

    deposits_v2: LookupMap<TokenByNearAccountV2, Balance>,
    telegram_tips_v2: LookupMap<TokenByTelegramAccount, Balance>,

}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct TokenByNearAccount {
    pub account_id: AccountId,
    pub token_id: TokenAccountId,
}

#[ext_contract(ext_swap)]
pub trait ExtSwap {
    fn ft_transfer_call(&mut self, receiver_id: AccountId, amount: WrappedBalance, msg: String) -> Balance;
}


/// Helper structure to for keys of the persistent collections.
#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
    // TODO remove if no migration
    Tips,
    TipsLookupMap,
    ChatPointsLookupMap,
    TelegramDepositsLookupMap,
    TelegramTipsLookupMap,
    TelegramDeposits,
    TelegramUsersInChats,
    WhitelistedTokensLookupSet,

    Deposits,
    UnclaimedTips,
    ServiceAccounts,
    ServiceAccountsByNearAccount,
    WhitelistedTokens,
    Treasury,
    TreasuryClaimed,
    TreasuryByAccount,
    ServiceFees,
    Config
}

#[near_bindgen]
impl NearTips {
    #[init]
    pub fn new(config: Config, version: u16) -> Self {
        Self {
            deposits: LookupMap::new(StorageKey::Deposits),
            unclaimed_tips: LookupMap::new(StorageKey::UnclaimedTips),
            service_accounts: LookupMap::new(StorageKey::ServiceAccounts),
            service_accounts_by_near_account: LookupMap::new(StorageKey::ServiceAccountsByNearAccount),
            whitelisted_tokens: UnorderedMap::new(StorageKey::WhitelistedTokens),
            treasury: LookupMap::new(StorageKey::Treasury),
            treasury_claimed: LookupMap::new(StorageKey::TreasuryClaimed),
            treasury_by_account: LookupMap::new(StorageKey::TreasuryByAccount),
            service_fees: LookupMap::new(StorageKey::ServiceFees),
            config: LazyOption::new(StorageKey::Config, Some(&config)),
            version,

            deposits_v2: LookupMap::new(StorageKey::TelegramDepositsLookupMap),
            telegram_tips_v2: LookupMap::new(StorageKey::TelegramTipsLookupMap),
        }
    }

    pub fn get_version(&self) -> u16 { self.version }
}