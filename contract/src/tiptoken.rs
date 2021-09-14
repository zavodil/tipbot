use crate::*;

// 100 M TipTokens
const MAX_TIPTOKEN_DISTRIBUTION: Balance = 100_000_000_000_000_000_000_000_000_000_000;

#[near_bindgen]
impl NearTips {
    pub(crate) fn asset_chat_owner(&self, chat_id: TelegramChatId) {
        let settings = self.get_chat_settings(chat_id);
        assert!(settings.is_some(), "Unknown chat");

        let account_id = env::predecessor_account_id();
        let admin_account_id: AccountId = settings.unwrap().admin_account_id;
        assert_eq!(admin_account_id, account_id, "Current user is not a chat admin");
    }

    pub(crate) fn distribute_tiptokens(&mut self, chat_id: TelegramChatId, chat_admin_account_id: AccountId, token_id: TokenAccountId, treasure_fee: Balance, sender_account_id: AccountId) {
        let tokens_for_chat: Balance = treasure_fee * 4 / 10;
        let tokens_for_sender: Balance = treasure_fee * 4 / 10;
        let tokens_for_treasury: Balance = treasure_fee - tokens_for_chat - tokens_for_sender;

        // update chat admin tiptoken balance
        let token_by_chat_admin = TokenByNearAccount {
            account_id: chat_admin_account_id.clone(),
            token_account_id: token_id.clone(),
        };
        let chat_admin_tokens: Balance = self.user_tokens_to_claim.get(&token_by_chat_admin).unwrap_or(0);
        let new_chat_admin_tokens = chat_admin_tokens + tokens_for_chat;
        self.user_tokens_to_claim.insert(&token_by_chat_admin, &new_chat_admin_tokens);

        // update sender tiptoken balance
        let token_by_sender = TokenByNearAccount {
            account_id: sender_account_id.clone(),
            token_account_id: token_id.clone(),
        };
        let user_tokens: Balance = self.user_tokens_to_claim.get(&token_by_sender).unwrap_or(0);
        let new_user_tokens = user_tokens + tokens_for_sender;
        self.user_tokens_to_claim.insert(&token_by_sender, &new_user_tokens);

        // update treasure balance
        let treasure_balance: Balance = self.get_treasure_balance_for_token(&token_id);
        self.treasure.insert(&token_id, &(treasure_balance + treasure_fee));

        env::log(format!("Reward point(s): {} for {} on behalf of chat {} (total: {}), {} for sender {} (total: {}). Treasury reward: {}",
                         tokens_for_chat, chat_admin_account_id, chat_id, new_chat_admin_tokens,
                         tokens_for_sender, sender_account_id, new_user_tokens, tokens_for_treasury).as_bytes());
    }

    // claim_chat_tokens TODO TEST
    #[payable]
    pub fn claim_tiptokens_for_chat(&mut self, chat_id: TelegramChatId, token_id: Option<TokenAccountId>) -> Promise {
        assert_one_yocto();
        self.asset_chat_owner(chat_id);
        self.claim_tiptokens_for_account_id(env::predecessor_account_id(), token_id)
    }

    #[payable] // TO REGISTER account in TipToken before to claim
    pub fn claim_tiptokens(&mut self, token_id: Option<TokenAccountId>) -> Promise {
        assert_one_yocto();
        self.claim_tiptokens_for_account_id(env::predecessor_account_id(), token_id)
    }

    pub fn convert_reward_points_to_tiptoken(&self, amount: Balance, token_id: TokenAccountId) -> Balance {
        if token_id == NEAR {
            amount
        } else {
            0
        }
    }

    pub fn convert_tiptoken_to_reward_points(&self, amount: Balance, token_id: TokenAccountId) -> Balance {
        if token_id == NEAR {
            amount
        } else {
            0
        }
    }

    pub(crate) fn claim_tiptokens_for_account_id(&mut self, account_id: AccountId, token_id: Option<TokenAccountId>) -> Promise {
        let token_id_unwrapped = NearTips::unwrap_token_id(&token_id);

        let token_by_user = TokenByNearAccount {
            account_id: account_id.clone(),
            token_account_id: token_id_unwrapped.clone(),
        };

        let user_balance: Balance = self.user_tokens_to_claim.get(&token_by_user).unwrap_or(0);
        assert!(user_balance > 0, "Nothing to claim");

        let tiptoken_amount: Balance = self.convert_reward_points_to_tiptoken(user_balance, token_id_unwrapped.clone());
        assert!(tiptoken_amount > 0, "Only claim for NEAR tips are available now");

        self.total_tiptokens += tiptoken_amount;
        assert!(self.total_tiptokens < MAX_TIPTOKEN_DISTRIBUTION, "No more Tiptokens available");

        self.user_tokens_to_claim.insert(&token_by_user, &0);

        ext_fungible_token::ft_transfer(
            account_id.clone(),
            tiptoken_amount.into(),
            Some(format!(
                "Tiptokens claimed for {} tips in {}",
                tiptoken_amount,
                token_id_unwrapped
            )),
            &self.tiptoken_account_id,
            ONE_YOCTO,
            GAS_FOR_FT_TRANSFER,
        )
            .then(ext_self::after_ft_transfer_claim_tiptokens(
                account_id,
                tiptoken_amount.into(),
                token_id_unwrapped.clone(),
                &env::current_account_id(),
                NO_DEPOSIT,
                GAS_FOR_AFTER_FT_TRANSFER,
            ))
    }

    #[private]
    pub fn after_ft_transfer_claim_tiptokens(
        &mut self,
        account_id: AccountId,
        amount_redeemed: WrappedBalance,
        token_account_id: TokenAccountId,
    ) -> bool {
        let promise_success = is_promise_success();
        if !is_promise_success() {
            let token_by_user = TokenByNearAccount {
                account_id: account_id.clone(),
                token_account_id: token_account_id.clone(),
            };

            let user_balance: Balance = self.user_tokens_to_claim.get(&token_by_user).unwrap_or(0);

            let user_reward_points: Balance = self.convert_tiptoken_to_reward_points(amount_redeemed.0, token_account_id);
            assert!(user_reward_points > 0, "Illegal redeem value");

            self.total_tiptokens -= amount_redeemed.0;

            self.user_tokens_to_claim.insert(&token_by_user, &(user_balance + user_reward_points));

            log!(
                "Tiptoken redeem for {} failed. Points to recharge: {} for {} TipTokens",
                account_id,
                user_reward_points,
                amount_redeemed.0
            );
        }
        promise_success
    }

    #[payable]
    pub fn redeem_tiptokens(&mut self, tokens_to_claim: Vec<TokenAccountId>) {
        let account_id = env::predecessor_account_id();

        let tiptoken_amount = self.get_deposit_for_account_id_and_token_id(&account_id, &self.tiptoken_account_id);

        assert!(tiptoken_amount > 0, "Nothing to redeem");

        let numerator = U256::from(tiptoken_amount);
        let denominator = U256::from(self.total_tiptokens);

        self.tiptokens_burned += tiptoken_amount;

        for token_account_id in tokens_to_claim {
            let treasure_balance: Balance = self.get_treasure_balance_for_token(&token_account_id);
            let amount = (U256::from(treasure_balance) * numerator / denominator).as_u128();

            if amount > 0 {
                let new_balance = treasure_balance.checked_sub(amount).expect("Not enough balance");

                self.set_treasure_balance_for_token(&token_account_id, &new_balance);

                self.deposit_amount_to_account(&account_id, amount, Some(token_account_id.clone()));

                log!("{} of {} redeemed", amount, token_account_id);
            }
        }
    }

    pub fn get_unclaimed_tiptokens_amount(&self) -> WrappedBalance {
        (MAX_TIPTOKEN_DISTRIBUTION - self.total_tiptokens).into()
    }

    pub fn get_total_tiptokens(&self) -> WrappedBalance {
        self.total_tiptokens.into()
    }

    pub fn get_chat_tokens(&self, chat_id: TelegramChatId, token_id: Option<TokenAccountId>) -> WrappedBalance {
        let settings = self.get_chat_settings(chat_id);
        assert!(settings.is_some(), "Unknown chat");

        self.get_user_tokens(settings.unwrap().admin_account_id, token_id)
    }

    pub fn get_user_tokens(&self, account_id: AccountId, token_id: Option<TokenAccountId>) -> WrappedBalance {
        let token_account_id = NearTips::unwrap_token_id(&token_id);
        self.user_tokens_to_claim.get(&TokenByNearAccount {
            account_id,
            token_account_id,
        }).unwrap_or(0).into()
    }

    pub fn get_chat_points(&self, chat_id: TelegramChatId) -> RewardPoint {
        self.chat_points.get(&chat_id).unwrap_or(0)
    }

    pub fn get_telegram_users_in_chats(&self, telegram_id: TelegramAccountId,
                                       chat_id: TelegramChatId) -> bool {
        self.telegram_users_in_chats.contains(&TelegramUserInChat {
            telegram_id,
            chat_id,
        })
    }

    pub fn assert_valid_treasure_fee_numerator(numerator: TreasureFeeNumerator) {
        assert!(
            numerator <= 10,
            "Treasure fee can't be greater then 10%"
        );
    }

    pub(crate) fn get_treasure_fee(value: Balance, numerator: TreasureFeeNumerator) -> Balance {
        (U256::from(numerator) * U256::from(value) / U256::from(100)).as_u128()
    }


    pub fn get_treasure_balance(&self, token_account_id: Option<TokenAccountId>) -> WrappedBalance {
        let token_id_unwrapped = NearTips::unwrap_token_id(&token_account_id);
        self.get_treasure_balance_for_token(&token_id_unwrapped).into()
    }

    pub(crate) fn get_treasure_balance_for_token(&self, token_id: &TokenAccountId) -> Balance {
        self.treasure.get(token_id).unwrap_or(0)
    }

    pub(crate) fn set_treasure_balance_for_token(&mut self, token_id: &TokenAccountId, amount: &Balance) {
        self.treasure.insert(token_id, amount);
    }


    pub fn add_chat_settings(&mut self,
                             chat_id: TelegramChatId,
                             admin_account_id: ValidAccountId,
                             treasure_fee_numerator: TreasureFeeNumerator,
                             track_chat_points: bool) {
        self.assert_master_account_id();

        self.chat_settings.insert(&chat_id, &ChatSettings {
            admin_account_id: admin_account_id.into(),
            treasure_fee_numerator,
            track_chat_points,
        });
    }

    pub fn delete_chat_settings(&mut self, chat_id: TelegramChatId) {
        self.assert_master_account_id();
        self.chat_settings.remove(&chat_id);
    }

    pub fn get_chat_settings(&self, chat_id: TelegramChatId) -> Option<ChatSettings> {
        self.chat_settings.get(&chat_id)
    }

    pub fn get_chat_numerator(&self, chat_id: TelegramChatId) -> TreasureFeeNumerator {
        let settings = self.get_chat_settings(chat_id);
        if let Some(settings_unwrapped) = settings {
            settings_unwrapped.treasure_fee_numerator
        } else {
            0
        }
    }
}