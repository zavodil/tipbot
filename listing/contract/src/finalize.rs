use crate::*;

const GAS_FOR_FT_TRANSFER: Gas = Gas(Gas::ONE_TERA.0 * 10);


#[near_bindgen]
impl ListingAuction {
    pub fn finalize(&mut self, auction_id: AuctionId) -> AccountId {
        let mut auction: Auction = self.unwrap_auction(&auction_id);
        self.assert_auction_end_date_already_passed(&auction);

        require!(auction.winner_id.is_none(), "ERR_ALREADY_FINALIZED");
        require!(!auction.tokens.is_empty(), "ERR_NO_TOKENS");

        let keys = auction.total_deposits.keys_as_vector();
        let values = auction.total_deposits.values_as_vector();
        let mut max_balance = values.get(0).unwrap();
        let mut winner_account_id: AccountId = keys.get(0).unwrap();
        for index in 1..keys.len() {
            let value: Balance = values.get(index).unwrap();
            if value > max_balance {
                max_balance = value;
                winner_account_id = keys.get(index).unwrap();
            }
        }

        self.existing_tokens.insert(&winner_account_id);
        self.active_auctions.remove(&auction_id);

        auction.winner_id = Some(winner_account_id.clone());
        self.auctions.insert(&auction_id, &VAuction::Current(auction));

        winner_account_id
    }

    pub fn withdraw_rewards(&mut self, auction_id: AuctionId, token_id: TokenId) -> Promise {
        let mut auction: Auction = self.unwrap_auction(&auction_id);
        let account_id = env::predecessor_account_id();

        if let Some(winner_id) = auction.winner_id.clone() {

            require!(winner_id != token_id, "ERR_WINNING_TOKENS_REWARDS_ARE_BEING_DISTRIBUTED");

            let key = get_auction_deposit(account_id.clone(), token_id.clone());
            let balance_to_withdraw = auction.rewards.get(&key).unwrap_or_default();

            require!(balance_to_withdraw > 0, "ERR_NOTHING_TO_WITHDRAW");

            auction.rewards.insert(&key, &0u128);
            self.auctions.insert(&auction_id, &VAuction::Current(auction));

            let amount = WrappedBalance::from(balance_to_withdraw);

            ext_token_receiver::ext(token_id.clone())
                .with_static_gas(GAS_FOR_FT_TRANSFER)
                .with_attached_deposit(1)
                .ft_transfer(
                    account_id.clone(),
                    amount,
                    Some(format!("Rewards withdrawn for {} listing: {}", token_id, balance_to_withdraw))
                )
                .then(
                    ext_self::ext(env::current_account_id())
                        .on_withdraw_rewards_ft_transfer(
                            auction_id,
                            token_id,
                            amount,
                            account_id
                        )
                )
        }
        else {
            env::panic_str("ERR_NO_WINNER_YET")

        }
    }

    pub fn withdraw_bids(&mut self, auction_id: AuctionId, token_id: TokenId) -> Promise {
        let mut auction: Auction = self.unwrap_auction(&auction_id);
        let account_id = env::predecessor_account_id();

        if let Some(winner_id) = auction.winner_id.clone() {

            let key = get_auction_deposit(account_id.clone(), token_id.clone());
            let balance_to_withdraw = auction.deposits.get(&key).unwrap_or_default();

            require!(balance_to_withdraw > 0, "ERR_NOTHING_TO_CLAIM");

            /* WINNING TOKEN. CLAIM AVAILABLE AFTER THE COOL DOWN */
            if winner_id == token_id {
                self.assert_auction_unlock_date_for_winner_already_passed(&auction);

                require!(auction.reward_receivers.contains(&account_id), "ERR_CLAIM_REWARDS_FIRST");
            }

            auction.deposits.insert(&key, &0u128);
            self.auctions.insert(&auction_id, &VAuction::Current(auction));

            let amount = WrappedBalance::from(balance_to_withdraw);

            ext_token_receiver::ext(self.tiptoken_account_id.clone())
                .with_static_gas(GAS_FOR_FT_TRANSFER)
                .with_attached_deposit(1)
                .ft_transfer(
                    account_id.clone(),
                    amount,
                    Some(format!("Tiptokens withdrawn for {} listing: {}", token_id, balance_to_withdraw))
                )
                .then(
                    ext_self::ext(env::current_account_id())
                        .on_withdraw_bids_ft_transfer(
                            auction_id,
                            token_id,
                            amount,
                            account_id
                        )
                )
        }
            else {
                env::panic_str("ERR_NO_WINNER_YET")

            }
    }

    // users with bids for winner may claim a share of reward pool
    pub fn claim_reward(&mut self, auction_id: AuctionId) -> Promise {
        let mut auction: Auction = self.unwrap_auction(&auction_id);
        let account_id = env::predecessor_account_id();
        require!(!auction.reward_receivers.contains(&account_id), "ERR_REWARD_ALREADY_CLAIMED");

       if let Some(token_id) = auction.winner_id.clone() {
            let key = get_auction_deposit(account_id.clone(), token_id.clone());
            let balance_to_claim = auction.deposits.get(&key).expect("ERR_NOTHING_TO_CLAIM");
            let total_deposited = auction.total_deposits.get(&token_id).unwrap_or_default();

            let total_reward = auction.total_rewards.get(&token_id).unwrap_or_default();
            let winner_rewards = u128_ratio(total_reward, balance_to_claim, total_deposited);

            auction.reward_receivers.insert(&account_id);
            self.auctions.insert(&auction_id, &VAuction::Current(auction));

            ext_token_receiver::ext(token_id.clone())
                .with_static_gas(GAS_FOR_FT_TRANSFER)
                .with_attached_deposit(1)
                .ft_transfer(
                    account_id.clone(),
                    U128::from(winner_rewards),
                    Some(format!("Rewards for {} listing: {}", token_id, winner_rewards))
                )
                .then(
                    ext_self::ext(env::current_account_id())
                        .on_claim_rewards_ft_transfer(
                            auction_id,
                            account_id
                        )
                )
        }
        else {
            env::panic_str("ERR_NO_WINNER_YET")
        }

    }
}

pub(crate) fn u128_ratio(a: u128, num: u128, denom: u128) -> Balance {
    (U256::from(a) * U256::from(num) / U256::from(denom)).as_u128()
}

uint::construct_uint!(
    pub struct U256(4);
);