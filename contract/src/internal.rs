use crate::*;

uint::construct_uint! {
    pub struct U256(4);
}

#[near_bindgen]
impl NearTips {
    pub(crate) fn get_deposit_for_account_id(&self, account_id: &AccountId, token_id: &Option<TokenAccountId>) -> Balance {
        self.deposits.get(
            &TokenByNearAccount {
                account_id: account_id.to_string(),
                token_account_id: NearTips::unwrap_token_id(token_id),
            }).unwrap_or(0)
    }

    pub(crate) fn get_deposit_for_account_id_and_token_id(&self, account_id: &AccountId, token_id: &TokenAccountId) -> Balance {
        self.deposits.get(
            &TokenByNearAccount {
                account_id: account_id.to_string(),
                token_account_id: token_id.to_string(),
            }).unwrap_or(0)
    }

    pub(crate) fn decrease_deposit(&mut self,
                                   account_id: AccountId,
                                   token_account_id: TokenAccountId,
                                   amount: Balance) {
        let key = TokenByNearAccount {
            account_id,
            token_account_id,
        };

        let sender_deposit: Balance = self.deposits.get(&key).unwrap_or(0);

        assert!(amount <= sender_deposit, "Not enough tokens to tip (Deposit: {}. Requested: {})", sender_deposit, amount);

        self.deposits.insert(&key, &(sender_deposit - amount));
    }

    pub(crate) fn increase_deposit(&mut self,
                                   account_id: AccountId,
                                   token_account_id: TokenAccountId,
                                   amount: Balance) {
        let key = TokenByNearAccount {
            account_id,
            token_account_id,
        };

        let sender_deposit: Balance = self.deposits.get(&key).unwrap_or(0);

        self.deposits.insert(&key, &(sender_deposit + amount));
    }

    pub(crate) fn set_deposit_to_zero(&mut self,
                                      account_id: AccountId,
                                      token_account_id: TokenAccountId) {
        let key = TokenByNearAccount {
            account_id,
            token_account_id,
        };

        self.deposits.insert(&key, &0);
    }

    pub(crate) fn increase_balance(&mut self,
                                   telegram_account: TelegramAccountId,
                                   token_account_id: TokenAccountId,
                                   amount: Balance) {
        let key = TokenByTelegramAccount {
            telegram_account,
            token_account_id,
        };
        let balance = self.telegram_tips.get(&key).unwrap_or(0);

        self.telegram_tips.insert(&key, &(balance + amount));
    }

    pub(crate) fn set_balance_to_zero(&mut self,
                                      telegram_account: TelegramAccountId,
                                      token_account_id: TokenAccountId) {
        let key = TokenByTelegramAccount {
            telegram_account,
            token_account_id,
        };
        self.telegram_tips.insert(&key, &0);
    }


    pub(crate) fn get_contact_owner(&self, contact: Contact, contract_address: AccountId) -> Promise {
        auth::get_account_for_contact(
            contact,
            &contract_address,
            NO_DEPOSIT,
            BASE_GAS)
    }


    pub(crate) fn assert_tip_available(&self) {
        assert!(self.tip_available, "Tips paused");
    }

    pub(crate) fn assert_withdraw_available(&self) {
        assert!(self.withdraw_available, "Withdrawals paused");
    }


    pub(crate) fn are_contacts_equal(contact1: Contact, contact2: Contact) -> bool {
        if contact1.category == ContactCategories::Telegram && contact2.category == ContactCategories::Telegram {
            contact1.account_id == contact2.account_id
        } else {
            contact1.category == contact2.category && contact1.value == contact2.value
        }
    }

    pub(crate) fn assert_check_whitelisted_token(&self, token_id: &Option<TokenAccountId>) {
        if let Some(token_id) = token_id {
            assert!(self.whitelisted_tokens.contains(&token_id), "Token wasn't whitelisted");
        }
    }
}