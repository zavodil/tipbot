use crate::*;

#[near_bindgen]
impl NearTips {
    pub fn get_unclaimed_tips_balance(&self,
                                      account: ServiceAccount,
                                      token_id: TokenAccountId) -> WrappedBalance {
        self.internal_get_unclaimed_tips_balance(account, token_id).into()
    }
}


impl NearTips {
    pub(crate) fn increase_unclaimed_tips(&mut self,
                                          service_account: ServiceAccount,
                                          token_id: TokenAccountId,
                                          amount: Balance) {
        let key = TokenByServiceAccount {
            account: service_account,
            token_id,
        };

        let unclaimed_tips_balance: Balance = self.unclaimed_tips.get(&key).unwrap_or(0);

        self.unclaimed_tips.insert(&key, &(unclaimed_tips_balance + amount));
    }

    pub(crate) fn internal_get_unclaimed_tips_balance(&self,
                                                      account: ServiceAccount,
                                                      token_id: TokenAccountId) -> Balance {
        let key = TokenByServiceAccount {
            account,
            token_id,
        };

        self.unclaimed_tips.get(&key).unwrap_or_default()
    }

    pub(crate) fn set_unclaimed_tips_balance(&mut self, account: ServiceAccount, token_id: TokenAccountId, amount: Balance) {
        let key = TokenByServiceAccount {
            account,
            token_id,
        };

        self.unclaimed_tips.insert(&key, &amount);
    }
}