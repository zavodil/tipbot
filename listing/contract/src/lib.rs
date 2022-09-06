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
    ActiveAuctions,

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
    active_auctions: UnorderedSet<AuctionId>,
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
            active_auctions: UnorderedSet::new(StorageKey::ActiveAuctions),
            existing_tokens: UnorderedSet::new(StorageKey::ExistingTokens),

            next_auction_id: 1,
		}
    }

    pub fn get_active_auction_ids(&self) -> Vec<AuctionId> {
        self.active_auctions.to_vec()
    }

    pub fn get_active_auctions(&self, from_index: Option<u64>, limit: Option<u64>) -> Vec<(u64, AuctionOutput)> {
        let keys = self.active_auctions.as_vector();
        let from_index = from_index.unwrap_or(0);
        let limit = limit.unwrap_or(keys.len());
        (from_index..std::cmp::min(keys.len(), from_index + limit))
            .map(|index| {
                let key = keys.get(index).unwrap();
                (key, self.auctions.get(&key).unwrap().into())
            })
            .collect()
    }

    pub fn get_auctions(&self, from_index: Option<u64>, limit: Option<u64>) -> Vec<(u64, AuctionOutput)> {
        unordered_map_pagination(&self.auctions, from_index, limit)
    }

    pub fn get_existing_tokens(&self) -> Vec<TokenId> {
        self.existing_tokens.to_vec()
    }

    pub fn add_existing_token(&mut self, token_id: TokenId) {
        self.assert_owner();
        self.existing_tokens.insert(&token_id);
    }

    pub fn remove_existing_token(&mut self, token_id: TokenId) {
        self.assert_owner();
        self.existing_tokens.remove(&token_id);
    }

    pub fn get_timestamp(&self) -> Timestamp {
        env::block_timestamp()
    }

    pub fn get_next_auction_id(&self) -> AuctionId { self.next_auction_id }
}
