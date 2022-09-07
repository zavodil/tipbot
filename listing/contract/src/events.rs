use crate::*;

pub mod emit {
    use super::*;
    use near_sdk::serde_json::json;

    fn log_event<T: Serialize>(event: &str, data: T) {
        let event = json!({
            "standard": "auction_listing",
            "version": "1.0.0",
            "event": event,
            "data": [data]
        });

        log!("EVENT_JSON:{}", event.to_string());
    }

    pub fn add_auction(auction_id: &AuctionId, start_date: &Timestamp,
                       end_date: &Timestamp, unlock_date_for_winner: &Timestamp) {
        log_event(
            "add_auction",
            json!({
                "auction_id": auction_id,
                "start_date": start_date,
                "end_date": end_date,
                "unlock_date_for_winner": unlock_date_for_winner
            }),
        );
    }

    pub fn vote(auction_id: &AuctionId, account_id: &AccountId, listing_token_id: &TokenId, amount: Balance) {
        log_event(
            "vote",
            json!({
                "auction_id": auction_id,
                "account_id": account_id,
                "listing_token_id": listing_token_id,
                "amount": amount
            }),
        );
    }

    pub fn add_reward(auction_id: &AuctionId, account_id: &AccountId, token_id: &TokenId, amount: Balance) {
        log_event(
            "vote",
            json!({
                "auction_id": auction_id,
                "account_id": account_id,
                "token_id": token_id,
                "amount": amount
            }),
        );
    }

    pub fn add_token_to_auction(auction_id: &AuctionId, token_id: &TokenId) {
        log_event(
            "add_token_to_auction",
            json!({
                "auction_id": auction_id,
                "token_id": token_id
            }),
        );
    }

    pub fn finalize(auction_id: &AuctionId, winner_token_id: &TokenId) {
        log_event(
            "finalize",
            json!({
                "auction_id": auction_id,
                "winner_token_id": winner_token_id
            }),
        );
    }

    pub fn withdraw_rewards(auction_id: &AuctionId, account_id: &AccountId, token_id: &TokenId, amount: Balance) {
        log_event(
            "withdraw_rewards",
            json!({
                "auction_id": auction_id,
                "account_id": account_id,
                "token_id": token_id,
                "amount": U128::from(amount)
            }),
        );
    }

    pub fn withdraw_bids(auction_id: &AuctionId, account_id: &AccountId, token_id: &TokenId, amount: Balance) {
        log_event(
            "withdraw_bids",
            json!({
                "auction_id": auction_id,
                "account_id": account_id,
                "token_id": token_id,
                "amount": U128::from(amount)
            }),
        );
    }

    pub fn claim_reward(auction_id: &AuctionId, account_id: &AccountId, token_id: &TokenId, amount: Balance) {
        log_event(
            "claim_reward",
            json!({
                "auction_id": auction_id,
                "account_id": account_id,
                "token_id": token_id,
                "amount": U128::from(amount)
            }),
        );
    }

    pub fn claim_reward_failed(auction_id: &AuctionId, account_id: &AccountId) {
        log_event(
            "claim_reward_failed",
            json!({
                "auction_id": auction_id,
                "account_id": account_id
            }),
        );
    }

    pub fn withdraw_bids_failed(auction_id: &AuctionId, account_id: &AccountId, token_id: &TokenId, amount: Balance) {
        log_event(
            "withdraw_bids_failed",
            json!({
                "auction_id": auction_id,
                "account_id": account_id,
                "token_id": token_id,
                "amount": U128::from(amount)
            }),
        );
    }

    pub fn withdraw_rewards_failed(auction_id: &AuctionId, account_id: &AccountId, token_id: &TokenId, amount: Balance) {
        log_event(
            "withdraw_rewards_failed",
            json!({
                "auction_id": auction_id,
                "account_id": account_id,
                "token_id": token_id,
                "amount": U128::from(amount)
            }),
        );
    }
}