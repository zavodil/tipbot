use crate::*;

pub const GAS_FOR_UNWRAP_NEAR: Gas = Gas(Gas::ONE_TERA.0 * 10);
pub const GAS_FOR_ON_UNWRAP_NEAR: Gas = Gas(Gas::ONE_TERA.0 * 10);

#[near_bindgen]
impl NearTips {
    // wnear stays in treasury after failed claims, user may redeam it
    pub fn unwrap_near_treasury(&mut self, amount: Option<WrappedBalance>) {
        let account_id = env::predecessor_account_id();

        let wrap_near_contract_id_unwrapped = self.get_wrap_near_contract_id();
        let wrap_near_contract_id = Some(wrap_near_contract_id_unwrapped.clone());

        let amount: Balance = if let Some(amount) = amount {
            amount.0
        } else {
            self.get_unclaimed_tiptoken_treasury(&account_id, &wrap_near_contract_id)
        };

        // service fee are already stored on the contract
        let service_fee = self.get_service_fee_fraction(amount);
        let amount_to_withdraw = amount - service_fee;

        self.treasury_remove(&account_id, amount, &wrap_near_contract_id);

        ext_ft_contract::ext(self.get_swap_contract_id(&wrap_near_contract_id))
            .with_attached_deposit(1)
            .with_static_gas(GAS_FOR_WITHDRAW)
            .withdraw(
                wrap_near_contract_id_unwrapped.clone(),
                U128::from(amount_to_withdraw),
                Some(false),
            )

            .then(
                ext_wrap_near::ext(wrap_near_contract_id_unwrapped)
                    .with_attached_deposit(ONE_YOCTO)
                    .with_static_gas(GAS_FOR_UNWRAP_NEAR)
                    .near_withdraw(
                        U128::from(amount)
                    )
            )

            .then(
                ext_self::ext(env::current_account_id())
                    .with_static_gas(GAS_FOR_ON_UNWRAP_NEAR)
                    .on_unwrap_near(
                        account_id.clone(),
                        U128::from(amount),
                    )
            );
    }

    #[private]
    pub fn on_wrap_near(&mut self, account_id: AccountId, amount: WrappedBalance) -> U128 {
        if is_promise_success() {
            log!("Wrap succeed. Deposit: {} wNEAR", amount.0);

            let wnear_token_account_id = Some(self.get_wrap_near_contract_id());

            self.treasury_add(&account_id, amount.0, &wnear_token_account_id);

            self.internal_claim(account_id, wnear_token_account_id, amount.0);

            amount
        } else {
            log!("Wrap failed. Refund: {} NEAR", amount.0);

            self.treasury_add(&account_id, amount.0, &NEAR);

            U128::from(0)
        }
    }

    #[private]
    pub fn on_unwrap_near(&mut self, account_id: AccountId, amount: WrappedBalance) -> U128 {
        if is_promise_success() {
            log!("Unwrap succeed. Deposit: {} NEAR", amount.0);

            self.treasury_add(&account_id, amount.0, &NEAR);

            amount
        } else {
            log!("Wrap failed. Refund: {} NEAR", amount.0);

            let wnear_token_account_id = Some(self.get_wrap_near_contract_id());

            self.treasury_add(&account_id, amount.0, &wnear_token_account_id);

            U128::from(0)
        }
    }
}