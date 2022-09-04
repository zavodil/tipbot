use crate::*;

#[near_bindgen]
impl NearTips {
    #[payable]
    pub fn deposit(&mut self, account_id: Option<AccountId>) {
        let attached_deposit: Balance = env::attached_deposit();
        let account_id: AccountId = account_id.unwrap_or(env::predecessor_account_id());
        self.deposit_to_near_account(&account_id, attached_deposit, None, true);
    }

    pub fn get_deposit(&self, account_id: AccountId, token_id: TokenAccountId) -> WrappedBalance {
        self.internal_get_deposit(account_id, token_id).into()
    }
}

impl NearTips {
    pub(crate) fn internal_get_deposit(&self, account_id: AccountId, token_id: TokenAccountId) -> Balance {
        self.deposits.get(
            &TokenByNearAccount {
                account_id,
                token_id,
            }).unwrap_or(0)
    }

    pub(crate) fn increase_deposit(&mut self,
                                   account_id: AccountId,
                                   token_id: TokenAccountId,
                                   amount: Balance) {
        let key = TokenByNearAccount {
            account_id: account_id.clone(),
            token_id: token_id.clone(),
        };

        let sender_deposit: Balance = self.deposits.get(&key).unwrap_or(0);

        self.deposits.insert(&key, &(sender_deposit + amount));

        events::emit::deposit(&account_id, amount, &token_id);
    }

    pub(crate) fn set_deposit(&mut self,
                                   account_id: AccountId,
                                   token_id: TokenAccountId,
                                   new_amount: Balance) {
        let key = TokenByNearAccount {
            account_id,
            token_id,
        };

        self.deposits.insert(&key, &new_amount);
    }

    pub(crate) fn deposit_to_near_account(&mut self, account_id: &AccountId, amount: Balance, token_id: TokenAccountId, check_deposit_amount: bool) {
        self.assert_withdraw_available();
        self.assert_check_whitelisted_token(&token_id);
        if check_deposit_amount {
            assert!(amount >= self.get_token_min_deposit(&token_id), "Deposit is too small");
        }

        self.increase_deposit(account_id.clone(), token_id.clone(), amount);
    }


}
