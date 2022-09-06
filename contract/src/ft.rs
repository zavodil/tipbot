use crate::*;

pub const ONE_YOCTO: Balance = 1;

pub const GAS_FOR_FT_TRANSFER: Gas = Gas(Gas::ONE_TERA.0 * 10);
pub const GAS_FOR_AFTER_FT_TRANSFER: Gas = Gas(Gas::ONE_TERA.0 * 10);

pub const GAS_FOR_FT_TRANSFER_CALL: Gas = Gas(Gas::ONE_TERA.0 * 35);

pub const GAS_FOR_SWAP: Gas = Gas(Gas::ONE_TERA.0 * 10);

// GAS_FOR_AFTER_SWAP includes GAS_FOR_WITHDRAW + GAS_FOR_ON_WITHDRAW
pub const GAS_FOR_WITHDRAW: Gas = Gas(Gas::ONE_TERA.0 * 50);
pub const GAS_FOR_ON_WITHDRAW: Gas = Gas(Gas::ONE_TERA.0 * 20);
pub const GAS_FOR_AFTER_SWAP: Gas = Gas(Gas::ONE_TERA.0 * 80);

pub const GAS_FOR_WRAP_NEAR: Gas = Gas(Gas::ONE_TERA.0 * 5);
// GAS_FOR_ON_WRAP_NEAR includes GAS_FOR_FT_TRANSFER_CALL + GAS_FOR_AFTER_FT_TRANSFER + GAS_FOR_SWAP + GAS_FOR_AFTER_SWAP
pub const GAS_FOR_ON_WRAP_NEAR: Gas = Gas(Gas::ONE_TERA.0 * 180);




pub const NO_DEPOSIT: Balance = 0;

#[ext_contract(ext_self)]
pub trait ExtNearTips {
    fn after_ft_transfer_deposit(&mut self, account_id: AccountId, amount: WrappedBalance, token_account_id: AccountId) -> bool;
    fn after_swap(&mut self,
                  #[callback_result] amount_out: Result<U128, PromiseError>,
                  swap_contract_id: AccountId,
                  account_id: AccountId,
                  amount_in: WrappedBalance,
                  token_id: AccountId,
                  service_fee: WrappedBalance) -> bool;
    fn on_withdraw(&mut self,
                   account_id: AccountId, amount_withdraw: WrappedBalance, token_id: AccountId) -> bool;
    fn on_wrap_near(&mut self, account_id: AccountId, amount: WrappedBalance) -> U128;
    fn on_unwrap_near(&mut self, account_id: AccountId, amount: WrappedBalance) -> U128;
}

#[ext_contract(ext_ft_contract)]
trait ExtFtContract {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
    fn ft_transfer_call(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>, msg: String) -> PromiseOrValue<U128>;

    // https://github.com/ref-finance/ref-contracts/blob/main/ref-exchange/src/lib.rs#L221
    fn swap(&mut self, actions: Vec<SwapAction>, referral_id: Option<AccountId>) -> U128;
    //https://github.com/ref-finance/ref-contracts/blob/main/ref-exchange/src/account_deposit.rs#L259
    fn withdraw(&mut self, token_id: AccountId, amount: U128, unregister: Option<bool>) -> Promise;
}

#[ext_contract(ext_wrap_near)]
trait ExtWrapNearContract {
    fn near_deposit(&mut self);
    fn near_withdraw(&mut self, amount: WrappedBalance);
}

#[near_bindgen]
impl NearTips {
    #[allow(unused_variables)]
    pub fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let token_id = Some(env::predecessor_account_id());
        if self.check_whitelisted_token(&token_id) && self.check_withdraw_available() {
            self.increase_deposit(sender_id, token_id, amount.0);
            PromiseOrValue::Value(0.into())
        } else {
            log!("Deposit is not available");
            PromiseOrValue::Value(amount)
        }
    }

    #[private]
    pub fn after_ft_transfer_deposit(
        &mut self,
        account_id: AccountId,
        amount: WrappedBalance,
        token_account_id: AccountId,
    ) -> bool {
        let promise_success = is_promise_success();
        if !promise_success {
            log!(
                "Token {} withdraw for user {} failed. Amount to recharge: {}",
                token_account_id,
                account_id,
                amount.0
            );

            let token_id = Some(token_account_id);

            let current_deposit = self.internal_get_deposit(account_id.clone(), token_id.clone());

            self.increase_deposit(account_id, token_id, current_deposit + amount.0);
        }
        promise_success
    }
}

impl NearTips {
    pub(crate) fn internal_withdraw_ft(&mut self, account_id: AccountId, token_account_id: AccountId, deposit: Balance) -> Promise {
        ext_ft_contract::ext(token_account_id.clone())
            .with_attached_deposit(1)
            .with_static_gas(GAS_FOR_FT_TRANSFER)
            .ft_transfer(
                account_id.clone(),
                deposit.into(),
                Some(format!(
                    "Claiming tips: {} of {:?} from @{}",
                    deposit,
                    token_account_id,
                    env::current_account_id()
                )),
            )
            .then(
                ext_self::ext(env::current_account_id())
                    .with_static_gas(GAS_FOR_AFTER_FT_TRANSFER)
                    .after_ft_transfer_deposit(
                        account_id,
                        deposit.into(),
                        token_account_id)
            )
    }
}