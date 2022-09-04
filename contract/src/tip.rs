use crate::*;

#[near_bindgen]
impl NearTips {
    pub fn tip (&mut self,
                        receiver_account_id: Option<AccountId>,
                        receiver_service_account: Option<ServiceAccount>,
                        amount: WrappedBalance,
                        token_id: TokenAccountId) {
        let account_id = env::predecessor_account_id();
        self.internal_tip(account_id, receiver_service_account, receiver_account_id, amount.0, token_id);
    }

    #[payable]
    pub fn tip_near(&mut self,
                    receiver_account_id: Option<AccountId>,
                    receiver_service_account: Option<ServiceAccount>) {
        let account_id = env::predecessor_account_id();
        let amount = env::attached_deposit();
        self.internal_tip(account_id, receiver_service_account, receiver_account_id, amount, NEAR);
    }
}

impl NearTips {
    pub(crate) fn internal_tip(&mut self,
                               sender_account_id: AccountId,
                               receiver_service_account: Option<ServiceAccount>,
                               receiver_account_id: Option<AccountId>,
                               amount: Balance,
                               token_id: TokenAccountId) {
        self.assert_tip_available();
        self.assert_check_whitelisted_token(&token_id);

        if let Some(receiver_service_account) = receiver_service_account.clone() {
            require!(receiver_account_id.is_none(), "ERR_TOO_MANY_TIP_RECEIVERS");
            receiver_service_account.verify();
        }
        else {
            require!(receiver_account_id.is_some(), "ERR_NO_TIP_RECEIVER");
        }

        assert_ne!(amount, 0, "Tip amount must be a positive number");
        assert!(amount > self.get_token_min_tip(&token_id), "Tip is too small");

        let deposit = self.internal_get_deposit(sender_account_id.clone(), token_id.clone());
        assert!(amount <= deposit, "Not enough tokens (Deposit: {}. Requested: {})", deposit, amount);

        // Tips in Tiptokens sends with NO fees
        let is_treasury_fee_needed = token_id != Some(self.get_tiptoken_account_id());

        let treasury_fee: Balance = if is_treasury_fee_needed {
            self.get_treasure_fee_fraction(amount)
        } else {
            0
        };
        let tip_without_treasury_fee: Balance = amount - treasury_fee;

        log!("Treasury. needed: {:?}. treasury_fee: {}", is_treasury_fee_needed, treasury_fee);

        /* SEND TOKENS TO TIP RECEIVER */
        self.increase_balance(receiver_service_account, receiver_account_id, token_id.clone(), tip_without_treasury_fee, &sender_account_id);

        /* STORE FEE AT TREASURY */
        self.treasury_add(&sender_account_id, treasury_fee, &token_id);

        /* REMOVE TOKENS FROM TIP SENDER */
        // don't use helper to avoid second deposit check
        self.deposits.insert(
            &TokenByNearAccount {
                account_id: sender_account_id,
                token_id,
            },
            &(deposit - amount));
    }

    /*
    pub(crate) fn internal_tip_service_account(&mut self,
                                               sender_account_id: AccountId,
                                               receiver_account: ServiceAccount,
                                               amount: Balance,
                                               token_id: TokenAccountId) {
        self.assert_tip_available();
        self.assert_check_whitelisted_token(&token_id);
        receiver_account.verify();

        assert_ne!(amount, 0, "Tip amount must be a positive number");
        assert!(amount > self.get_token_min_tip(&token_id), "Tip is too small");

        let deposit = self.internal_get_deposit(sender_account_id.clone(), token_id.clone());
        assert!(amount <= deposit, "Not enough tokens (Deposit: {}. Requested: {})", deposit, amount);

        // Tips in Tiptokens sends with NO fees
        let is_treasury_fee_needed = token_id != Some(self.get_tiptoken_account_id());

        let treasury_fee: Balance = if is_treasury_fee_needed {
            self.get_treasure_fee_fraction(amount)
        } else {
            0
        };
        let tip_without_treasury_fee: Balance = amount - treasury_fee;

        log!("Treasury. needed: {:?}. treasury_fee: {}", is_treasury_fee_needed, treasury_fee);

        /* SEND TOKENS TO TIP RECEIVER */
        self.increase_balance(receiver_account, token_id.clone(), tip_without_treasury_fee, &sender_account_id);

        /* STORE FEE AT TREASURY */
        self.treasury_add(&sender_account_id, treasury_fee, &token_id);

        /* REMOVE TOKENS FROM TIP SENDER */
        // don't use helper to avoid second deposit check
        self.deposits.insert(
            &TokenByNearAccount {
                account_id: sender_account_id,
                token_id,
            },
            &(deposit - amount));
    }*/


    pub(crate) fn increase_balance(&mut self,
                                   receiver_service_account: Option<ServiceAccount>,
                                   receiver_account_id: Option<AccountId>,
                                   token_id: TokenAccountId,
                                   amount: Balance,
                                   sender_account_id: &AccountId) {
        if let Some(receiver_service_account) = receiver_service_account {
            if let Some(receiver_account_id) = self.service_accounts.get(&receiver_service_account) {
                self.increase_deposit(receiver_account_id.clone(), token_id.clone(), amount);

                log!("{} tipped {} of {:?} to service account {}", sender_account_id, amount, get_token_name(&token_id), receiver_account_id);
                events::emit::increase_balance(sender_account_id, &Some(receiver_account_id), &Some(receiver_service_account), amount, &token_id);
            } else {
                self.increase_unclaimed_tips(receiver_service_account.clone(), token_id.clone(), amount);

                log!("{} tipped {} of {:?} to service account {}", sender_account_id, amount, get_token_name(&token_id), receiver_service_account);
                events::emit::increase_balance(sender_account_id, &None, &Some(receiver_service_account), amount, &token_id);
            }
        }
        else {
            let receiver_account_id = receiver_account_id.unwrap();
            self.increase_deposit(receiver_account_id.clone(), token_id.clone(), amount);

            log!("{} tipped {} of {:?} to NEAR account {}", sender_account_id, amount, get_token_name(&token_id), receiver_account_id);
            events::emit::increase_balance(sender_account_id, &Some(receiver_account_id), &None, amount, &token_id);
        }
    }
}