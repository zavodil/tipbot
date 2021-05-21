use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::wee_alloc;
use near_sdk::{env, near_bindgen, AccountId, Balance, Promise, Gas, ext_contract, PromiseResult, PromiseOrValue};
use near_sdk::json_types::U128;
use std::collections::HashMap;

pub type WrappedBalance = U128;

// TODO Set telegram bot master account id
const MASTER_ACCOUNT_ID: &str = "nearup_bot.app.near";
//const MASTER_ACCOUNT_ID: &str = "zavodil.testnet";

const WITHDRAW_COMMISSION: Balance = 3_000_000_000_000_000_000_000;
// 0.003 NEAR
const ACCESS_KEY_ALLOWANCE: Balance = 1_000_000_000_000_000_000_000_000;
const BASE_GAS: Gas = 25_000_000_000_000;
const CALLBACK: Gas = 25_000_000_000_000;
const NO_DEPOSIT: Balance = 0;

#[ext_contract(linkdrop)]
pub trait ExtLinkdrop {
    fn send(&self, public_key: String);
}

#[ext_contract(auth)]
pub trait ExtAuth {
    fn get_contacts(&self, account_id: AccountId) -> Option<Vec<Contact>>;
    fn get_owners(&self, contact: Contact) -> Vec<AccountId>;
}

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct NearTips {
    deposits: HashMap<AccountId, Balance>,
    telegram_tips: HashMap<String, Balance>,
    tips: HashMap<AccountId, Vec<Tip>>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Tip {
    pub contact: Contact,
    pub amount: Balance,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct TipWrapped {
    pub contact: Contact,
    pub amount: WrappedBalance,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Contact {
    pub category: ContactCategories,
    pub value: String,
}


#[derive(BorshSerialize, BorshDeserialize, Eq, PartialEq, Debug, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum ContactCategories {
    Email,
    Telegram,
    Twitter,
    Github,
    NearGovForum,
    Discord,
    Facebook,
}

#[ext_contract(ext_self)]
pub trait ExtNearTips {
    fn on_withdraw_from_telegram(&mut self, predecessor_account_id: AccountId, amount: Balance, telegram_account: String) -> bool;
    fn on_withdraw(&mut self, predecessor_account_id: AccountId, deposit: Balance) -> bool;
    fn on_withdraw_linkdrop(&mut self, amount: Balance, telegram_account: String, public_key: String) -> bool;
    fn on_get_contacts_on_withdraw_tip_for_current_account(&mut self, #[callback] contacts: Option<Vec<Contact>>, recipient_account_id: AccountId, recipient_contact: Contact, balance: Balance) -> bool;
    fn on_get_contact_owner_on_tip_contact_to_deposit(&mut self, #[callback] accounts: Option<Vec<AccountId>>, sender_account_id: AccountId, contact: Contact, amount: Balance) -> bool;
    fn on_get_contact_owner_on_tip_contact_with_attached_tokens(&mut self, #[callback] accounts: Option<Vec<AccountId>>, sender_account_id: AccountId, contact: Contact, deposit: Balance) -> bool;
    fn on_get_contact_owner_on_withdraw_tip_for_undefined_account(&mut self, #[callback] accounts: Option<Vec<AccountId>>, recipient_account_id: AccountId, recipient_contact: Contact, balance_to_withdraw: Balance) -> bool;
    fn on_withdraw_tip(&mut self, account_id: AccountId, contact: Contact, balance: Balance) -> bool;
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
    fn get_undefined_account() -> String {
        "".to_string()
    }

    fn get_linkdrop_contract() -> String {
        "near".to_string() // mainnet
        //"linkdrop.zavodil.testnet".to_string() // testnet
    }

    fn get_auth_contract() -> String {
        // TODO Set Auth contract
        "auth.name.near".to_string() // mainnet
        //"dev-1620499613958-3096267".to_string() // testnet
    }

    pub fn get_contact_owner(&self, contact: Contact, contract_address: AccountId) -> Promise {
        auth::get_owners(
            contact,
            &contract_address,
            NO_DEPOSIT,
            BASE_GAS)
    }

    pub fn on_get_contact_owner_on_tip_contact_to_deposit(&mut self,
                                                          #[callback] accounts: Vec<AccountId>,
                                                          sender_account_id: AccountId,
                                                          contact: Contact,
                                                          amount: Balance) {
        assert_eq!(
            env::predecessor_account_id(),
            env::current_account_id(),
            "Callback can only be called from the contract"
        );

        let owners_count = accounts.len();
        assert!(owners_count > 0, "Owner not found");
        assert!(owners_count <= 1, "Contact belongs to more then 1 account");

        let receiver_account_id: AccountId = accounts[0].clone();

        let sender_deposit: Balance = NearTips::get_deposit(self, sender_account_id.clone()).0;
        self.deposits.insert(sender_account_id.clone(), sender_deposit - amount);

        let receiver_deposit: Balance = NearTips::get_deposit(self, receiver_account_id.clone()).0;
        self.deposits.insert(receiver_account_id.clone(), receiver_deposit + amount);

        env::log(format!("@{} transferred {} yNEAR to deposit of @{} [{:?} account {:?}]",
                         sender_account_id, amount, receiver_account_id, contact.category, contact.value).as_bytes());
    }

    #[payable]
    // tip without knowing NEAR account id. telegram_handler = @username, not a numeric ID 123123123
    pub fn tip_contact_to_deposit(&mut self, telegram_handler: String, amount: WrappedBalance) -> Promise {
        assert!(amount.0 > 0, "Positive amount needed");

        let account_id = env::predecessor_account_id();
        let deposit: Balance = NearTips::get_deposit(self, account_id.clone()).0;

        let contact: Contact = Contact {
            category: ContactCategories::Telegram,
            value: telegram_handler,
        };

        assert!(
            amount.0 <= deposit,
            "Not enough tokens deposited to tip (Deposit: {}. Requested: {})",
            deposit, amount.0
        );

        self.get_contact_owner(contact.clone(), NearTips::get_auth_contract()).
            then(ext_self::on_get_contact_owner_on_tip_contact_to_deposit(
                account_id,
                contact,
                amount.0,
                &env::current_account_id(),
                NO_DEPOSIT,
                BASE_GAS,
            ))
    }

    #[payable]
    // tip without knowing NEAR account id
    pub fn tip_contact_with_attached_tokens(&mut self, contact: Contact) -> Promise {
        let deposit: Balance = near_sdk::env::attached_deposit();

        let account_id = env::predecessor_account_id();

        self.get_contact_owner(contact.clone(), NearTips::get_auth_contract()).
            then(ext_self::on_get_contact_owner_on_tip_contact_with_attached_tokens(
                account_id,
                contact,
                deposit,
                &env::current_account_id(),
                NO_DEPOSIT,
                BASE_GAS,
            ))
    }

    pub fn on_get_contact_owner_on_tip_contact_with_attached_tokens(&mut self,
                                                                    #[callback] accounts: Vec<AccountId>,
                                                                    sender_account_id: AccountId,
                                                                    contact: Contact,
                                                                    deposit: Balance) {
        assert_eq!(
            env::predecessor_account_id(),
            env::current_account_id(),
            "Callback can only be called from the contract"
        );

        let owners_count = accounts.len();

        assert!(owners_count <= 1, "Contact belongs to more then 1 account");
        assert!(owners_count > 0, "Owner not found");

        let receiver_account_id: AccountId = if owners_count == 0 { "".to_string() } else { accounts[0].clone() };

        NearTips::tip_transfer(self, sender_account_id, receiver_account_id, contact, deposit);
    }


    fn tip_transfer(&mut self,
                    sender_account_id: AccountId,
                    receiver_account_id: AccountId,
                    contact: Contact,
                    deposit: Balance) {
        match self.tips.get(&receiver_account_id) {
            Some(tips) => {
                let mut contact_found = false;
                let mut filtered_tips: Vec<_> =
                    tips
                        .iter()
                        .map(|tip| {
                            if tip.contact.value == contact.value && tip.contact.category == contact.category {
                                contact_found = true;
                                Tip {
                                    contact: contact.clone(),
                                    amount: tip.amount + deposit,
                                }
                            } else {
                                tip.clone()
                            }
                        })
                        .collect();

                env::log(format!("contact_found {}", contact_found).as_bytes());

                if contact_found {
                    self.tips.insert(receiver_account_id.clone(), filtered_tips);
                } else {
                    let tip: Tip = Tip {
                        contact: contact.clone(),
                        amount: deposit,
                    };
                    filtered_tips.push(tip);
                    self.tips.insert(receiver_account_id.clone(), filtered_tips);
                }
            }
            None => {
                let mut tips: Vec<Tip> = vec![];
                let tip: Tip = Tip {
                    contact: contact.clone(),
                    amount: deposit,
                };
                tips.push(tip);
                self.tips.insert(receiver_account_id.clone(), tips);
            }
        }

        env::log(format!("@{} tipped {} yNEAR to @{} [{:?} account {:?}]",
                         sender_account_id, deposit, receiver_account_id, contact.category, contact.value).as_bytes());
    }

    #[payable]
    // tip contact of existing NEAR account id
    pub fn tip_with_attached_tokens(&mut self, receiver_account_id: AccountId, contact: Contact) {
        let deposit: Balance = near_sdk::env::attached_deposit();

        let account_id = env::predecessor_account_id();

        NearTips::tip_transfer(self, account_id, receiver_account_id, contact, deposit);
    }


    pub fn get_tips(&self, account_id: AccountId) -> Option<Vec<Tip>> {
        match self.tips.get(&account_id) {
            Some(tips) => Some(tips.to_vec()),
            None => None
        }
    }


    pub fn get_tips_wrapped(&self, account_id: AccountId) -> Option<Vec<TipWrapped>> {
        match self.tips.get(&account_id) {
            Some(tips) => Some(tips
                .iter()
                .map(|tip| TipWrapped {
                    contact: tip.contact.clone(),
                    amount: WrappedBalance::from(tip.amount),
                })
                .collect::<Vec<TipWrapped>>()
                .to_vec()),
            None => None
        }
    }

    fn withdraw_tip_for_current_account(&self, contact: Contact, balance_to_withdraw: Balance) -> Promise {
        let account_id = env::predecessor_account_id();

        auth::get_contacts(account_id.clone(), &NearTips::get_auth_contract(), NO_DEPOSIT, BASE_GAS)
            .then(ext_self::on_get_contacts_on_withdraw_tip_for_current_account(
                account_id,
                contact,
                balance_to_withdraw,
                &env::current_account_id(),
                NO_DEPOSIT,
                2 * BASE_GAS,
            ))
    }

    fn withdraw_tip_for_undefined_account(&self, contact: Contact, balance_to_withdraw: Balance) -> Promise {
        let account_id = env::predecessor_account_id();

        self.get_contact_owner(contact.clone(), NearTips::get_auth_contract())
            .then(ext_self::on_get_contact_owner_on_withdraw_tip_for_undefined_account(
                account_id,
                contact,
                balance_to_withdraw,
                &env::current_account_id(),
                NO_DEPOSIT,
                2 * BASE_GAS,
            ))
    }


    pub fn on_get_contact_owner_on_withdraw_tip_for_undefined_account(&mut self,
                                                                      #[callback] accounts: Vec<AccountId>,
                                                                      recipient_account_id: AccountId,
                                                                      recipient_contact: Contact,
                                                                      balance_to_withdraw: Balance) -> Promise {
        assert_eq!(
            env::predecessor_account_id(),
            env::current_account_id(),
            "Callback can only be called from the contract"
        );

        let owners_count = accounts.len();

        assert!(owners_count <= 1, "Contact belongs to more then 1 account");
        assert!(owners_count > 0, "Owner not found");

        let undefined_account_id = NearTips::get_undefined_account();
        let contact_owner_account_id: AccountId = if owners_count == 0 { undefined_account_id.clone() } else { accounts[0].clone() };

        assert_eq!(
            contact_owner_account_id,
            recipient_account_id,
            "Current user not allowed to withdraw tip for this contact");

        env::log(format!("Transfer to @{} [{:?} account {:?}]", recipient_account_id, recipient_contact.category, recipient_contact.value).as_bytes());

        Promise::new(recipient_account_id)
            .transfer(balance_to_withdraw)
            .then(ext_self::on_withdraw_tip(
                undefined_account_id,
                recipient_contact,
                balance_to_withdraw,
                &env::current_account_id(),
                0,
                CALLBACK,
            ))
    }

    pub fn withdraw_tip(&mut self, contact: Contact) -> PromiseOrValue<bool> {
        // check tips sent exactly to this account
        let account_id = env::predecessor_account_id();
        let balance_of_account: Balance = NearTips::get_tip_by_contact(self, account_id.clone(), contact.clone()).0;

        // check tips sent exactly to contacts belongs to undefined account
        let undefined_account_id = NearTips::get_undefined_account();
        let balance_of_undefined_account: Balance = NearTips::get_tip_by_contact(self, undefined_account_id, contact.clone()).0;

        env::log(format!("balance_of_account  {} found", balance_of_account).as_bytes());
        env::log(format!("balance_of_undefined_account  {} found", balance_of_undefined_account).as_bytes());

        if balance_of_account > 0 && balance_of_undefined_account > 0 {
            env::log(format!("Tips for account & undefined account {} found", account_id).as_bytes());

            PromiseOrValue::Promise(
                NearTips::withdraw_tip_for_current_account(self, contact.clone(), balance_of_account)
                    .then(NearTips::withdraw_tip_for_undefined_account(self, contact, balance_of_undefined_account)))
        } else if balance_of_account > 0 {
            env::log(format!("Tips for account {} found", account_id).as_bytes());
            PromiseOrValue::Promise(
                NearTips::withdraw_tip_for_current_account(self, contact, balance_of_account))
        } else if balance_of_undefined_account > 0 {
            env::log("Tips for undefined account".to_string().as_bytes());
            PromiseOrValue::Promise(
                NearTips::withdraw_tip_for_undefined_account(self, contact, balance_of_undefined_account))
        } else {
            PromiseOrValue::Value(false)
        }
    }


    pub fn on_get_contacts_on_withdraw_tip_for_current_account(&mut self,
                                                               #[callback] contacts: Option<Vec<Contact>>,
                                                               recipient_account_id: AccountId,
                                                               recipient_contact: Contact,
                                                               balance: Balance) -> bool {
        assert_eq!(
            env::predecessor_account_id(),
            env::current_account_id(),
            "Callback can only be called from the contract"
        );

        match contacts {
            Some(contacts) => {
                for contact in &contacts {
                    env::log(format!("Check:  [{:?} account {:?}]", contact.category, contact.value).as_bytes());
                    if contact.value == recipient_contact.value && contact.category == recipient_contact.category {
                        Promise::new(recipient_account_id.clone())
                            .transfer(balance)
                            .then(ext_self::on_withdraw_tip(
                                recipient_account_id.clone(),
                                contact.clone(),
                                balance,
                                &env::current_account_id(),
                                0,
                                CALLBACK,
                            ));

                        env::log(format!("Transfer to {} [{:?} account {:?}]", recipient_account_id, contact.category, contact.value).as_bytes());

                        return true;
                    }
                }
            }
            None => {
                env::log("Contacts not found".to_string().as_bytes());
            }
        }

        false
    }

    pub fn on_withdraw_tip(&mut self, account_id: AccountId, contact: Contact, balance: Balance) -> bool {
        NearTips::assert_self();

        let transfer_succeeded = is_promise_success();
        if transfer_succeeded {
            match self.tips.get(&account_id) {
                Some(tips) => {
                    let mut contact_found = false;
                    let filtered_tips: Vec<_> =
                        tips
                            .iter()
                            .map(|tip| {
                                if tip.contact == contact {
                                    contact_found = true;
                                    Tip {
                                        contact: contact.clone(),
                                        amount: tip.amount - balance,
                                    }
                                } else {
                                    tip.clone()
                                }
                            })
                            .collect();

                    env::log(format!("on_withdraw_tip contact_found {}", contact_found).as_bytes());

                    if contact_found {
                        env::log(format!("Tip deducted for @{} by {} [{:?} account {:?}]", account_id, balance, contact.category, contact.value).as_bytes());
                        self.tips.insert(account_id.clone(), filtered_tips);
                        true
                    } else {
                        false
                    }
                }
                None => {
                    false
                }
            }
        } else {
            false
        }
    }

    pub fn get_tip_by_contact(&self, account_id: AccountId, contact: Contact) -> WrappedBalance {
        match self.tips.get(&account_id) {
            Some(tips) => {
                let filtered_tip: Vec<_> =
                    tips
                        .iter()
                        .filter(|tip| tip.contact == contact)
                        .collect();

                let tips_quantity = filtered_tip.len();

                if tips_quantity == 1 {
                    WrappedBalance::from(filtered_tip[0].amount)
                } else {
                    WrappedBalance::from(0)
                }
            }
            None => WrappedBalance::from(0)
        }
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
        assert!(amount.0 > 0, "Positive amount needed");

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
        Promise::new(account_id)
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
        assert!(balance > WITHDRAW_COMMISSION, "Not enough tokens to pay transfer commission");

        let deposit: Balance = NearTips::get_deposit(self, account_id.clone()).0;
        self.deposits.insert(account_id.clone(), deposit + balance - WITHDRAW_COMMISSION);

        self.telegram_tips.insert(telegram_account.clone(), 0);

        env::log(format!("@{} transfer {} yNEAR from telegram account {}. Transfer commission: {} yNEAR",
                         account_id, balance, telegram_account, WITHDRAW_COMMISSION).as_bytes());
    }

    pub fn assert_self() {
        assert_eq!(env::predecessor_account_id(), env::current_account_id());
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