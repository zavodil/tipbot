use crate::*;

/// Single swap action.
#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SwapAction {
    /// Pool which should be used for swapping.
    pub pool_id: u32,
    /// Token to swap from.
    pub token_in: AccountId,
    /// Amount to exchange.
    /// If amount_in is None, it will take amount_out from previous step.
    /// Will fail if amount_in is None on the first step.
    pub amount_in: Option<U128>,
    /// Token to swap into.
    pub token_out: AccountId,
    /// Required minimum amount of token_out.
    pub min_amount_out: U128,
}

pub const NEAR: TokenAccountId = None;

#[near_bindgen]
impl NearTips {
    // tiptoken are bought on AMM and go to the TipBot deposit, increasing the user's internal balance
    pub fn claim_tiptoken(&mut self, token_id: TokenAccountId, amount: Option<WrappedBalance>) {
        let account_id = env::predecessor_account_id();

        let amount: Balance = if let Some (amount) = amount {
            amount.0
        }
            else {
                self.get_unclaimed_tiptoken_treasury(&account_id, &token_id)
        };

        require!(amount > 0, "ERR_AMOUNT_IS_NOT_POSITIVE");

        if token_id.is_some() {
            // FT
            self.treasury_remove(&account_id, amount, &token_id);

            self.internal_claim(account_id, token_id, amount);
        }
        else {
            // NEAR
            self.treasury_remove(&account_id, amount, &NEAR);

            let wrap_near_contract_id = self.get_wrap_near_contract_id();

            ext_wrap_near::ext(wrap_near_contract_id)
                .with_attached_deposit(amount)
                .with_static_gas(GAS_FOR_WRAP_NEAR)
                .near_deposit()

                .then(
                    ext_self::ext(env::current_account_id())
                        .with_static_gas(GAS_FOR_ON_WRAP_NEAR)
                        .on_wrap_near(
                            account_id.clone(),
                            U128::from(amount)
                        )
                );
        }


    }

    #[private]
    pub fn after_swap(
        &mut self,
        #[callback_result] amount_out: Result<U128, PromiseError>,
        swap_contract_id: AccountId,
        account_id: AccountId,
        amount_in: WrappedBalance,
        token_id: AccountId,
        service_fee: WrappedBalance,
    ) -> U128 {
        if let Ok(amount_out) = amount_out {
            if amount_out.0 > 0 {
                log!("{:?} deposited {} tiptokens", account_id, amount_out.0);
                self.increase_deposit(account_id.clone(), Some(self.get_tiptoken_account_id()), amount_out.0);

                let tiptoken_account_id = self.get_tiptoken_account_id();

                ext_ft_contract::ext(swap_contract_id)
                    .with_attached_deposit(1)
                    .with_static_gas(GAS_FOR_WITHDRAW)
                    .withdraw(
                        tiptoken_account_id.clone(),
                        amount_out,
                        Some(false)
                    )
                .then(
                    ext_self::ext(env::current_account_id())
                        .with_static_gas(GAS_FOR_ON_WITHDRAW)
                        .on_withdraw(
                            account_id,
                            amount_out,
                            tiptoken_account_id,
                        )
                );

                return amount_out;

            }
        }

        let token_account_id = Some(token_id.clone());

        let total_refund = amount_in.0 + service_fee.0;
        log!("Error. Refunding {} of {} to {} ({} are service fees)", total_refund, token_id, account_id, service_fee.0);
        self.treasury_add(&account_id, total_refund, &token_account_id);
        self.service_fees_remove(service_fee.0, &token_account_id);
        self.treasury_claimed_remove(&token_account_id, amount_in.0);

        U128::from(0)
    }

    #[private]
    pub fn on_withdraw(&mut self, account_id: AccountId, amount_withdraw: WrappedBalance, token_id: AccountId) -> U128 {
        let transfer_succeeded = is_promise_success();

        log!("Withdraw succeed: {:?}. AccountId: {}. Amount to withdraw: {} of {:?} ", transfer_succeeded, account_id.clone(), amount_withdraw.0, token_id);

        if !transfer_succeeded {
            self.deposit_to_near_account(&account_id, amount_withdraw.0, Some(token_id), false);
            return amount_withdraw;
        }

        U128::from(0)
    }
}

impl NearTips {
    pub(crate) fn internal_claim(&mut self, account_id: AccountId, token_id: TokenAccountId, amount: Balance) {
        require!(amount > 0, "ERR_AMOUNT_IS_NOT_POSITIVE");

        let (dex, swap_contract_id, swap_pool_id) = self.get_swap_contract(&token_id);

        let service_fee = self.get_service_fee_fraction(amount);
        let amount_to_swap = amount - service_fee;

        self.service_fees_add(service_fee, &token_id);

        let tiptoken_account_id = self.get_tiptoken_account_id();

        if let Some(token_account_id) = token_id.clone() {
            require!(token_account_id != tiptoken_account_id, "ERR_TIP_TOKEN_IS_NOT_CLAIMABLE");

            let swap_actions =
                get_swap_action(&token_account_id, &tiptoken_account_id, amount_to_swap,  &dex, swap_pool_id);

            log!("Deposit {} of {} to DEX", amount_to_swap, token_account_id.to_string());

            // pure deposit
            ext_ft_contract::ext(token_account_id.clone())
                .with_attached_deposit(1)
                .with_static_gas(GAS_FOR_FT_TRANSFER_CALL)
                .ft_transfer_call(
                    swap_contract_id.clone(),
                    WrappedBalance::from(amount_to_swap),
                    None,
                    "".to_string(),
                )
                // swap
                .then(
                    ext_ft_contract::ext(swap_contract_id.clone())
                        .with_attached_deposit(1)
                        .with_static_gas(GAS_FOR_SWAP)
                        .swap(
                            swap_actions,
                            Some(self.get_operator_id())
                        )
                )
                .then(
                    ext_self::ext(env::current_account_id())
                        .with_static_gas(GAS_FOR_AFTER_SWAP)
                        .after_swap(
                            swap_contract_id,
                            account_id,
                            WrappedBalance::from(amount_to_swap),
                            token_account_id,
                            WrappedBalance::from(service_fee),
                        ));
        }
    }
}

pub(crate) fn get_swap_action (token_in: &AccountId, token_out: &AccountId, amount: Balance, dex: &DEX, swap_pool_id: u32) -> Vec<SwapAction> {
    match dex {
        DEX::RefFinance => vec![SwapAction {
            pool_id: swap_pool_id,
            token_in: token_in.clone(),
            amount_in: Some(U128(amount)),
            token_out: token_out.clone(),
            min_amount_out: U128(1),
        }]
    }
}