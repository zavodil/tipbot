use near_sdk::Promise;
use crate::*;

#[near_bindgen]
impl NearTips {
    #[payable]
    pub fn deposit_to_account(&mut self, account_id: AccountId) {
        self.deposit(Some(account_id));
    }

    pub fn get_balance(&self,
                       telegram_account: ServiceAccountId,
                       token_id: TokenAccountId) -> WrappedBalance {
        self.get_unclaimed_tips_balance(get_telegram_account(telegram_account), token_id)
    }

    pub fn get_balances(&self,
                        telegram_account: ServiceAccountId,
                        token_ids: Vec<TokenAccountId>,
    ) -> Vec<(TokenAccountId, WrappedBalance)> {
        token_ids
            .iter()
            .map(|token_account_id|
                (
                    token_account_id.clone(),
                    self.get_balance(telegram_account, (*token_account_id).clone()),
                ))
            .collect()
    }

    pub fn send_tip_to_telegram(&mut self,
                                telegram_account: ServiceAccountId,
                                amount: WrappedBalance,
                                token_id: TokenAccountId) {
        self.tip(None, Some(get_telegram_account(telegram_account)), amount, token_id);
    }

    pub fn withdraw_from_telegram(&mut self,
                                  telegram_account: ServiceAccountId,
                                  account_id: AccountId,
                                  token_id: TokenAccountId) -> Promise {
        self.internal_withdraw_from_service_account(get_telegram_account(telegram_account), token_id, account_id)
    }

    pub fn transfer_tips_to_deposit(&mut self, telegram_account: ServiceAccountId,
                                    account_id: AccountId,
                                    token_id: TokenAccountId) {
        self.transfer_unclaimed_tips_to_deposit(get_telegram_account(telegram_account), token_id, account_id);
    }
}


pub(crate) fn get_telegram_account(telegram_account: ServiceAccountId) -> ServiceAccount {
    ServiceAccount {
        service: Service::Telegram,
        account_id: Some(telegram_account),
        account_name: None,
    }
}
