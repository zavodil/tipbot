use near_sdk::is_promise_success;
use crate::*;

const TOKENS_FOR_STORAGE_DEPOSIT: Balance = 1_250_000_000_000_000_000_000;
const GAS_FOR_STORAGE_DEPOSIT_PAY: Gas = Gas(Gas::ONE_TERA.0 * 5);

#[ext_contract(ext_self)]
pub trait ExtMultisender {
    fn on_storage_balance_of(&mut self, #[callback] storage_balance_of: Option<StorageBalance>, auction_id: AuctionId, token_id: AccountId);
    fn on_claim_rewards_ft_transfer(&mut self, auction_id: AuctionId, account_id: AccountId);
    fn on_withdraw_bids_ft_transfer(&mut self, auction_id: AuctionId, token_id: AccountId, amount: WrappedBalance, account_id: AccountId);
    fn on_withdraw_rewards_ft_transfer(&mut self, auction_id: AuctionId, token_id: AccountId, amount: WrappedBalance, account_id: AccountId);
    fn add_token_to_auction(&mut self, auction_id: AuctionId, token_id: AccountId) -> Promise;
}

#[ext_contract(ext_token_receiver)]
pub trait ExtToken {
    fn storage_balance_of(&self, account_id: AccountId) -> Promise;
    fn ft_transfer(&self, receiver_id: AccountId, amount: WrappedBalance, memo: Option<String>);
    fn storage_deposit(&self, account_id: Option<AccountId>, registration_only: Option<bool>) -> StorageBalance;
}

#[derive(Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, Serialize))]
#[serde(crate = "near_sdk::serde")]
pub enum TokenReceiverMsg {
    Vote { auction_id: AuctionId, listing_token_id: TokenId },
    AddReward { auction_id: AuctionId },
}

#[near_bindgen]
impl ListingAuction {
    pub fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let attached_token_id = env::predecessor_account_id();

        let token_receiver_msg: TokenReceiverMsg =
            serde_json::from_str(&msg).expect("Can't parse TokenReceiverMsg");
        match token_receiver_msg {
            TokenReceiverMsg::Vote { auction_id, listing_token_id } => {
                self.vote(auction_id, sender_id, listing_token_id, attached_token_id, amount.0);
            }
            TokenReceiverMsg::AddReward { auction_id } => {
                self.add_reward(auction_id, sender_id, attached_token_id, amount.0);
            }
        }

        PromiseOrValue::Value(U128(0))
    }

    #[private]
    #[payable]
    pub fn on_storage_balance_of(&mut self, #[callback] storage_balance_of: Option<StorageBalance>, auction_id: AuctionId, token_id: AccountId) {
        if let Some (storage_balance_of) = storage_balance_of {
            log!("Storage balance for {} is {}", token_id, storage_balance_of.total.0);
            self.internal_add_token_to_auction(auction_id, token_id);
        } else {
            let current_account_id = env::current_account_id();
            ext_token_receiver::ext(token_id.clone())
                .with_static_gas(GAS_FOR_STORAGE_DEPOSIT_PAY)
                .with_attached_deposit(TOKENS_FOR_STORAGE_DEPOSIT)
                .storage_deposit(
                    Some(current_account_id.clone()),
                    Some(true),
                )
                .then(
                    ext_self::ext(current_account_id)
                        .with_attached_deposit(env::attached_deposit() - TOKENS_FOR_STORAGE_DEPOSIT)
                        .add_token_to_auction(
                            auction_id,
                            token_id,
                        )
                );
        }
    }

    #[private]
    pub fn on_claim_rewards_ft_transfer(&mut self, auction_id: AuctionId, account_id: AccountId) {
        if !is_promise_success() {
            log!("FT transfer of rewards failed");
            let mut auction: Auction = self.unwrap_auction(&auction_id);
            auction.reward_receivers.remove(&account_id);

            self.auctions.insert(&auction_id, &VAuction::Current(auction));
        }
    }

    #[private]
    pub fn on_withdraw_bids_ft_transfer(&mut self, auction_id: AuctionId, token_id: AccountId, amount: WrappedBalance, account_id: AccountId) {
        // TODO what if malicious FT will fail all fr_transfer?
        if !is_promise_success() {
            log!("FT transfer of bids failed. Restore the state");

            let mut auction: Auction = self.unwrap_auction(&auction_id);
            let key = get_auction_deposit(account_id, token_id);
            auction.deposits.insert(&key, &amount.0);

            self.auctions.insert(&auction_id, &VAuction::Current(auction));
        }
    }

    #[private]
    pub fn on_withdraw_rewards_ft_transfer(&mut self, auction_id: AuctionId, token_id: AccountId, amount: WrappedBalance, account_id: AccountId) {
        // TODO what if malicious FT will fail all fr_transfer?
        if !is_promise_success() {
            log!("FT transfer of rewards failed. Restore the state");

            let mut auction: Auction = self.unwrap_auction(&auction_id);
            let key = get_auction_deposit(account_id, token_id);
            auction.rewards.insert(&key, &amount.0);

            self.auctions.insert(&auction_id, &VAuction::Current(auction));
        }
    }
}

