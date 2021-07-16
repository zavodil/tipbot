use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::wee_alloc;
use near_sdk::{env, near_bindgen, AccountId, Balance, Promise, Gas, ext_contract, PromiseResult, PromiseOrValue, PanicOnDefault};
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::collections::{LookupSet, LookupMap, UnorderedMap};
use std::collections::HashMap;
use std::convert::TryFrom;

pub type WrappedBalance = U128;
pub type TelegramAccountId = u64;
pub type TelegramChatId = u64;
pub type RewardPoint = u16;

// TODO INIT Set telegram bot master account id
//const MASTER_ACCOUNT_ID: &str = "nearup_bot.app.near";
const MASTER_ACCOUNT_ID: &str = "zavodil.testnet";

const MIN_AMOUNT_TO_REWARD_CHAT: Balance = 100_000_000_000_000_000_000_000; // 0.1 NEAR

const WITHDRAW_COMMISSION: Balance = 3_000_000_000_000_000_000_000;
// 0.003 NEAR
const ACCESS_KEY_ALLOWANCE: Balance = 1_000_000_000_000_000_000_000_000;
const BASE_GAS: Gas = 25_000_000_000_000;
const CALLBACK_GAS: Gas = 25_000_000_000_000;
const NO_DEPOSIT: Balance = 0;

#[ext_contract(linkdrop)]
pub trait ExtLinkdrop {
    fn send(&self, public_key: String);
}

#[ext_contract(auth)]
pub trait ExtAuth {
    fn get_contacts(&self, account_id: AccountId) -> Option<Vec<Contact>>;
    fn get_account_for_contact(&self, contact: Contact) -> Vec<AccountId>;
}

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct NearTips {
    deposits: HashMap<AccountId, Balance>,
    telegram_tips: LookupMap<TelegramAccountId, Balance>,
    tips: LookupMap<AccountId, Vec<Tip>>,
    telegram_users_in_chats: LookupSet<TelegramUserInChat>,
    chat_points: LookupMap<TelegramChatId, RewardPoint>,
    version: u16,
    withdraw_available: bool,
    tip_available: bool,
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
    pub account_id: Option<TelegramAccountId>,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct TelegramUserInChat {
    pub telegram_id: TelegramAccountId,
    pub chat_id: TelegramChatId, // chat_id is negative, so don't forget * -1
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct ContactVer1 {
    pub category: ContactCategories,
    pub value: String,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct TipVer1 {
    pub contact: ContactVer1,
    pub amount: Balance,
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
    fn on_withdraw_from_telegram(&mut self, predecessor_account_id: AccountId, amount: Balance, telegram_account: TelegramAccountId) -> bool;
    fn on_withdraw_from_telegram_without_commission(&mut self, predecessor_account_id: AccountId, amount: Balance, telegram_account: TelegramAccountId) -> bool;
    fn on_withdraw(&mut self, predecessor_account_id: AccountId, deposit: Balance) -> bool;
    fn on_withdraw_linkdrop(&mut self, amount: Balance, telegram_account: TelegramAccountId, public_key: String) -> bool;
    fn on_get_contacts_on_withdraw_tip_for_current_account(&mut self, #[callback] contacts: Option<Vec<Contact>>, recipient_account_id: AccountId, recipient_contact: Contact, balance: Balance) -> bool;
    fn on_get_contact_owner_on_tip_contact_to_deposit(&mut self, #[callback] account: Option<AccountId>, sender_account_id: AccountId, contact: Contact, amount: Balance) -> bool;
    fn on_get_contact_owner_on_tip_contact_with_attached_tokens(&mut self, #[callback] account: Option<AccountId>, sender_account_id: AccountId, contact: Contact, deposit: Balance) -> bool;
    fn on_get_contact_owner_on_withdraw_tip_for_undefined_account(&mut self, #[callback] account: Option<AccountId>, recipient_account_id: AccountId, recipient_contact: Contact, balance_to_withdraw: Balance) -> bool;
    fn on_withdraw_tip(&mut self, account_id: AccountId, contact: Contact, balance: Balance) -> bool;
    fn on_get_contact_owner_on_withdraw_from_telegram_with_auth(&mut self, #[callback] account: Option<AccountId>, recipient_account_id: AccountId, contact: Contact) -> bool;
    fn on_get_contact_owner_on_send_tip_to_telegram_with_auth(&mut self, #[callback] account: Option<AccountId>, sender_account_id: AccountId, sender_deposit: Balance, tip_amount: Balance, telegram_account: TelegramAccountId, chat_id: Option<TelegramChatId>) -> bool;
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

/// Helper structure to for keys of the persistent collections.
#[derive(BorshSerialize)]
pub enum StorageKey {
    Tips,
    TipsLookupMap,
    ChatPointsLookupMap,
    TelegramTips,
    TelegramUsersInChats,
}

#[near_bindgen]
impl NearTips {
    fn get_undefined_account() -> String {
        "".to_string()
    }

    fn get_linkdrop_contract() -> String {
        // TODO INIT
        //"near".to_string() // mainnet
        "linkdrop.zavodil.testnet".to_string() // testnet
    }

    fn get_auth_contract() -> String {
        // TODO INIT
        //"auth.name.near".to_string() // mainnet
        "dev-1625611642901-32969379055293".to_string() // testnet
    }

    #[init]
    pub fn new() -> Self {
        Self {
            deposits: HashMap::new(),
            telegram_tips: LookupMap::new(StorageKey::TelegramTips.try_to_vec().unwrap()), // first object only for telegram tips
            tips: LookupMap::new(StorageKey::TipsLookupMap.try_to_vec().unwrap()), // generic object for any tips
            telegram_users_in_chats: LookupSet::new(StorageKey::TelegramUsersInChats.try_to_vec().unwrap()),
            chat_points: LookupMap::new(StorageKey::ChatPointsLookupMap.try_to_vec().unwrap()),
            version: 0,
            withdraw_available: true,
            tip_available: true,
        }
    }

    #[payable]
    pub fn deposit(&mut self) {
        assert!(self.withdraw_available, "Deposits/Withdrawals paused");
        let account_id: AccountId = env::predecessor_account_id();
        let attached_deposit: Balance = env::attached_deposit();
        let deposit: Balance = NearTips::get_deposit(self, account_id.clone()).0;
        self.deposits.insert(account_id, deposit + attached_deposit);
    }

    #[payable]
    pub fn deposit_to_account(&mut self, account_id: ValidAccountId) {
        assert!(self.withdraw_available, "Deposits/Withdrawals paused");
        let attached_deposit: Balance = env::attached_deposit();
        self.deposit_amount_to_account(account_id, attached_deposit);
    }

    pub(crate) fn deposit_amount_to_account(&mut self, account_id: ValidAccountId, amount: Balance) {
        assert!(self.withdraw_available, "Deposits/Withdrawals paused");

        let account_id_prepared: AccountId = account_id.into();
        let deposit: Balance = NearTips::get_deposit(self, account_id_prepared.clone()).0;
        self.deposits.insert(account_id_prepared, deposit + amount);
    }

    pub fn get_deposit(&self, account_id: AccountId) -> WrappedBalance {
        match self.deposits.get(&account_id) {
            Some(deposit) => WrappedBalance::from(*deposit),
            None => WrappedBalance::from(0)
        }
    }

    /* INITIAL FUNCTIONS, using telegram_tips object */

    pub fn get_balance(&self, telegram_account: TelegramAccountId) -> WrappedBalance {
        match self.telegram_tips.get(&telegram_account) {
            Some(balance) => WrappedBalance::from(balance),
            None => WrappedBalance::from(0)
        }
    }

    pub fn claim_chat_points(&mut self) {
        // TODO
    }

    pub fn get_chat_score(&self, chat_id: TelegramChatId) -> RewardPoint {
        self.chat_points.get(&chat_id).unwrap_or(0)
    }

    pub fn get_telegram_users_in_chats(&self, telegram_id: TelegramAccountId,
                                       chat_id: TelegramChatId) -> bool {
        let user_in_chat: TelegramUserInChat = TelegramUserInChat {
            telegram_id,
            chat_id
        };

        self.telegram_users_in_chats.contains(&user_in_chat)
    }

    pub(crate) fn send_tip_to_telegram_from_account(&mut self,
                                                    sender_account_id: AccountId,
                                                    telegram_account: TelegramAccountId,
                                                    amount: WrappedBalance,
                                                    chat_id: Option<TelegramChatId>) {
        assert!(self.tip_available, "Tips paused");
        assert!(amount.0 > 0, "Positive amount needed");

        let deposit: Balance = NearTips::get_deposit(self, sender_account_id.clone()).0;

        assert!(
            amount.0 <= deposit,
            "Not enough tokens deposited to tip (Deposit: {}. Requested: {})",
            deposit, amount.0
        );

        let balance: Balance = NearTips::get_balance(self, telegram_account.clone()).0;
        self.telegram_tips.insert(&telegram_account, &(balance + amount.0));

        self.deposits.insert(sender_account_id.clone(), deposit - amount.0);

        if amount.0 > MIN_AMOUNT_TO_REWARD_CHAT && !chat_id.is_none() {
            let chat_id_value = chat_id.unwrap();
            let user_in_chat: TelegramUserInChat = TelegramUserInChat {
                telegram_id: telegram_account,
                chat_id: chat_id_value,
            };

            if !self.telegram_users_in_chats.contains(&user_in_chat) {
                let chat_score: RewardPoint = self.chat_points.get(&chat_id_value).unwrap_or(0);
                let new_score = chat_score + 1;
                self.chat_points.insert(&chat_id_value, &new_score);
                self.telegram_users_in_chats.insert(&user_in_chat);

                env::log(format!("Reward point for chat {} added. Total: {}", chat_id_value, new_score).as_bytes());
            }
        }

        env::log(format!("@{} tipped {} yNEAR for telegram account {}", sender_account_id, amount.0, telegram_account).as_bytes());
    }

    pub fn send_tip_to_telegram(&mut self,
                                telegram_account: TelegramAccountId,
                                amount: WrappedBalance,
                                chat_id: Option<TelegramChatId>) {
        let account_id = env::predecessor_account_id();
        self.send_tip_to_telegram_from_account(account_id, telegram_account, amount, chat_id);
    }

    pub fn send_tip_to_telegram_with_auth(&mut self,
                                          telegram_account: TelegramAccountId,
                                          amount: WrappedBalance,
                                          chat_id: Option<TelegramChatId>) -> Promise {
        assert!(self.tip_available, "Tips paused");
        assert!(amount.0 > 0, "Positive amount needed");

        let account_id = env::predecessor_account_id();
        let deposit: Balance = NearTips::get_deposit(self, account_id.clone()).0;

        let contact: Contact = Contact {
            category: ContactCategories::Telegram,
            value: "".to_string(),
            account_id: Some(telegram_account.clone()),
        };

        assert!(
            amount.0 <= deposit,
            "Not enough tokens deposited to tip (Deposit: {}. Requested: {})",
            deposit, amount.0
        );

        self.get_contact_owner(contact.clone(), NearTips::get_auth_contract()).
            then(ext_self::on_get_contact_owner_on_send_tip_to_telegram_with_auth(
                account_id,
                deposit,
                amount.0,
                telegram_account,
                chat_id,
                &env::current_account_id(),
                NO_DEPOSIT,
                CALLBACK_GAS * 2,
            ))
    }

    pub fn on_get_contact_owner_on_send_tip_to_telegram_with_auth(&mut self,
                                                                  #[callback] account: Option<AccountId>,
                                                                  sender_account_id: AccountId,
                                                                  sender_deposit: Balance,
                                                                  tip_amount: Balance,
                                                                  telegram_account: TelegramAccountId,
                                                                  chat_id: Option<TelegramChatId>) {
        assert_eq!(
            env::predecessor_account_id(),
            env::current_account_id(),
            "Callback can only be called from the contract"
        );

        match account {
            Some(account_id) => {
                let valid_account_id = ValidAccountId::try_from(account_id.clone()).unwrap();
                self.deposit_amount_to_account(valid_account_id, tip_amount);
                self.deposits.insert(sender_account_id.clone(), sender_deposit - tip_amount);
                env::log(format!("@{} tipped {} yNEAR to NEAR account {} telegram account {}", sender_account_id, tip_amount, account_id, telegram_account).as_bytes());
            }
            None => {
                env::log(format!("Authorized contact wasn't found for telegram {}. Continue to send from @{}", telegram_account, sender_account_id).as_bytes());
                self.send_tip_to_telegram_from_account(sender_account_id, telegram_account, U128::from(tip_amount), chat_id);
            }
        }
    }

    // centralized tips withdraw, with master_account authorisation
    pub fn withdraw_from_telegram(&mut self, telegram_account: TelegramAccountId, account_id: AccountId) -> Promise {
        assert!(self.withdraw_available, "Withdrawals paused");
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
                CALLBACK_GAS,
            ))
    }

    pub fn on_withdraw_from_telegram(&mut self,
                                     predecessor_account_id: AccountId,
                                     amount: Balance,
                                     telegram_account: TelegramAccountId) -> bool {
        assert_eq!(
            env::predecessor_account_id(),
            env::current_account_id(),
            "Callback can only be called from the contract"
        );
        let withdraw_succeeded = is_promise_success();
        if withdraw_succeeded {
            self.telegram_tips.insert(&telegram_account.clone(), &0);
            Promise::new(MASTER_ACCOUNT_ID.to_string()).transfer(WITHDRAW_COMMISSION);

            env::log(format!("@{} withdrew {} yNEAR from telegram account {}. Withdraw commission: {} yNEAR",
                             predecessor_account_id, amount, telegram_account, WITHDRAW_COMMISSION).as_bytes());
        }

        withdraw_succeeded
    }

    #[payable]
    // decentralized tips withdraw for those who made near auth. telegram_account is numeric ID 123123123
    pub fn withdraw_from_telegram_with_auth(&mut self, telegram_account: TelegramAccountId) -> Promise {
        assert!(self.withdraw_available, "Withdrawals paused");

        let account_id = env::predecessor_account_id();

        let contact: Contact = Contact {
            category: ContactCategories::Telegram,
            value: "".to_string(),
            account_id: Some(telegram_account),
        };

        self.get_contact_owner(contact.clone(), NearTips::get_auth_contract()).
            then(ext_self::on_get_contact_owner_on_withdraw_from_telegram_with_auth(
                account_id,
                contact,
                &env::current_account_id(),
                NO_DEPOSIT,
                CALLBACK_GAS * 3,
            ))
    }

    pub fn on_get_contact_owner_on_withdraw_from_telegram_with_auth(&mut self,
                                                                    #[callback] account: Option<AccountId>,
                                                                    recipient_account_id: AccountId,
                                                                    contact: Contact) -> Promise {
        assert_eq!(
            env::predecessor_account_id(),
            env::current_account_id(),
            "Callback can only be called from the contract"
        );

        match account {
            Some(account) => {
                assert!(account == recipient_account_id, "Not authorized to withdraw");
                assert!(!contact.account_id.is_none(), "Account ID is missing");

                let balance: Balance = NearTips::get_balance(self, contact.account_id.unwrap()).0;
                assert!(balance > 0, "Not enough tokens to withdraw");

                Promise::new(recipient_account_id)
                    .transfer(balance)
                    .then(ext_self::on_withdraw_from_telegram_without_commission(
                        env::predecessor_account_id(),
                        balance,
                        contact.account_id.unwrap(),
                        &env::current_account_id(),
                        NO_DEPOSIT,
                        CALLBACK_GAS,
                    ))
            }
            None => {
                panic!("Contact wasn't authorized to any account");
            }
        }
    }


    pub fn on_withdraw_from_telegram_without_commission(&mut self,
                                                        predecessor_account_id: AccountId,
                                                        amount: Balance,
                                                        telegram_account: TelegramAccountId) -> bool {
        assert_eq!(
            env::predecessor_account_id(),
            env::current_account_id(),
            "Callback can only be called from the contract"
        );
        let withdraw_succeeded = is_promise_success();
        if withdraw_succeeded {
            self.telegram_tips.insert(&telegram_account.clone(), &0);

            env::log(format!("@{} withdrew {} yNEAR from telegram account {}",
                             predecessor_account_id, amount, telegram_account).as_bytes());
        }

        withdraw_succeeded
    }

    // withdraw from deposit
    pub fn withdraw(&mut self) -> Promise {
        assert!(self.withdraw_available, "Withdrawals paused");
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
                CALLBACK_GAS,
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

    pub fn withdraw_linkdrop(&mut self, public_key: String, telegram_account: TelegramAccountId) -> Promise {
        assert!(self.withdraw_available, "Withdrawals paused");
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
                CALLBACK_GAS,
            ))
    }

    pub fn on_withdraw_linkdrop(&mut self, amount: Balance, telegram_account: TelegramAccountId, public_key: String) -> bool {
        assert_eq!(
            env::predecessor_account_id(),
            env::current_account_id(),
            "Callback can only be called from the contract"
        );
        let withdraw_succeeded = is_promise_success();
        if withdraw_succeeded {
            self.telegram_tips.insert(&telegram_account.clone(), &0);
            Promise::new(MASTER_ACCOUNT_ID.to_string()).transfer(WITHDRAW_COMMISSION);

            env::log(format!("Telegram account {} withdrew {} yNEAR with linkDrop for public key {}. Withdraw commission: {} yNEAR",
                             telegram_account, amount, public_key, WITHDRAW_COMMISSION).as_bytes());
        }

        withdraw_succeeded
    }

    pub fn get_contact_owner(&self, contact: Contact, contract_address: AccountId) -> Promise {
        auth::get_account_for_contact(
            contact,
            &contract_address,
            NO_DEPOSIT,
            BASE_GAS)
    }

    #[payable]
    // tip from balance to near account deposit without knowing NEAR account_id. telegram_account is numeric ID 123123123
    pub fn tip_contact_to_deposit(&mut self, telegram_account: TelegramAccountId, amount: WrappedBalance) -> Promise {
        assert!(self.tip_available, "Tips paused");
        assert!(amount.0 > 0, "Positive amount needed");

        let account_id = env::predecessor_account_id();
        let deposit: Balance = NearTips::get_deposit(self, account_id.clone()).0;

        let contact: Contact = Contact {
            category: ContactCategories::Telegram,
            value: "".to_string(),
            account_id: Some(telegram_account),
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
                CALLBACK_GAS,
            ))
    }

    pub fn on_get_contact_owner_on_tip_contact_to_deposit(&mut self,
                                                          #[callback] account: Option<AccountId>,
                                                          sender_account_id: AccountId,
                                                          contact: Contact,
                                                          amount: Balance) {
        assert_eq!(
            env::predecessor_account_id(),
            env::current_account_id(),
            "Callback can only be called from the contract"
        );

        assert!(!account.is_none(), "Owner not found");
        let receiver_account_id: AccountId = account.unwrap();

        let sender_deposit: Balance = NearTips::get_deposit(self, sender_account_id.clone()).0;
        self.deposits.insert(sender_account_id.clone(), sender_deposit - amount);

        let receiver_deposit: Balance = NearTips::get_deposit(self, receiver_account_id.clone()).0;
        self.deposits.insert(receiver_account_id.clone(), receiver_deposit + amount);

        env::log(format!("@{} transferred {} yNEAR to deposit of @{} [{:?} account {:?}]",
                         sender_account_id, amount, receiver_account_id, contact.category, contact.value).as_bytes());
    }

    #[payable]
    // tip attached tokens without knowing NEAR account id
    pub fn tip_contact_with_attached_tokens(&mut self, contact: Contact) -> Promise {
        assert!(self.tip_available, "Tips paused");
        let deposit: Balance = near_sdk::env::attached_deposit();

        let account_id = env::predecessor_account_id();

        self.get_contact_owner(contact.clone(), NearTips::get_auth_contract()).
            then(ext_self::on_get_contact_owner_on_tip_contact_with_attached_tokens(
                account_id,
                contact,
                deposit,
                &env::current_account_id(),
                NO_DEPOSIT,
                CALLBACK_GAS,
            ))
    }

    pub fn on_get_contact_owner_on_tip_contact_with_attached_tokens(&mut self,
                                                                    #[callback] account: Option<AccountId>,
                                                                    sender_account_id: AccountId,
                                                                    contact: Contact,
                                                                    deposit: Balance) {
        assert_eq!(
            env::predecessor_account_id(),
            env::current_account_id(),
            "Callback can only be called from the contract"
        );

        assert!(!account.is_none(), "Owner not found");
        let receiver_account_id: AccountId = account.unwrap();

        NearTips::tip_transfer(self, sender_account_id, receiver_account_id, contact, deposit);
    }

    /* GENERIC TIPS, using tips object */
    pub(crate) fn tip_transfer(&mut self,
                               sender_account_id: AccountId,
                               receiver_account_id: AccountId,
                               contact: Contact,
                               deposit: Balance) {
        assert!(self.tip_available, "Tips paused");
        match self.tips.get(&receiver_account_id) {
            Some(tips) => {
                let mut contact_found = false;
                let mut filtered_tips: Vec<_> =
                    tips
                        .iter()
                        .map(|tip| {
                            if NearTips::are_contacts_equal(tip.contact.clone(), contact.clone()) {
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

                if !contact_found {
                    let tip: Tip = Tip {
                        contact: contact.clone(),
                        amount: deposit,
                    };
                    filtered_tips.push(tip);
                }

                self.tips.insert(&receiver_account_id.clone(), &filtered_tips);
            }
            None => {
                let mut tips: Vec<Tip> = vec![];
                let tip: Tip = Tip {
                    contact: contact.clone(),
                    amount: deposit,
                };
                tips.push(tip);
                self.tips.insert(&receiver_account_id.clone(), &tips);
            }
        }

        env::log(format!("@{} tipped {} yNEAR to @{} [{:?} account {:?}]",
                         sender_account_id, deposit, receiver_account_id, contact.category, contact.value).as_bytes());
    }

    #[payable]
    // tip contact of existing NEAR account_id
    pub fn tip_with_attached_tokens(&mut self, receiver_account_id: AccountId, contact: Contact) {
        assert!(self.tip_available, "Tips paused");
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

    // we can tip contact which doesn't have near account_id yet
    fn withdraw_tip_for_undefined_account(&self, contact: Contact, balance_to_withdraw: Balance) -> Promise {
        let account_id = env::predecessor_account_id();

        self.get_contact_owner(contact.clone(), NearTips::get_auth_contract())
            .then(ext_self::on_get_contact_owner_on_withdraw_tip_for_undefined_account(
                account_id,
                contact,
                balance_to_withdraw,
                &env::current_account_id(),
                NO_DEPOSIT,
                CALLBACK_GAS * 3,
            ))
    }


    pub fn on_get_contact_owner_on_withdraw_tip_for_undefined_account(&mut self,
                                                                      #[callback] account: Option<AccountId>,
                                                                      recipient_account_id: AccountId,
                                                                      recipient_contact: Contact,
                                                                      balance_to_withdraw: Balance) -> Promise {
        assert_eq!(
            env::predecessor_account_id(),
            env::current_account_id(),
            "Callback can only be called from the contract"
        );

        assert!(!account.is_none(), "Owner not found");
        let contact_owner_account_id: AccountId = account.unwrap();
        let undefined_account_id = NearTips::get_undefined_account();

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
                CALLBACK_GAS,
            ))
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
                CALLBACK_GAS * 3,
            ))
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
                    //if contact.value == recipient_contact.value && contact.category == recipient_contact.category {
                    if NearTips::are_contacts_equal(contact.clone(), recipient_contact.clone()) {
                        Promise::new(recipient_account_id.clone())
                            .transfer(balance)
                            .then(ext_self::on_withdraw_tip(
                                recipient_account_id.clone(),
                                contact.clone(),
                                balance,
                                &env::current_account_id(),
                                0,
                                CALLBACK_GAS,
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

    pub fn withdraw_tip(&mut self, contact: Contact) -> PromiseOrValue<bool> {
        assert!(self.withdraw_available, "Withdrawals paused");
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
                        self.tips.insert(&account_id.clone(), &filtered_tips);
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

    pub(crate) fn are_contacts_equal(contact1: Contact, contact2: Contact) -> bool {
        if contact1.category == ContactCategories::Telegram && contact2.category == ContactCategories::Telegram {
            contact1.account_id == contact2.account_id
        } else {
            contact1.category == contact2.category && contact1.value == contact2.value
        }
    }

    pub fn get_tip_by_contact(&self, account_id: AccountId, contact: Contact) -> WrappedBalance {
        match self.tips.get(&account_id) {
            Some(tips) => {
                let filtered_tip: Vec<_> =
                    tips
                        .iter()
                        .filter(|tip| NearTips::are_contacts_equal(tip.contact.clone(), contact.clone()))
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


    pub fn set_withdraw_available(&mut self, withdraw_available: bool) {
        assert!(env::predecessor_account_id() == MASTER_ACCOUNT_ID, "No access");
        self.withdraw_available = withdraw_available;
    }

    pub fn get_withdraw_available(self) -> bool {
        self.withdraw_available
    }

    pub fn set_tip_available(&mut self, tip_available: bool) {
        assert!(env::predecessor_account_id() == MASTER_ACCOUNT_ID, "No access");
        self.tip_available = tip_available;
    }

    pub fn get_tip_available(self) -> bool {
        self.tip_available
    }

    pub fn transfer_tips_to_deposit(&mut self, telegram_account: TelegramAccountId, account_id: AccountId) {
        assert!(self.withdraw_available, "Withdrawals paused");

        assert!(env::predecessor_account_id() == MASTER_ACCOUNT_ID, "No access");
        assert!(env::is_valid_account_id(account_id.as_bytes()), "Account @{} is invalid", account_id);

        let balance: Balance = NearTips::get_balance(self, telegram_account.clone()).0;
        assert!(balance > WITHDRAW_COMMISSION, "Not enough tokens to pay transfer commission");

        let deposit: Balance = NearTips::get_deposit(self, account_id.clone()).0;
        self.deposits.insert(account_id.clone(), deposit + balance - WITHDRAW_COMMISSION);

        self.telegram_tips.insert(&telegram_account.clone(), &0);

        env::log(format!("@{} transfer {} yNEAR from telegram account {}. Transfer commission: {} yNEAR",
                         account_id, balance, telegram_account, WITHDRAW_COMMISSION).as_bytes());
    }

    pub fn assert_self() {
        assert_eq!(env::predecessor_account_id(), env::current_account_id());
    }

    pub fn get_master_account_id() -> String {
        MASTER_ACCOUNT_ID.to_string()
    }

    pub fn get_version(&self) -> u16 {
        self.version
    }

    pub fn get_deposits(&self) -> HashMap<AccountId, U128> {
        self.deposits
            .iter()
            .map(|(account_id, balance)| (account_id.clone(), (*balance).into()))
            .collect()
    }

    /*  not available with LookupMap
    pub fn get_telegram_tips(&self, from_index: u64, limit: u64) -> HashMap<TelegramAccountId, U128> {
        let keys = self.telegram_tips.keys_as_vector();

        (from_index..std::cmp::min(from_index + limit, keys.len()))
            .map(|index| {
                let account_id = keys.get(index).unwrap();
                let amount = self.get_balance(account_id.clone());
                (account_id, amount)
            })
            .collect()
    }

    pub fn get_all_tips(&self, from_index: u64, limit: u64) -> HashMap<AccountId, Option<Vec<Tip>>> {
        let keys = self.tips.keys_as_vector();

        (from_index..std::cmp::min(from_index + limit, keys.len()))
            .map(|index| {
                let account_id = keys.get(index).unwrap();
                let tips = self.get_tips(account_id.clone());
                (account_id, tips)
            })
            .collect()
    }
    */

    /*
    #[init(ignore_state)]
    pub fn migrate_state_1() -> Self {
        let migration_version: u16 = 1;
        assert_eq!(env::predecessor_account_id(), env::current_account_id(), "Private function");

        #[derive(BorshDeserialize)]
        struct OldContract {
            deposits: HashMap<AccountId, Balance>,
            telegram_tips: HashMap<String, Balance>,
        }

        let old_contract: OldContract = env::state_read().expect("Old state doesn't exist");

        Self {
            deposits: old_contract.deposits,
            telegram_tips: old_contract.telegram_tips,
            tips: HashMap::new(),
            version: migration_version,
            withdraw_available: true,
            tip_available: true,
        }
    }*/

    /*
    #[init(ignore_state)]
    pub fn migrate_state_2() -> Self {
        let migration_version: u16 = 2;
        assert_eq!(env::predecessor_account_id(), env::current_account_id(), "Private function");

        #[derive(BorshDeserialize)]
        struct OldContract {
            deposits: HashMap<AccountId, Balance>,
            telegram_tips: HashMap<String, Balance>,
            tips: HashMap<AccountId, Vec<TipVer1>>,
            version: u16,
            withdraw_available: bool,
            tip_available: bool,
        }

        let old_contract: OldContract = env::state_read().expect("Old state doesn't exist");

        let new_tips = UnorderedMap::new(StorageKey::Tips.try_to_vec().unwrap());

        Self {
            deposits: old_contract.deposits,
            telegram_tips: old_contract.telegram_tips,
            tips: new_tips,
            version: migration_version,
            withdraw_available: old_contract.withdraw_available,
            tip_available: old_contract.tip_available,
        }
    }
    */


    #[init(ignore_state)]
    #[allow(dead_code)]
    pub fn migrate_state_3() -> Self { // add telegram_users_in_chats, Migration to
        let migration_version: u16 = 3;
        assert_eq!(env::predecessor_account_id(), env::current_account_id(), "Private function");

        #[derive(BorshDeserialize)]
        struct OldContract {
            deposits: HashMap<AccountId, Balance>,
            telegram_tips: HashMap<String, Balance>,
            tips: UnorderedMap<AccountId, Vec<Tip>>,
            version: u16,
            withdraw_available: bool,
            tip_available: bool,
        }

        let old_contract: OldContract = env::state_read().expect("Old state doesn't exist");
        let telegram_users_in_chats = LookupSet::new(StorageKey::TelegramUsersInChats.try_to_vec().unwrap());
        let tips_new = LookupMap::new(StorageKey::TipsLookupMap.try_to_vec().unwrap());
        let chat_points = LookupMap::new(StorageKey::ChatPointsLookupMap.try_to_vec().unwrap());

        let mut telegram_tips_new = LookupMap::new(StorageKey::TelegramTips.try_to_vec().unwrap());

        for (telegram_account, amount) in &old_contract.telegram_tips {
            let telegram_id = telegram_account.parse::<u64>().unwrap_or(0);
            if telegram_id > 0 {
                telegram_tips_new.insert(&telegram_id, amount);
            } else {
                env::log(format!("Invalid telegram_account {}", telegram_account).as_bytes());
            }
        }

        Self {
            deposits: old_contract.deposits,
            telegram_tips: telegram_tips_new,
            tips: tips_new,
            telegram_users_in_chats,
            chat_points,
            version: migration_version,
            withdraw_available: old_contract.withdraw_available,
            tip_available: old_contract.tip_available,
        }
    }
    /*
        #[init(ignore_state)]
        #[allow(dead_code)]
        pub fn migrate_state_4() -> Self { // telegram_tips migration
            let migration_version: u16 = 4;
            assert_eq!(env::predecessor_account_id(), env::current_account_id(), "Private function");

            #[derive(BorshDeserialize)]
            struct OldContract {
                deposits: HashMap<AccountId, Balance>,
                telegram_tips: HashMap<String, Balance>,
                tips: UnorderedMap<AccountId, Vec<Tip>>,
                telegram_users_in_chats: LookupSet<TelegramUserInChat>,
                version: u16,
                withdraw_available: bool,
                tip_available: bool,
            }

            let old_contract: OldContract = env::state_read().expect("Old state doesn't exist");

            let mut telegram_tips_new = UnorderedMap::new(StorageKey::TelegramTips.try_to_vec().unwrap());

            for (telegram_account, amount) in &old_contract.telegram_tips {
                let telegram_id = telegram_account.parse::<u64>().unwrap_or(0);
                /*let telegram_id: TelegramAccountId = match telegram_account.parse::<u64>().unwrap() {
                    Ok(telegram_id) => telegram_tips_new.insert(&telegram_id, amount),
                    Err(e) => 0
                }*/
                if telegram_id > 0 {
                    telegram_tips_new.insert(&telegram_id, amount);
                } else {
                    env::log(format!("Invalid telegram_account {}", telegram_account).as_bytes());
                }
            }

            Self {
                deposits: old_contract.deposits,
                telegram_tips: telegram_tips_new,
                tips: old_contract.tips,
                telegram_users_in_chats: old_contract.telegram_users_in_chats,
                version: migration_version,
                withdraw_available: old_contract.withdraw_available,
                tip_available: old_contract.tip_available,
            }
        }*/
}

#[cfg(test)]
mod tests {
    // outdated. use jest simulation tests instead.
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