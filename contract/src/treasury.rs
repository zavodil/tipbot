use crate::*;

#[near_bindgen]
impl NearTips {
    pub fn get_treasury(&self, token_id: TokenAccountId) -> WrappedBalance {
        self.internal_get_treasury(&token_id).into()
    }

    pub fn get_unclaimed_treasury(&self, token_id: TokenAccountId, account_id: AccountId) -> WrappedBalance {
        self.get_unclaimed_tiptoken_treasury(&account_id, &token_id).into()
    }

    pub fn get_claimed_treasury(&self, token_id: TokenAccountId) -> WrappedBalance {
        self.internal_get_claimed_treasury(&token_id).into()
    }

    pub fn get_service_fee(&self, token_id: TokenAccountId) -> WrappedBalance {
        self.internal_get_service_fee(&token_id).into()
    }
}

impl NearTips {
    pub(crate) fn internal_get_treasury(&self, token_id: &TokenAccountId) -> Balance {
        self.treasury.get(token_id).unwrap_or(0)
    }

    pub(crate) fn internal_get_claimed_treasury(&self, token_id: &TokenAccountId) -> Balance {
        self.treasury_claimed.get(token_id).unwrap_or(0)
    }

    pub(crate) fn internal_get_service_fee(&self, token_id: &TokenAccountId) -> Balance {
        self.service_fees.get(token_id).unwrap_or(0)
    }

    pub(crate) fn get_unclaimed_tiptoken_treasury(&self, sender: &AccountId, token_id: &TokenAccountId) -> Balance{
        self.treasury_by_account.get(
            &TokenByNearAccount {
                account_id: sender.to_owned(),
                token_id: (*token_id).to_owned()
            }).unwrap_or(0)
    }

    pub(crate) fn service_fees_add(&mut self, amount: Balance, token_id: &TokenAccountId) {
        let service_fee_balance = self.internal_get_service_fee(token_id);
        self.service_fees.insert(token_id, &(service_fee_balance + amount));

        events::emit::service_fees_add(amount, token_id);
    }

    pub(crate) fn service_fees_remove(&mut self, amount: Balance, token_id: &TokenAccountId) {
        let service_fee_balance = self.internal_get_service_fee(token_id);
        assert!(amount <= service_fee_balance, "Not enough tokens in service fees");
        self.service_fees.insert(token_id, &(service_fee_balance - amount));

        events::emit::service_fees_remove(amount, token_id);
    }

    pub(crate) fn treasury_add(&mut self, sender: &AccountId, amount: Balance, token_id: &TokenAccountId) {
        /* ADD TO TREASURY */
        let treasury_balance = self.internal_get_treasury(token_id);
        self.treasury.insert(token_id, &(treasury_balance + amount));

        /* INCREASE SENDER'S SHARE OF TREASURY */
        let sender_tiptoken_treasure: Balance = self.get_unclaimed_tiptoken_treasury(sender, token_id);
        self.treasury_by_account.insert(
            &TokenByNearAccount{
                account_id: sender.to_owned(),
                token_id: (*token_id).to_owned()
            },
            &(sender_tiptoken_treasure + amount));

        events::emit::treasury_add(amount, token_id, sender);
    }

    pub(crate) fn treasury_remove(&mut self, sender: &AccountId, amount: Balance, token_id: &TokenAccountId) {
        let treasury_balance = self.internal_get_treasury(token_id);
        let sender_tiptoken_treasure: Balance = self.get_unclaimed_tiptoken_treasury(sender, token_id);
        assert!(amount <= sender_tiptoken_treasure && amount <= treasury_balance, "ERR_NOT_ENOUGH_TOKENS_TO_REMOVE_FROM_TREASURY. Token: {}, amount: {}", get_token_name(token_id), amount);
        self.treasury.insert(token_id, &(treasury_balance - amount));

        // wNEAR and NEAR claims may be double counted
        let treasury_claimed_balance = self.internal_get_claimed_treasury(token_id);
        self.treasury_claimed.insert(token_id, &(treasury_claimed_balance + amount));

        self.treasury_by_account.insert(
            &TokenByNearAccount{
                account_id: sender.to_owned(),
                token_id: (*token_id).to_owned()
            },
            &(sender_tiptoken_treasure - amount));

        events::emit::treasury_remove(amount, token_id, sender);
    }

    pub(crate) fn treasury_claimed_remove(&mut self, token_id: &TokenAccountId, amount: Balance) {
        let treasury_claimed_balance = self.internal_get_claimed_treasury(token_id);
        assert!(treasury_claimed_balance > amount, "ERR_NOT_ENOUGH_TOKENS_TO_REMOVE_FROM_TREASURY_CLAIMED");
        self.treasury_claimed.insert(token_id, &(treasury_claimed_balance - amount));
    }
}