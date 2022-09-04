use near_sdk::Promise;
use crate::*;

#[near_bindgen]
impl NearTips {

    // withdraw from deposit
    pub fn withdraw(&mut self, token_id: TokenAccountId, amount: Option<WrappedBalance>) -> Promise {
        let account_id = env::predecessor_account_id();
        self.internal_withdraw(token_id, account_id, amount)
    }


    pub fn withdraw_from_service_account(&mut self, service_account: ServiceAccount, token_id: TokenAccountId) -> Promise {
        let account_id = env::predecessor_account_id();
        self.internal_withdraw_from_service_account(service_account, token_id.clone(), account_id)
    }


    // function for user who got tips BEFORE he logged in to TipBot
    pub fn transfer_unclaimed_tips_to_deposit(&mut self, service_account: ServiceAccount, token_id: TokenAccountId, receiver_account_id: AccountId) {
        self.assert_operator();
        self.assert_withdraw_available();

        let mut balance = self.internal_get_unclaimed_tips_balance(service_account.clone(), token_id.clone());

        let withdraw_commission = self.get_withdraw_commission(&token_id);
        assert!(balance > withdraw_commission, "ERR_NOT_ENOUGH_TOKENS_TO_PAY_TRANSFER_COMMISSION");

        /* BOT IS PAYING FOR THIS WITHDRAW AND KEEPS COMMISSION TO COVER TX FEE */
        balance = balance - withdraw_commission;

        events::emit::withdraw_from_service_account(&receiver_account_id, &service_account, balance, &token_id);

        self.increase_deposit(receiver_account_id, token_id.clone(), balance);
        self.set_unclaimed_tips_balance(service_account, token_id, 0);
    }
}

impl NearTips {
    pub(crate) fn internal_withdraw(&mut self, token_id: TokenAccountId, account_id: AccountId, amount: Option<WrappedBalance>) -> Promise {
        self.assert_withdraw_available();
        self.assert_check_whitelisted_token(&token_id);

        let balance: Balance = self.internal_get_deposit(account_id.clone(), token_id.clone());

        assert!(balance > 0, "ERR_ZERO_BALANCE");

        let amount: Balance =
            if let Some(amount) = amount {
                assert!(amount.0 < balance, "ERR_BALANCE_IS_TOO_LOW");
                amount.0
            } else {
                balance
            };

        self.set_deposit(account_id.clone(), token_id.clone(), balance - amount);

        events::emit::withdraw(&account_id, amount, &token_id);

        if let Some(token_account_id) = token_id {
            self.internal_withdraw_ft(account_id, token_account_id, amount)
        } else {
            Promise::new(account_id).transfer(amount)
        }
    }


    pub(crate) fn internal_withdraw_from_service_account(&mut self, service_account: ServiceAccount, token_id: TokenAccountId, account_id: AccountId) -> Promise {
        self.transfer_unclaimed_tips_to_deposit(service_account, token_id.clone(), account_id.clone());
        self.internal_withdraw(token_id, account_id, None)
    }
}
