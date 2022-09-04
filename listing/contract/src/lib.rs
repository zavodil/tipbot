use near_sdk::{env, near_bindgen, AccountId, BorshStorageKey, PanicOnDefault,
               borsh::{self, BorshDeserialize, BorshSerialize}, collections::{UnorderedMap},
               serde::{Deserialize, Serialize}, Timestamp, Balance, Promise, PromiseOrValue, serde_json, Gas, ext_contract, log, require};
use near_sdk::collections::UnorderedSet;
use near_sdk::json_types::U128;

mod utils;
mod auction;
mod ft;
mod vote;
mod finalize;

use crate::utils::*;
pub use crate::auction::*;
pub use crate::ft::*;
pub use crate::vote::*;
pub use crate::finalize::*;

type AuctionId = u64;
pub type WrappedBalance = U128;
pub type TokenId = AccountId;

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    ListingAuctions,
    ExistingTokens,

    AuctionTokens { auction_id: AuctionId },
    AuctionDeposits { auction_id: AuctionId },
    AuctionRewards { auction_id: AuctionId },
    AuctionTotalDeposits { auction_id: AuctionId },
    AuctionTotalRewards { auction_id: AuctionId },
    AuctionRewardReceivers { auction_id: AuctionId },
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct ListingAuction {
    owner_id: AccountId,
    tiptoken_account_id: AccountId,
    listing_payment: Balance,

    auctions: UnorderedMap<AuctionId, VAuction>,
    existing_tokens: UnorderedSet<AccountId>,

    next_auction_id: AuctionId,
}

#[near_bindgen]
impl ListingAuction {
    #[init]
    pub fn new(owner_id: AccountId, tiptoken_account_id: AccountId, listing_payment: WrappedBalance) -> Self {
        Self {
            owner_id,
            tiptoken_account_id,
            listing_payment: listing_payment.0,

            auctions: UnorderedMap::new(StorageKey::ListingAuctions),
            existing_tokens: UnorderedSet::new(StorageKey::ExistingTokens),

            next_auction_id: 1,
		}
    }

    pub fn get_auctions(&self, from_index: Option<u64>, limit: Option<u64>) -> Vec<(u64, AuctionOutput)> {
        unordered_map_pagination(&self.auctions, from_index, limit)
    }

    pub fn get_existing_tokens(&self) -> Vec<TokenId> {
        self.existing_tokens.to_vec()
    }

    pub fn get_timestamp(&self) -> Timestamp {
        env::block_timestamp()
    }
}
