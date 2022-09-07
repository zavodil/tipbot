use crate::*;

const GAS_FOR_STORAGE_DEPOSIT_READ: Gas = Gas(Gas::ONE_TERA.0 * 10);

#[derive(BorshDeserialize, BorshSerialize)]
pub struct AuctionDeposit {
    pub account_id: AccountId,
    pub token_id: TokenId,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct StorageBalance {
    pub total: U128,
    pub available: U128,
}

#[near_bindgen]
impl ListingAuction {
    #[payable]
    pub fn add_token_to_auction(&mut self, auction_id: AuctionId, token_id: AccountId) -> Promise {
        let auction: Auction = self.unwrap_auction(&auction_id);
        self.assert_auction_timestamp(&auction);

        assert!(env::attached_deposit() >= self.listing_payment, "ERR_DEPOSIT_IS_TOO_SMALL");

        assert!(!auction.tokens.contains(&token_id), "ERR_TOKEN_ALREADY_ADDED_TO_AUCTION");
        assert!(!self.existing_tokens.contains(&token_id), "ERR_TOKEN_ALREADY_LISTED");

        let current_account_id = env::current_account_id();

        ext_token_receiver::ext(token_id.clone())
            .with_static_gas(GAS_FOR_STORAGE_DEPOSIT_READ)
            .storage_balance_of(
                current_account_id.clone(),
            )
            .then(
                ext_self::ext(current_account_id)
                    .with_attached_deposit(env::attached_deposit())
                    .on_storage_balance_of(
                        auction_id,
                        token_id
                    )
            )
    }
}


impl ListingAuction {
    pub(crate) fn vote(&mut self, auction_id: AuctionId, account_id: AccountId, listing_token_id: TokenId, attached_token_id: TokenId, amount: Balance) {
        self.assert_tiptoken(attached_token_id);

        let mut auction: Auction = self.unwrap_auction(&auction_id);
        self.assert_auction_timestamp(&auction);
        self.assert_auction_token(&auction, &listing_token_id);

        events::emit::vote(&auction_id, &account_id, &listing_token_id, amount);

        let key = get_auction_deposit(account_id, listing_token_id.clone());

        let previous_deposit = auction.deposits.get(&key).unwrap_or_default();
        auction.deposits.insert(&key, &(previous_deposit + amount));

        let previous_total_deposit = auction.total_deposits.get(&listing_token_id).unwrap_or_default();
        auction.total_deposits.insert(&listing_token_id, &(previous_total_deposit + amount));

        self.auctions.insert(&auction_id, &VAuction::Current(auction));
    }

    pub(crate) fn add_reward(&mut self, auction_id: AuctionId, account_id: AccountId, attached_token_id: TokenId, amount: Balance) {
        let mut auction: Auction = self.unwrap_auction(&auction_id);
        self.assert_auction_timestamp(&auction);
        assert!(auction.tokens.contains(&attached_token_id), "ERR_ILLEGAL_TOKEN");

        events::emit::add_reward(&auction_id, &account_id, &attached_token_id, amount);

        let key = get_auction_deposit(account_id, attached_token_id.clone());

        let previous_rewards = auction.rewards.get(&key).unwrap_or_default();
        auction.rewards.insert(&key, &(previous_rewards + amount));

        let previous_total_rewards = auction.total_rewards.get(&attached_token_id).unwrap_or_default();
        auction.total_rewards.insert(&attached_token_id, &(previous_total_rewards + amount));

        self.auctions.insert(&auction_id, &VAuction::Current(auction));
    }
}

pub(crate) fn get_auction_deposit(account_id: AccountId, token_id: TokenId) -> AuctionDeposit {
    AuctionDeposit {
        account_id,
        token_id,
    }
}