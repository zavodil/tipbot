use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::wee_alloc;
use near_sdk::{env, near_bindgen, AccountId, Balance, Promise,  Gas, ext_contract, PromiseResult};
use near_sdk::json_types::U128;
use std::collections::HashMap;

pub type WrappedBalance = U128;

const MASTER_ACCOUNT_ID: &str = "zavodil.testnet"; // telegram bot master account id
const WITHDRAW_COMMISSION: Balance = 3_000_000_000_000_000_000_000; // 0.003 NEAR
const ACCESS_KEY_ALLOWANCE: Balance = 1_000_000_000_000_000_000_000_000;
const BASE_GAS: Gas = 25_000_000_000_000;
const NO_DEPOSIT: Balance = 0;

#[ext_contract(linkdrop)]
pub trait ExtLinkdrop {
    fn send(&self, public_key: String);
}

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct NearTips {
    deposits: HashMap<AccountId, Balance>,
    telegram_tips: HashMap<String, Balance>,
}

#[ext_contract(ext_self)]
pub trait ExtNearTips {
    fn on_withdraw_from_telegram(&mut self, predecessor_account_id: AccountId, amount: Balance, telegram_account: String) -> bool;
    fn on_withdraw(&mut self, predecessor_account_id: AccountId, deposit: Balance) -> bool;
    fn on_withdraw_linkdrop(&mut self, amount: Balance, telegram_account: String, public_key: String) -> bool;
}

fn is_promise_success() -> bool {
    assert_eq!(
        env::promise_results_count(),
        1,
        "Contract expected a result on the callback"
    );
    match env::promise_result(0) {
        PromiseResult::Successful(_) => true,
        _ => false,
    }
}

#[near_bindgen]
impl NearTips {
    fn get_linkdrop_contract() -> String {
        //  linkdrop.near ??????? for Mainnet, linkdrop.zavodil.testnet for Testnet
        "linkdrop.zavodil.testnet".to_string()
    }

    #[payable]
    pub fn deposit(&mut self) {
        let account_id: AccountId = env::predecessor_account_id();
        let attached_deposit: Balance = env::attached_deposit();
        let deposit: Balance = NearTips::get_deposit(self, account_id.clone()).0;
        self.deposits.insert(account_id, deposit + attached_deposit);
    }

    pub fn get_deposit(&self, account_id: AccountId) -> WrappedBalance {
        match self.deposits.get(&account_id) {
            Some(deposit) => WrappedBalance::from(*deposit),
            None => WrappedBalance::from(0)
        }
    }

    pub fn get_balance(&self, telegram_account: String) -> WrappedBalance {
        match self.telegram_tips.get(&telegram_account) {
            Some(balance) => WrappedBalance::from(*balance),
            None => WrappedBalance::from(0)
        }
    }

    pub fn send_tip_to_telegram(&mut self, telegram_account: String, amount: WrappedBalance) {
        let account_id = env::predecessor_account_id();
        let deposit: Balance = NearTips::get_deposit(self, account_id.clone()).0;

        assert!(
            amount.0 <= deposit,
            "Not enough tokens deposited to tip (Deposit: {}. Requested: {})",
            deposit, amount.0
        );

        let balance: Balance = NearTips::get_balance(self, telegram_account.clone()).0;
        self.telegram_tips.insert(telegram_account.clone(), balance + amount.0);

        self.deposits.insert(account_id.clone(), deposit - amount.0);

        env::log(format!("@{} tipped {} yNEAR for telegram account {}", account_id, amount.0, telegram_account).as_bytes());
    }

    pub fn withdraw_from_telegram(&mut self, telegram_account: String, account_id: AccountId) -> Promise {
        assert!(env::predecessor_account_id() == MASTER_ACCOUNT_ID, "No access");
        assert!(env::is_valid_account_id(account_id.as_bytes()), "Account @{} is invalid", account_id);

        let balance: Balance = NearTips::get_balance(self, telegram_account.clone()).0;
        assert!(balance > WITHDRAW_COMMISSION, "Not enough tokens to pay withdraw commission");

        let amount = balance - WITHDRAW_COMMISSION;
        Promise::new(account_id.clone())
            .transfer(amount)
            .then(ext_self::on_withdraw_from_telegram(
                env::predecessor_account_id(),
                amount,
                telegram_account,
                &env::current_account_id(),
                NO_DEPOSIT,
                BASE_GAS,
            ))
    }

    pub fn on_withdraw_from_telegram(&mut self, predecessor_account_id: AccountId, amount: Balance, telegram_account: String) -> bool {
        assert_eq!(
            env::predecessor_account_id(),
            env::current_account_id(),
            "Callback can only be called from the contract"
        );
        let withdraw_succeeded = is_promise_success();
        if withdraw_succeeded {
            self.telegram_tips.insert(telegram_account.clone(), 0);
            Promise::new(MASTER_ACCOUNT_ID.to_string()).transfer(WITHDRAW_COMMISSION);

            env::log(format!("@{} withdrew {} yNEAR from telegram account {}. Withdraw commission: {} yNEAR",
                             predecessor_account_id, amount, telegram_account, WITHDRAW_COMMISSION).as_bytes());
        }

        withdraw_succeeded
    }

    pub fn withdraw(&mut self) -> Promise {
        let account_id = env::predecessor_account_id();
        let deposit: Balance = NearTips::get_deposit(self, account_id.clone()).0;

        assert!(deposit > 0, "Missing deposit");

        Promise::new(account_id.clone())
            .transfer(deposit)
            .then(ext_self::on_withdraw(
                account_id,
                deposit,
                &env::current_account_id(),
                NO_DEPOSIT,
                BASE_GAS,
            ))

    }

    pub fn on_withdraw(&mut self, predecessor_account_id: AccountId, deposit: Balance) -> bool {
        assert_eq!(
            env::predecessor_account_id(),
            env::current_account_id(),
            "Callback can only be called from the contract"
        );
        let withdraw_succeeded = is_promise_success();
        if withdraw_succeeded {
            self.deposits.insert(predecessor_account_id.clone(), 0);

            env::log(format!("@{} withdrew {} yNEAR from internal deposit", predecessor_account_id, deposit).as_bytes());
        }

        withdraw_succeeded
    }

    pub fn withdraw_linkdrop(&mut self, public_key: String, telegram_account: String) -> Promise {
        assert!(env::predecessor_account_id() == MASTER_ACCOUNT_ID, "No access");
        let balance: Balance = NearTips::get_balance(self, telegram_account.clone()).0;
        assert!(balance > WITHDRAW_COMMISSION + ACCESS_KEY_ALLOWANCE, "Not enough tokens to pay for key allowance and withdraw commission");

        let amount = balance - WITHDRAW_COMMISSION;
        linkdrop::send(public_key.clone(), &NearTips::get_linkdrop_contract(), amount, BASE_GAS).
            then(ext_self::on_withdraw_linkdrop(
            amount,
            telegram_account,
            public_key,
            &env::current_account_id(),
            NO_DEPOSIT,
            BASE_GAS,
        ))
    }

    pub fn on_withdraw_linkdrop(&mut self, amount: Balance, telegram_account: String, public_key: String) -> bool {
        assert_eq!(
            env::predecessor_account_id(),
            env::current_account_id(),
            "Callback can only be called from the contract"
        );
        let withdraw_succeeded = is_promise_success();
        if withdraw_succeeded {
            self.telegram_tips.insert(telegram_account.clone(), 0);
            Promise::new(MASTER_ACCOUNT_ID.to_string()).transfer(WITHDRAW_COMMISSION);

            env::log(format!("Telegram account {} withdrew {} yNEAR with linkDrop for public key {}. Withdraw commission: {} yNEAR",
                             telegram_account, amount, public_key, WITHDRAW_COMMISSION).as_bytes());

        }

        withdraw_succeeded
    }

    pub fn transfer_tips_to_deposit(&mut self, telegram_account: String, account_id: AccountId) {
        assert!(env::predecessor_account_id() == MASTER_ACCOUNT_ID, "No access");
        assert!(env::is_valid_account_id(account_id.as_bytes()), "Account @{} is invalid", account_id);

        let balance: Balance = NearTips::get_balance(self, telegram_account.clone()).0;
        assert!(balance > WITHDRAW_COMMISSION, "Not enough tokens to pay withdraw commission");

        let deposit: Balance = NearTips::get_deposit(self, account_id.clone()).0;
        self.deposits.insert(account_id.clone(), deposit + balance - WITHDRAW_COMMISSION);

        self.telegram_tips.insert(telegram_account.clone(), 0);

        env::log(format!("@{} transfer {} yNEAR from telegram account {}. Withdraw commission: {} yNEAR",
                         account_id, balance, telegram_account, WITHDRAW_COMMISSION).as_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};

    fn alice_account() -> AccountId {
        MASTER_ACCOUNT_ID.to_string()
    }

    fn bob_account() -> AccountId {
        "bob.near".to_string()
    }

    fn alice_telegram() -> AccountId {
        "1234".to_string()
    }

    fn bob_telegram() -> AccountId {
        "5678".to_string()
    }

    pub fn get_context(
        predecessor_account_id: AccountId,
        attached_deposit: u128,
        is_view: bool,
    ) -> VMContext {
        VMContext {
            current_account_id: predecessor_account_id.clone(),
            signer_account_id: predecessor_account_id.clone(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id,
            input: vec![],
            block_index: 1,
            block_timestamp: 0,
            epoch_height: 1,
            account_balance: 0,
            account_locked_balance: 0,
            storage_usage: 10u64.pow(6),
            attached_deposit,
            prepaid_gas: 10u64.pow(15),
            random_seed: vec![0, 1, 2],
            is_view,
            output_data_receivers: vec![],
        }
    }

    fn ntoy(near_amount: Balance) -> Balance {
        near_amount * 10u128.pow(24)
    }

    #[test]
    fn test_deposit() {
        let context = get_context(alice_account(), ntoy(100), false);
        testing_env!(context.clone());

        let mut contract = NearTips::default();
        contract.deposit();

        assert_eq!(
            ntoy(100),
            contract.get_deposit(alice_account()).0
        );
    }

    #[test]
    fn test_withdraw() {
        let context = get_context(alice_account(), ntoy(100), false);
        testing_env!(context.clone());

        let mut contract = NearTips::default();

        contract.deposit();

        assert_eq!(
            ntoy(100),
            contract.get_deposit(alice_account()).0
        );

        contract.withdraw();
        assert_eq!(
            ntoy(0),
            contract.get_deposit(alice_account()).0
        );
    }

    #[test]
    fn test_withdraw_from_telegram() {
        let mut context = get_context(alice_account(), ntoy(30), false);
        testing_env!(context.clone());

        let mut contract = NearTips::default();

        contract.deposit();

        contract.send_tip_to_telegram(alice_telegram(), WrappedBalance::from(ntoy(30)));

        contract.withdraw_from_telegram(alice_telegram(), alice_account());
        context.account_balance += ntoy(30) - WITHDRAW_COMMISSION;

        assert_eq!(
            ntoy(30) - WITHDRAW_COMMISSION,
            context.account_balance
        );

        assert_eq!(
            WITHDRAW_COMMISSION,
            env::account_balance()
        );

        assert_eq!(
            0,
            contract.get_balance(alice_telegram()).0
        );
    }

    #[test]
    fn test_tip() {
        let context = get_context(alice_account(), ntoy(100), false);
        testing_env!(context.clone());

        let mut contract = NearTips::default();

        contract.deposit();

        contract.send_tip_to_telegram(alice_telegram(), WrappedBalance::from(ntoy(30)));
        assert_eq!(
            ntoy(30),
            contract.get_balance(alice_telegram()).0
        );

        assert_eq!(
            ntoy(70),
            contract.get_deposit(alice_account()).0
        );

        contract.send_tip_to_telegram(alice_telegram(), WrappedBalance::from(ntoy(70)));
        assert_eq!(
            ntoy(100),
            contract.get_balance(alice_telegram()).0
        );

        assert_eq!(
            ntoy(0),
            contract.get_deposit(alice_account()).0
        );
    }

    #[test]
    fn test_transfer_tips() {
        let context = get_context(alice_account(), ntoy(100), false);
        testing_env!(context.clone());

        let mut contract = NearTips::default();

        contract.deposit();

        contract.send_tip_to_telegram(bob_telegram(), WrappedBalance::from(ntoy(30)));

        assert_eq!(
            ntoy(30),
            contract.get_balance(bob_telegram()).0
        );

        contract.transfer_tips_to_deposit(bob_telegram(), bob_account());


        assert_eq!(
            ntoy(30) - WITHDRAW_COMMISSION,
            contract.get_deposit(bob_account()).0
        );
    }
}