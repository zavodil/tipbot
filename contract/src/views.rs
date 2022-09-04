use crate::*;

#[near_bindgen]
impl NearTips {
    /* GLOBAL GETTERS */
    pub fn get_deposits(&self, account_id: AccountId, token_ids: Option<Vec<TokenAccountId>>) -> Vec<(AccountId, WrappedBalance)> {
        self.get_whitelisted_tokens_keys(token_ids)
            .iter()
            .map(|token_id|
                (
                    unwrap_token_id(token_id),
                    self.get_deposit(account_id.clone(), (*token_id).clone())
                ))
            .collect()
    }

    pub fn get_whitelisted_token_ids(&self) -> Vec<AccountId> {
        self.get_whitelisted_tokens_keys(None)
            .iter()
            .map(|token_id| unwrap_token_id(token_id))
            .collect()
    }

    pub fn get_whitelisted_tokens(&self, token_ids: Option<Vec<TokenAccountId>>) -> Vec<(AccountId, WhitelistedTokenOutput)> {
        self.get_whitelisted_tokens_keys(token_ids)
            .iter()
            .map(|token_id|
                 (
                     unwrap_token_id(token_id),
                     self.internal_get_whitelisted_token(token_id)
                 ))
            .collect()
    }

    pub fn get_treasuries(&self, token_ids: Option<Vec<TokenAccountId>>) -> Vec<(AccountId, WrappedBalance)> {
        self.get_whitelisted_tokens_keys(token_ids)
            .iter()
            .map(|token_id|
                (
                    unwrap_token_id(token_id),
                    self.internal_get_treasury(token_id).into()
                ))
            .collect()

    }

    pub fn get_unclaimed_treasuries(&self, account_id: AccountId, token_ids: Option<Vec<TokenAccountId>>) -> Vec<(AccountId, WrappedBalance)> {
        self.get_whitelisted_tokens_keys(token_ids)
            .iter()
            .map(|token_id|
                (
                    unwrap_token_id(token_id),
                    self.get_unclaimed_tiptoken_treasury(&account_id, token_id).into()
                ))
            .collect()
    }

    pub fn get_claimed_treasuries(&self, token_ids: Option<Vec<TokenAccountId>>) -> Vec<(AccountId, WrappedBalance)> {
        self.get_whitelisted_tokens_keys(token_ids)
            .iter()
            .map(|token_id|
                (
                    unwrap_token_id(token_id),
                    self.internal_get_claimed_treasury(token_id).into()
                ))
            .collect()
    }

    pub fn get_service_fees(&self, token_ids: Option<Vec<TokenAccountId>>) -> Vec<(AccountId, WrappedBalance)> {
        self.get_whitelisted_tokens_keys(token_ids)
            .iter()
            .map(|token_id|
                (
                    unwrap_token_id(token_id),
                    self.internal_get_service_fee(token_id).into()
                ))
            .collect()
    }
}

impl NearTips {
    pub(crate) fn get_whitelisted_tokens_keys(&self, token_ids: Option<Vec<TokenAccountId>>) -> Vec<TokenAccountId> {
        if let Some(token_ids) = token_ids {
            token_ids
        } else {
            self.whitelisted_tokens.keys_as_vector().to_vec()
        }
    }


}

pub(crate) fn unwrap_token_id(token_id: &TokenAccountId) -> AccountId {
    token_id.clone().unwrap_or(AccountId::new_unchecked("NEAR".to_string()))
}