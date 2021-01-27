use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::wee_alloc;
use near_sdk::{env, near_bindgen, AccountId, Balance, Promise};
use near_sdk::json_types::U128;
use std::collections::HashMap;

pub type WrappedBalance = U128;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// Structs in Rust are similar to other languages, and may include impl keyword as shown below
// Note: the names of the structs are not important when calling the smart contract, but the function names are
#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct NearTips {
    deposits: HashMap<AccountId, Balance>,
    telegram_tips: HashMap<String, Balance>,
}

#[near_bindgen]
impl NearTips {
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
        let account_id = env::signer_account_id();
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

    pub fn withdraw_from_telegram(&mut self, telegram_account: String) {
        let balance: Balance = NearTips::get_balance(self, telegram_account.clone()).0;

        assert!(balance > 0, "Nothing to withdraw");

        let account_id = env::predecessor_account_id();
        Promise::new(account_id.clone()).transfer(balance);

        self.telegram_tips.insert(telegram_account.clone(), 0);

        env::log(format!("@{} withdrew {} yNEAR from telegram account {}", account_id, balance, telegram_account).as_bytes());
    }
}

/*
 * The rest of this file holds the inline tests for the code above
 * Learn more about Rust tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
 *
 * To run from contract directory:
 * cargo test -- --nocapture
 *
 * From project root, to run in combination with frontend tests:
 * yarn test
 *
 */
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};

    // mock the context for testing, notice "signer_account_id" that was accessed above from env::
    fn get_context(input: Vec<u8>, is_view: bool) -> VMContext {
        VMContext {
            current_account_id: "alice_near".to_string(),
            signer_account_id: "bob_near".to_string(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id: "carol_near".to_string(),
            input,
            block_index: 0,
            block_timestamp: 0,
            account_balance: 0,
            account_locked_balance: 0,
            storage_usage: 0,
            attached_deposit: 0,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view,
            output_data_receivers: vec![],
            epoch_height: 19,
        }
    }

    #[test]
    fn set_then_get_greeting() {
        let context = get_context(vec![], false);
        testing_env!(context);
        let mut contract = NearTips::default();
        contract.set_greeting("howdy".to_string());
        assert_eq!(
            "howdy".to_string(),
            contract.get_greeting("bob_near".to_string())
        );
    }

    #[test]
    fn get_default_greeting() {
        let context = get_context(vec![], true);
        testing_env!(context);
        let contract = NearTips::default();
        // this test did not call set_greeting so should return the default "Hello" greeting
        assert_eq!(
            "Hello".to_string(),
            contract.get_greeting("francis.near".to_string())
        );
    }
}
