use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::wee_alloc;
use near_sdk::{env, near_bindgen, AccountId, Balance, Promise, Gas, ext_contract, PromiseResult, PromiseOrValue, PanicOnDefault, BorshStorageKey, log};
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::collections::{LookupSet, LookupMap};
use std::collections::HashMap;
use std::convert::TryFrom;  
use near_contract_standards::fungible_token::core_impl::ext_fungible_token;

pub type WrappedBalance = U128;
pub type TelegramAccountId = u64; // id may potentially overflow the u64 limit 
pub type TelegramChatId = u64;
pub type RewardPoint = u128;
pub type TokenAccountId = AccountId;
pub type TreasureFeeNumerator = u128; // u128 to avoid additional castings

// UPDATE BEFORE DEPLOYMENT
/*
const MASTER_ACCOUNT_ID: &str = "nearup_bot.app.near";
const LINKDROP_ACCOUNT_ID: &str = "near";
const AUTH_ACCOUNT_ID: &str = "auth.name.near";
const TREASURE_ACCOUNT_ID: &str =
*/

const MASTER_ACCOUNT_ID: &str = "zavodil.testnet";
const LINKDROP_ACCOUNT_ID: &str = "linkdrop.zavodil.testnet";
const AUTH_ACCOUNT_ID: &str = "dev-1625611642901-32969379055293";
// const TREASURE_ACCOUNT_ID: &str = "treasure.zavodil.testnet";

const UNDEFINED_ACCOUNT_ID: &str = "";

// 0.1 NEAR
const MIN_AMOUNT_TO_REWARD_CHAT: Balance = 100_000_000_000_000_000_000_000;
const MIN_DEPOSIT_NEAR: Balance = 100_000_000_000_000_000_000_000;
const MIN_DEPOSIT_FT: Balance = 100_000_000_000_000_000;

// 0.003 NEAR
const WITHDRAW_COMMISSION: Balance = 3_000_000_000_000_000_000_000;

const ACCESS_KEY_ALLOWANCE: Balance = 1_000_000_000_000_000_000_000_000;
const BASE_GAS: Gas = 25_000_000_000_000;
const CALLBACK_GAS: Gas = 25_000_000_000_000;
const GAS_FOR_FT_TRANSFER: Gas = 10_000_000_000_000;
const GAS_FOR_AFTER_FT_TRANSFER: Gas = 10_000_000_000_000;
const NO_DEPOSIT: Balance = 0;
const ONE_YOCTO: Balance = 1;
const NEAR: &str = "near";

uint::construct_uint! {
    pub struct U256(4);
}

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
    deposits: LookupMap<TokenByNearAccount, Balance>,
    telegram_tips: LookupMap<TokenByTelegramAccount, Balance>,
    tips: LookupMap<AccountId, Vec<Tip>>, // should be removed, bad approach
    telegram_users_in_chats: LookupSet<TelegramUserInChat>,
    chat_points: LookupMap<TokenByTelegramChat, RewardPoint>,
    whitelisted_tokens: LookupSet<TokenAccountId>,
    version: u16,
    withdraw_available: bool,
    tip_available: bool,
    generic_tips_available: bool,

    // empty, used for migration
    telegram_tips_v1: HashMap<String, Balance>,

    total_chat_points: RewardPoint, // not used, TODO for each token?
    chat_settings: LookupMap<TelegramChatId, ChatSettings>,
    treasure: LookupMap<TokenAccountId, Balance>
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
pub struct TokenByNearAccount {
    pub account_id: AccountId,
    pub token_account_id: TokenAccountId,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct TokenByTelegramAccount {
    pub telegram_account: TelegramAccountId,
    pub token_account_id: TokenAccountId,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct TokenByTelegramChat {
    pub chat_id: TelegramChatId,
    pub token_account_id: TokenAccountId,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct TelegramUserInChat {
    pub telegram_id: TelegramAccountId,
    pub chat_id: TelegramChatId, // chat_id is negative, so don't forget * -1
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct ChatSettings {
    pub admin_account_id: AccountId,
    pub treasure_fee_numerator: TreasureFeeNumerator,
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
    fn on_withdraw(&mut self, predecessor_account_id: AccountId, deposit: Balance, token_id: Option<TokenAccountId>) -> bool;
    fn on_withdraw_linkdrop(&mut self, amount: Balance, telegram_account: TelegramAccountId, public_key: String) -> bool;
    fn on_get_contacts_on_withdraw_tip_for_current_account(&mut self, #[callback] contacts: Option<Vec<Contact>>, recipient_account_id: AccountId, recipient_contact: Contact, balance: Balance) -> bool;
    fn on_get_contact_owner_on_tip_contact_to_deposit(&mut self, #[callback] account: Option<AccountId>, sender_account_id: AccountId, contact: Contact, amount: Balance, token_id: Option<TokenAccountId>) -> bool;
    fn on_get_contact_owner_on_tip_contact_with_attached_tokens(&mut self, #[callback] account: Option<AccountId>, sender_account_id: AccountId, contact: Contact, deposit: Balance) -> bool;
    fn on_get_contact_owner_on_withdraw_tip_for_undefined_account(&mut self, #[callback] account: Option<AccountId>, recipient_account_id: AccountId, recipient_contact: Contact, balance_to_withdraw: Balance) -> bool;
    fn on_withdraw_tip(&mut self, account_id: AccountId, contact: Contact, balance: Balance) -> bool;
    fn on_get_contact_owner_on_withdraw_from_telegram_with_auth(&mut self, #[callback] account: Option<AccountId>, recipient_account_id: AccountId, contact: Contact, token_id: Option<TokenAccountId>) -> bool;
    fn on_get_contact_owner_on_send_tip_to_telegram_with_auth(&mut self, #[callback] account: Option<AccountId>, sender_account_id: AccountId, tip_amount: Balance, telegram_account: TelegramAccountId, chat_id: Option<TelegramChatId>, token_id: Option<TokenAccountId>) -> bool;

    fn after_ft_transfer_balance(&mut self, telegram_account: TelegramAccountId, amount: WrappedBalance, token_account_id: TokenAccountId) -> bool;
    fn after_ft_transfer_deposit(&mut self, account_id: AccountId, amount: WrappedBalance, token_account_id: TokenAccountId) -> bool;
    fn after_ft_transfer_to_treasure(&mut self, chat_id: TelegramChatId, treasure_fee: WrappedBalance, token_account_id: TokenAccountId) -> bool;
    fn after_ft_transfer_claim_by_chat(&mut self, chat_id: TelegramChatId, amount_claimed: WrappedBalance, token_account_id: TokenAccountId) -> bool;
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
#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
    Tips,
    TipsLookupMap,
    ChatPointsLookupMap,
    TelegramDepositsLookupMap,
    TelegramTipsLookupMap,
    TelegramDeposits,
    TelegramUsersInChats,
    WhitelistedTokensLookupSet,
    ChatPointsLookupMapU128,
    ChatSettingsLookupMap,
    TreasureLookupMap
}

#[near_bindgen]
impl NearTips {
    #[init]
    pub fn new() -> Self {
        Self {
            deposits: LookupMap::new(StorageKey::TelegramDepositsLookupMap),
            telegram_tips: LookupMap::new(StorageKey::TelegramTipsLookupMap), // first object only for telegram tips
            tips: LookupMap::new(StorageKey::TipsLookupMap), // generic object for any tips
            telegram_users_in_chats: LookupSet::new(StorageKey::TelegramUsersInChats),
            chat_points: LookupMap::new(StorageKey::ChatPointsLookupMap),
            whitelisted_tokens: LookupSet::new(StorageKey::WhitelistedTokensLookupSet),
            version: 0,
            withdraw_available: true,
            tip_available: true,
            generic_tips_available: false,
            telegram_tips_v1: HashMap::new(),
            total_chat_points: 0,
            chat_settings: LookupMap::new(StorageKey::ChatSettingsLookupMap),
            treasure: LookupMap::new(StorageKey::TreasureLookupMap),
        }
    }

    /* INITIAL FUNCTIONS, using telegram_tips object */

    #[payable]
    pub fn deposit(&mut self, account_id: Option<ValidAccountId> ) {
        self.assert_withdraw_available();

        let account_id_prepared: ValidAccountId = account_id.unwrap_or(
            ValidAccountId::try_from(env::predecessor_account_id()).unwrap()
        );
        let attached_deposit: Balance = env::attached_deposit();

        self.deposit_amount_to_account(account_id_prepared.as_ref(), attached_deposit, Some(NEAR.to_string()));
    }

    pub(crate) fn deposit_amount_to_account(&mut self, account_id: &AccountId, amount: Balance, token_id: Option<TokenAccountId>) {
        self.assert_withdraw_available();
        let token_id_unwrapped = NearTips::unwrap_token_id(&token_id);

        if token_id_unwrapped == NEAR {
            assert!(amount >= MIN_DEPOSIT_NEAR, "Minimum deposit is 0.1");
        } else {
            assert!(amount >= MIN_DEPOSIT_FT, "Minimum deposit is 0.1");
        }

        self.assert_check_whitelisted_token(&token_id);

        let account_id_prepared : ValidAccountId = ValidAccountId::try_from(account_id.as_str()).unwrap();
        let deposit: Balance = self.get_deposit(account_id_prepared.clone(), token_id).0;

        self.deposits.insert(
            &TokenByNearAccount {
                account_id: account_id.clone(),
                token_account_id: token_id_unwrapped.clone(),
            },
            &(deposit + amount),
        );

        env::log(format!("@{} deposited {} of {:?}", account_id_prepared, amount, token_id_unwrapped).as_bytes());
    }

    pub fn get_deposit(&self, account_id: ValidAccountId, token_id: Option<TokenAccountId>) -> WrappedBalance {
        self.deposits.get(
            &TokenByNearAccount {
                account_id: account_id.into(),
                token_account_id: NearTips::unwrap_token_id(&token_id),
            }).unwrap_or(0).into()
    }

    pub fn get_deposits(&self,
                        account_id: ValidAccountId,
                        token_ids: Vec<TokenAccountId>,
    ) -> HashMap<TokenAccountId, WrappedBalance> {
        token_ids
            .iter()
            .map(|token_account_id|
                (
                    token_account_id.clone(),
                    self.get_deposit(account_id.clone(), Some(token_account_id.clone()))
                ))
            .collect()
    }

    pub fn get_balance(&self,
                       telegram_account: TelegramAccountId,
                       token_id: Option<TokenAccountId>,
    ) -> WrappedBalance {
        self.telegram_tips.get(
            &TokenByTelegramAccount {
                telegram_account,
                token_account_id: NearTips::unwrap_token_id(&token_id),
            }
        ).unwrap_or(0).into()
    }

    pub fn get_balances(&self,
                        telegram_account: TelegramAccountId,
                        token_ids: Vec<TokenAccountId>,
    ) -> HashMap<TokenAccountId, WrappedBalance> {
        token_ids
            .iter()
            .map(|token_account_id|
                (
                    token_account_id.clone(),
                    self.get_balance(telegram_account, Some(token_account_id.clone())),
                ))
            .collect()
    }

    pub fn get_chat_settings(&self, chat_id: TelegramChatId) -> Option<ChatSettings> {
        self.chat_settings.get(&chat_id)
    }

    pub fn get_chat_numerator(&self, chat_id: TelegramChatId) -> TreasureFeeNumerator {
        let settings = self.get_chat_settings(chat_id);
        if let Some(settings_unwrapped) = settings { 
            settings_unwrapped.treasure_fee_numerator
        }
        else {
            0
        }
    }

    pub fn get_treasure_balance(&self, token_account_id: Option<TokenAccountId>) -> WrappedBalance {
        let token_id_unwrapped = NearTips::unwrap_token_id(&token_account_id);
        self.treasure.get(&token_id_unwrapped).unwrap_or(0).into()
    }

    pub fn add_chat_settings(&mut self, chat_id: TelegramChatId, admin_account_id: ValidAccountId, treasure_fee_numerator: TreasureFeeNumerator) {
        NearTips::assert_master_account_id();

        self.chat_settings.insert(&chat_id, &ChatSettings {
            admin_account_id: admin_account_id.into(),
            treasure_fee_numerator,
        });
    }

    pub fn delete_chat_settings(&mut self, chat_id: TelegramChatId) {
        NearTips::assert_master_account_id();
        self.chat_settings.remove(&chat_id);
    }

    pub fn claim_chat_tokens(&mut self, chat_id: TelegramChatId, token_id: Option<TokenAccountId>) -> Promise {
        let settings = self.get_chat_settings(chat_id);
        assert!(settings.is_some(), "Unknown chat");

        let account_id = env::predecessor_account_id();
        let admin_account_id: AccountId = settings.unwrap().admin_account_id;
        assert_eq!(admin_account_id, account_id, "Current user is not a chat admin");

        let token_id_unwrapped = NearTips::unwrap_token_id(&token_id);
        assert!(token_id_unwrapped != "point".to_string(), "Claim tokens using this method");

        let chat_balance: RewardPoint = self.get_chat_score(chat_id, token_id).0;
        assert!(chat_balance > 0, "Nothing to claim");

        let treasure_balance: Balance = self.treasure.get(&token_id_unwrapped).unwrap_or(0);
        assert!(treasure_balance >= chat_balance, "Not enough funds in treasure");
        self.treasure.insert(&token_id_unwrapped, &(treasure_balance - chat_balance));

        let points_by_chat = TokenByTelegramChat {
            chat_id,
            token_account_id: token_id_unwrapped.clone(),
        };
        self.chat_points.insert(&points_by_chat, &0);

        if token_id_unwrapped == NEAR {
            Promise::new(account_id.to_string()).transfer(chat_balance)
        } else {
            ext_fungible_token::ft_transfer(
                account_id.to_string(),
                chat_balance.into(),
                Some(format!(
                    "Claimed by {} on behalf of chat {}: {} of {:?}",
                    account_id,
                    chat_balance,
                    chat_id,
                    token_id_unwrapped
                )),
                &token_id_unwrapped,
                ONE_YOCTO,
                GAS_FOR_FT_TRANSFER,
            )
                .then(ext_self::after_ft_transfer_claim_by_chat(
                    chat_id,
                    chat_balance.into(),
                    token_id_unwrapped.clone(),
                    &env::current_account_id(),
                    NO_DEPOSIT,
                    GAS_FOR_AFTER_FT_TRANSFER,
                ))
        }
    }

    #[private]
    pub fn after_ft_transfer_claim_by_chat(
        &mut self,
        chat_id: TelegramChatId,
        amount_claimed: WrappedBalance,
        token_account_id: TokenAccountId,
    ) -> bool {
        let promise_success = is_promise_success();
        if !is_promise_success() {
            log!(
                "FT claim of {} on behalf of chat {} failed. Points to recharge: {}",
                token_account_id,
                chat_id,
                amount_claimed.0
            );

            let token_by_chat = TokenByTelegramChat {
                chat_id,
                token_account_id: token_account_id.clone(),
            };

            let chat_score: RewardPoint = self.chat_points.get(&token_by_chat).unwrap_or(0);
            self.chat_points.insert(&token_by_chat, &(chat_score + amount_claimed.0));

            let treasure_balance: Balance = self.treasure.get(&token_account_id).unwrap_or(0);
            self.treasure.insert(&token_account_id, &(treasure_balance + amount_claimed.0));
        }
        promise_success
    }


    pub fn get_chat_score(&self, chat_id: TelegramChatId, token_id: Option<TokenAccountId>) -> WrappedBalance {
        let token_account_id = NearTips::unwrap_token_id(&token_id);
        self.chat_points.get(&TokenByTelegramChat {
            chat_id,
            token_account_id,
        }).unwrap_or(0).into()
    }


    pub fn get_telegram_users_in_chats(&self, telegram_id: TelegramAccountId,
                                       chat_id: TelegramChatId) -> bool {
        self.telegram_users_in_chats.contains(&TelegramUserInChat {
            telegram_id,
            chat_id,
        })
    }

    pub fn assert_valid_treasure_fee_numerator(numerator: TreasureFeeNumerator) {
        assert!(
            numerator <= 10,
            "Treasure fee can't be greater then 10%"
        );
    }

    pub fn multiply_treasure_fee_numerator(value: Balance, numerator: TreasureFeeNumerator) -> Balance {
        (U256::from(numerator) * U256::from(value) / U256::from(100)).as_u128() 
    }


    pub(crate) fn send_tip_to_telegram_from_account(&mut self,
                                                    sender_account_id: AccountId,
                                                    telegram_account: TelegramAccountId,
                                                    amount: WrappedBalance,
                                                    chat_id: Option<TelegramChatId>,
                                                    token_id: Option<TokenAccountId>) {
        self.assert_tip_available();
        self.assert_check_whitelisted_token(&token_id);
        assert!(amount.0 > 0, "Positive amount needed");

        let sender_account_id_prepared = ValidAccountId::try_from(sender_account_id.clone()).unwrap();
        let deposit: Balance = NearTips::get_deposit(self, sender_account_id_prepared, token_id.clone()).0;

        assert!(
            amount.0 <= deposit,
            "Not enough tokens deposited to tip (Deposit: {}. Requested: {})",
            deposit, amount.0
        );

        let mut tip_amount: Balance = amount.0;
        let token_id_unwrapped = NearTips::unwrap_token_id(&token_id);

        // trasure fee & points
        if let Some(chat_id_unwrapped) = chat_id {
            if chat_id_unwrapped > 0 {
                let treasure_fee_numerator = self.get_chat_numerator(chat_id_unwrapped);

                if amount.0 > MIN_AMOUNT_TO_REWARD_CHAT {
                    let points_by_chat = TokenByTelegramChat {
                        chat_id: chat_id_unwrapped,
                        token_account_id: "point".to_string(),
                    };

                    let user_in_chat: TelegramUserInChat = TelegramUserInChat {
                        telegram_id: telegram_account,
                        chat_id: chat_id_unwrapped,
                    };

                    if !self.telegram_users_in_chats.contains(&user_in_chat) {
                        let chat_score: RewardPoint = self.chat_points.get(&points_by_chat).unwrap_or(0);
                        let new_score = chat_score + 1;
                        self.chat_points.insert(&points_by_chat, &new_score);
                        self.telegram_users_in_chats.insert(&user_in_chat);
                        env::log(format!("Reward point for chat {} added. Total: {}", chat_id_unwrapped, new_score).as_bytes());
                    }
                }

                NearTips::assert_valid_treasure_fee_numerator(treasure_fee_numerator);
                let treasure_fee = NearTips::multiply_treasure_fee_numerator(amount.0, treasure_fee_numerator);
                tip_amount = amount.0 - treasure_fee; // overwrite tip amount

                if treasure_fee > 0  {
                    let token_by_chat = TokenByTelegramChat {
                        chat_id: chat_id_unwrapped,
                        token_account_id: token_id_unwrapped.clone(),
                    };

                    let chat_score: RewardPoint = self.chat_points.get(&token_by_chat).unwrap_or(0);
                    let new_score = chat_score + treasure_fee;
                    self.chat_points.insert(&token_by_chat, &new_score);

                    let treasure_balance: Balance = self.treasure.get(&token_id_unwrapped).unwrap_or(0);
                    self.treasure.insert(
                        &token_id_unwrapped, 
                        &(treasure_balance + treasure_fee)
                    );    

                    env::log(format!("@{} tipped {} of {:?} for telegram account {}. {} reward point(s) for chat {} added. Total: {}", sender_account_id, tip_amount, token_id_unwrapped, telegram_account, treasure_fee, chat_id_unwrapped, new_score).as_bytes());
                } else {
                    env::log(format!("@{} tipped {} of {:?} for telegram account {}. No reward point(s) added", sender_account_id, tip_amount, token_id_unwrapped, telegram_account).as_bytes());
                }
                
            }
        } else {
            env::log(format!("@{} tipped {} of {:?} for telegram account {}", sender_account_id, tip_amount, token_id_unwrapped, telegram_account).as_bytes());
        }

        // perform a tip
        let balance: Balance = NearTips::get_balance(self, telegram_account, token_id.clone()).0;
        self.telegram_tips.insert(
            &TokenByTelegramAccount {
                telegram_account,
                token_account_id: token_id_unwrapped.clone(),
            },
            &(balance + tip_amount));

        self.deposits.insert( // TODO set_balance helper
                              &TokenByNearAccount {
                                  account_id: sender_account_id.clone(),
                                  token_account_id: token_id_unwrapped.clone(),
                              },
                              &(deposit - amount.0));
    }

    #[private]
    pub fn after_ft_transfer_to_treasure(
        &mut self,
        chat_id: TelegramChatId,
        treasure_fee: WrappedBalance,
        token_account_id: TokenAccountId,
    ) -> bool {
        let promise_success = is_promise_success();
        if !is_promise_success() {
            log!(
                "FT transfer of {} to treasure failed. Points to recharge: {}, chat_id: {}",
                token_account_id,
                treasure_fee.0,
                chat_id
            );

            let token_by_chat = TokenByTelegramChat {
                chat_id,
                token_account_id,
            };

            let chat_score: RewardPoint = self.chat_points.get(&token_by_chat).unwrap_or(0);

            self.chat_points.insert(&token_by_chat, &(chat_score - treasure_fee.0));
        }
        promise_success
    }

    pub fn send_tip_to_telegram(&mut self,
                                telegram_account: TelegramAccountId,
                                amount: WrappedBalance,
                                chat_id: Option<TelegramChatId>,
                                token_id: Option<TokenAccountId>) {
        let account_id = env::predecessor_account_id();
        self.send_tip_to_telegram_from_account(account_id, telegram_account, amount, chat_id, token_id);
    }

    pub fn send_tip_to_telegram_with_auth(&mut self,
                                          telegram_account: TelegramAccountId,
                                          amount: WrappedBalance,
                                          chat_id: Option<TelegramChatId>,
                                          token_id: Option<TokenAccountId>) -> Promise {
        self.assert_tip_available();
        assert!(amount.0 > 0, "Positive amount needed");
        self.assert_check_whitelisted_token(&token_id);

        let account_id = env::predecessor_account_id();
        let account_id_prepared : ValidAccountId = ValidAccountId::try_from(account_id.clone()).unwrap();

        let deposit: Balance = NearTips::get_deposit(self, account_id_prepared, token_id.clone()).0;

        assert!(amount.0 <= deposit, "Not enough tokens to tip (Deposit: {}. Requested: {})", deposit, amount.0);

        let contact: Contact = Contact {
            category: ContactCategories::Telegram,
            value: "".to_string(),
            account_id: Some(telegram_account),
        };

        self.get_contact_owner(contact, AUTH_ACCOUNT_ID.to_string()).
            then(ext_self::on_get_contact_owner_on_send_tip_to_telegram_with_auth(
                account_id,
                amount.0,
                telegram_account,
                chat_id,
                token_id,
                &env::current_account_id(),
                NO_DEPOSIT,
                CALLBACK_GAS * 2,
            ))
    }

    pub fn on_get_contact_owner_on_send_tip_to_telegram_with_auth(&mut self,
                                                                  #[callback] account: Option<AccountId>,
                                                                  sender_account_id: AccountId,
                                                                  tip_amount: Balance,
                                                                  telegram_account: TelegramAccountId,
                                                                  chat_id: Option<TelegramChatId>,
                                                                  token_id: Option<TokenAccountId>) {
        assert_eq!(
            env::predecessor_account_id(),
            env::current_account_id(),
            "Callback can only be called from the contract"
        );

        let token_id_unwrapped = NearTips::unwrap_token_id(&token_id);

        let sender_account_id_prepared : ValidAccountId = ValidAccountId::try_from(sender_account_id.clone()).unwrap();
        let sender_deposit: Balance = self.get_deposit(sender_account_id_prepared.clone(), token_id.clone()).0;
        assert!(tip_amount <= sender_deposit, "Not enough tokens to tip (Deposit: {}. Requested: {})", sender_deposit, tip_amount);

        match account {
            Some(account_id) => {
                let valid_account_id = ValidAccountId::try_from(account_id.clone()).unwrap();
                self.deposit_amount_to_account(valid_account_id.as_ref(), tip_amount, token_id);
                self.deposits.insert(
                    &TokenByNearAccount {
                        account_id: sender_account_id.clone(),
                        token_account_id: token_id_unwrapped.clone(),
                    },
                    &(sender_deposit - tip_amount));
                env::log(format!("@{} tipped {} of {:?} to NEAR account {} telegram account {}", sender_account_id, tip_amount, token_id_unwrapped, account_id, telegram_account).as_bytes());
            }
            None => {
                env::log(format!("Authorized contact wasn't found for telegram {}. Continue to send from @{}", telegram_account, sender_account_id).as_bytes());
                self.send_tip_to_telegram_from_account(sender_account_id, telegram_account, U128::from(tip_amount), chat_id, token_id);
            }
        }
    }

    pub(crate) fn assert_check_whitelisted_token(&self, token_id: &Option<TokenAccountId>) {
        if let Some(token_id) = token_id {
            assert!(self.whitelisted_tokens.contains(&token_id), "Token wasn't whitelisted");
        }
    }

    pub fn whitelist_token(&mut self, token_id: TokenAccountId) {
        NearTips::assert_master_account_id();

        if !self.whitelisted_tokens.contains(&token_id) {
            self.whitelisted_tokens.insert(&token_id);
        }
    }

    pub fn is_whitelisted_token(&self, token_id: TokenAccountId) -> bool {
        self.whitelisted_tokens.contains(&token_id)
    }

    pub(crate) fn unwrap_token_id(token_id: &Option<TokenAccountId>) -> TokenAccountId {
        token_id.clone().unwrap_or_else(|| NEAR.to_string())
    }

    // centralized tips withdraw, with master_account authorisation
    pub fn withdraw_from_telegram(&mut self,
                                  telegram_account: TelegramAccountId,
                                  account_id: AccountId,
                                  token_id: Option<TokenAccountId>) -> Promise { // TODO FT ft_on_transfer
        self.assert_withdraw_available();
        NearTips::assert_master_account_id();
        assert!(env::is_valid_account_id(account_id.as_bytes()), "Account @{} is invalid", account_id);

        let balance: Balance = self.get_balance(telegram_account, token_id.clone()).0;

        let token_id_unwrapped = NearTips::unwrap_token_id(&token_id);

        let amount: Balance;

        if token_id_unwrapped == NEAR {
            assert!(balance > WITHDRAW_COMMISSION, "Not enough tokens to pay withdraw commission"); // TODO COMMISSION IN NEAR?
            amount = balance - WITHDRAW_COMMISSION;
            Promise::new(MASTER_ACCOUNT_ID.to_string()).transfer(WITHDRAW_COMMISSION);
        } else {
            amount = balance;
        }

        self.telegram_tips.insert(
            &TokenByTelegramAccount {
                telegram_account,
                token_account_id: token_id_unwrapped.clone(),
            },
            &0);

        env::log(format!("@{} is withdrawing {} of {:?} from telegram account {}",
                         account_id, amount, token_id_unwrapped, telegram_account).as_bytes());

        if token_id_unwrapped == NEAR {
            Promise::new(account_id).transfer(amount)
        } else {
            ext_fungible_token::ft_transfer(
                account_id,
                amount.into(),
                Some(format!(
                    "Claiming tips: {} of {:?} from @{}",
                    amount,
                    token_id_unwrapped,
                    env::current_account_id()
                )),
                &token_id_unwrapped,
                ONE_YOCTO,
                GAS_FOR_FT_TRANSFER,
            )
                .then(ext_self::after_ft_transfer_balance(
                    telegram_account,
                    amount.into(),
                    token_id_unwrapped,
                    &env::current_account_id(),
                    NO_DEPOSIT,
                    GAS_FOR_AFTER_FT_TRANSFER,
                ))
        }
    }


    pub fn after_ft_transfer_balance(
        &mut self,
        telegram_account: TelegramAccountId,
        amount: WrappedBalance,
        token_account_id: TokenAccountId,
    ) -> bool {
        assert_eq!(
            env::predecessor_account_id(),
            env::current_account_id(),
            "Callback can only be called from the contract"
        );

        let promise_success = is_promise_success();
        if !is_promise_success() {
            log!(
                "Token {} withdraw by telegram account {} failed. Amount to recharge: {}",
                token_account_id,
                telegram_account,
                amount.0
            );

            let current_balance = self.get_balance(telegram_account, Some(token_account_id.clone()));

            self.telegram_tips.insert(
                &TokenByTelegramAccount {
                    telegram_account,
                    token_account_id,
                },
                &(current_balance.0 + amount.0),
            );
        }
        promise_success
    }

    pub fn after_ft_transfer_deposit(
        &mut self,
        account_id: AccountId,
        amount: WrappedBalance,
        token_account_id: TokenAccountId,
    ) -> bool {
        assert_eq!(
            env::predecessor_account_id(),
            env::current_account_id(),
            "Callback can only be called from the contract"
        );

        let promise_success = is_promise_success();
        if !is_promise_success() {
            log!(
                "Token {} withdraw for user {} failed. Amount to recharge: {}",
                token_account_id,
                account_id,
                amount.0
            );

            let account_id_prepared : ValidAccountId = ValidAccountId::try_from(account_id.clone()).unwrap();
            let current_deposit = self.get_deposit(account_id_prepared, Some(token_account_id.clone()));

            self.deposits.insert(
                &TokenByNearAccount {
                    account_id,
                    token_account_id,
                },
                &(current_deposit.0 + amount.0),
            );
        }
        promise_success
    }

    #[payable]
    // decentralized tips withdraw for those who made near auth. telegram_account is numeric ID 123123123
    pub fn withdraw_from_telegram_with_auth(&mut self,
                                            telegram_account: TelegramAccountId,
                                            token_id: Option<TokenAccountId>) -> Promise {
        self.assert_withdraw_available();

        let account_id = env::predecessor_account_id();

        let contact: Contact = Contact {
            category: ContactCategories::Telegram,
            value: "".to_string(),
            account_id: Some(telegram_account),
        };

        self.get_contact_owner(contact.clone(), AUTH_ACCOUNT_ID.to_string()).
            then(ext_self::on_get_contact_owner_on_withdraw_from_telegram_with_auth(
                account_id,
                contact,
                token_id,
                &env::current_account_id(),
                NO_DEPOSIT,
                CALLBACK_GAS * 3,
            ))
    }

    pub fn on_get_contact_owner_on_withdraw_from_telegram_with_auth(&mut self,
                                                                    #[callback] account: Option<AccountId>,
                                                                    recipient_account_id: AccountId,
                                                                    contact: Contact,
                                                                    token_id: Option<TokenAccountId>) -> Promise {
        assert_eq!(
            env::predecessor_account_id(),
            env::current_account_id(),
            "Callback can only be called from the contract"
        );

        match account {
            Some(account) => {
                assert!(account == recipient_account_id, "Not authorized to withdraw");
                assert!(!contact.account_id.is_none(), "Account ID is missing");

                let balance: Balance = NearTips::get_balance(self, contact.account_id.unwrap(), token_id.clone()).0;
                assert!(balance > 0, "Not enough tokens to withdraw");

                let telegram_account = contact.account_id.unwrap();
                let predecessor_account_id = env::predecessor_account_id();
                let token_id_unwrapped = NearTips::unwrap_token_id(&token_id);

                self.telegram_tips.insert(
                    &TokenByTelegramAccount {
                        telegram_account,
                        token_account_id: token_id_unwrapped.clone(),
                    }
                    , &0);

                env::log(format!("@{} withdrew {} of {:?} from telegram account {}",
                                 predecessor_account_id, balance, token_id_unwrapped, telegram_account).as_bytes());

                //Promise::new(recipient_account_id).transfer(balance)

                if token_id_unwrapped == NEAR {
                    Promise::new(recipient_account_id).transfer(balance)
                } else {
                    ext_fungible_token::ft_transfer(
                        recipient_account_id,
                        balance.into(),
                        Some(format!(
                            "Claiming tips: {} of {:?} from @{}",
                            balance,
                            token_id_unwrapped,
                            env::current_account_id()
                        )),
                        &token_id_unwrapped,
                        ONE_YOCTO,
                        GAS_FOR_FT_TRANSFER,
                    )
                        .then(ext_self::after_ft_transfer_balance(
                            telegram_account,
                            balance.into(),
                            token_id_unwrapped,
                            &env::current_account_id(),
                            NO_DEPOSIT,
                            GAS_FOR_AFTER_FT_TRANSFER,
                        ))
                }
            }
            None => {
                panic!("Contact wasn't authorized to any account");
            }
        }
    }

    // withdraw from deposit
    pub fn withdraw(&mut self, token_id: Option<TokenAccountId>) -> Promise {
        self.assert_withdraw_available();
        self.assert_check_whitelisted_token(&token_id);

        let account_id = env::predecessor_account_id();
        let account_id_prepared : ValidAccountId = ValidAccountId::try_from(account_id.clone()).unwrap();
        let deposit: Balance = NearTips::get_deposit(self, account_id_prepared, token_id.clone()).0;

        assert!(deposit > 0, "Missing deposit");

        let token_id_unwrapped = NearTips::unwrap_token_id(&token_id);

        self.deposits.insert(
            &TokenByNearAccount {
                account_id: account_id.clone(),
                token_account_id: token_id_unwrapped.clone(),
            },
            &0);

        env::log(format!("@{} withdrew {} of {:?} from internal deposit", account_id, deposit, token_id_unwrapped).as_bytes());

        if token_id_unwrapped == NEAR {
            Promise::new(account_id).transfer(deposit)
        } else {
            ext_fungible_token::ft_transfer(
                account_id.clone(),
                deposit.into(),
                Some(format!(
                    "Claiming tips: {} of {:?} from @{}",
                    deposit,
                    token_id_unwrapped,
                    env::current_account_id()
                )),
                &token_id_unwrapped,
                ONE_YOCTO,
                GAS_FOR_FT_TRANSFER,
            )
                .then(ext_self::after_ft_transfer_deposit(
                    account_id,
                    deposit.into(),
                    token_id_unwrapped,
                    &env::current_account_id(),
                    NO_DEPOSIT,
                    GAS_FOR_AFTER_FT_TRANSFER,
                ))
        }
    }

    pub fn withdraw_linkdrop(&mut self, public_key: String, telegram_account: TelegramAccountId) -> Promise {
        self.assert_withdraw_available();
        NearTips::assert_master_account_id();
        // TODO Linkdrop for NOT NEAR???
        let balance: Balance = NearTips::get_balance(self, telegram_account, Some(NEAR.to_string())).0;
        assert!(balance > WITHDRAW_COMMISSION + ACCESS_KEY_ALLOWANCE, "Not enough tokens to pay for key allowance and withdraw commission");

        let amount = balance - WITHDRAW_COMMISSION;

        self.telegram_tips.insert(
            &TokenByTelegramAccount {
                telegram_account,
                token_account_id: NEAR.to_string(),
            },
            &0);
        Promise::new(MASTER_ACCOUNT_ID.to_string()).transfer(WITHDRAW_COMMISSION);

        env::log(format!("Telegram account {} withdrew {} yNEAR with linkDrop for public key {}. Withdraw commission: {} yNEAR",
                         telegram_account, amount, public_key, WITHDRAW_COMMISSION).as_bytes());

        linkdrop::send(public_key, &LINKDROP_ACCOUNT_ID.to_string(), amount, BASE_GAS)
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
    pub fn tip_contact_to_deposit(&mut self, telegram_account: TelegramAccountId, amount: WrappedBalance, token_id: Option<TokenAccountId>) -> Promise {
        self.assert_tip_available();
        assert!(amount.0 > 0, "Positive amount needed");
        self.assert_check_whitelisted_token(&token_id);

        let account_id = env::predecessor_account_id();
        let account_id_prepared : ValidAccountId = ValidAccountId::try_from(account_id.clone()).unwrap();
        let deposit: Balance = NearTips::get_deposit(self, account_id_prepared, token_id.clone()).0;

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

        self.get_contact_owner(contact.clone(), AUTH_ACCOUNT_ID.to_string()).
            then(ext_self::on_get_contact_owner_on_tip_contact_to_deposit(
                account_id,
                contact,
                amount.0,
                token_id,
                &env::current_account_id(),
                NO_DEPOSIT,
                CALLBACK_GAS,
            ))
    }

    pub fn on_get_contact_owner_on_tip_contact_to_deposit(&mut self,
                                                          #[callback] account: Option<AccountId>,
                                                          sender_account_id: AccountId,
                                                          contact: Contact,
                                                          amount: Balance,
                                                          token_id: Option<TokenAccountId>) {
        assert_eq!(
            env::predecessor_account_id(),
            env::current_account_id(),
            "Callback can only be called from the contract"
        );

        assert!(!account.is_none(), "Owner not found");
        let receiver_account_id: AccountId = account.unwrap();

        let sender_account_id_prepared : ValidAccountId = ValidAccountId::try_from(sender_account_id.clone()).unwrap();
        let sender_deposit: Balance = NearTips::get_deposit(self, sender_account_id_prepared, token_id.clone()).0;
        let token_id_unwrapped = token_id.clone().unwrap_or_else(|| NEAR.to_string());
        self.deposits.insert(
            &TokenByNearAccount {
                account_id: sender_account_id.clone(),
                token_account_id: token_id_unwrapped.clone(),
            },
            &(sender_deposit - amount));

        let receiver_account_id_prepared : ValidAccountId = ValidAccountId::try_from(receiver_account_id.clone()).unwrap();
        let receiver_deposit: Balance = NearTips::get_deposit(self, receiver_account_id_prepared, token_id).0;
        self.deposits.insert(
            &TokenByNearAccount {
                account_id: receiver_account_id.clone(),
                token_account_id: token_id_unwrapped.clone(),
            },
            &(receiver_deposit + amount));

        env::log(format!("@{} transferred {} of {:?} to deposit of @{} [{:?} account {:?}]",
                         sender_account_id, amount, token_id_unwrapped, receiver_account_id, contact.category, contact.value).as_bytes());
    }

    #[payable]
    // tip attached tokens without knowing NEAR account id
    pub fn tip_contact_with_attached_tokens(&mut self, contact: Contact) -> Promise {
        self.assert_tip_available();
        let deposit: Balance = near_sdk::env::attached_deposit();

        let account_id = env::predecessor_account_id();

        self.get_contact_owner(contact.clone(), AUTH_ACCOUNT_ID.to_string()).
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


    pub(crate) fn assert_tip_available(&self) {
        assert!(self.tip_available, "Tips paused");
    }

    pub(crate) fn assert_withdraw_available(&self) {
        assert!(self.withdraw_available, "Withdrawals paused");
    }

    /* GENERIC TIPS, using tips object */

    pub(crate) fn assert_generic_tips_available(&self) {
        assert!(self.generic_tips_available, "Generic tips and withdrawals disabled");
    }

    pub(crate) fn tip_transfer(&mut self,
                               sender_account_id: AccountId,
                               receiver_account_id: AccountId,
                               contact: Contact,
                               deposit: Balance) {
        self.assert_tip_available();
        self.assert_generic_tips_available();

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
        self.assert_tip_available();
        self.assert_generic_tips_available();

        let deposit: Balance = near_sdk::env::attached_deposit();
        let account_id = env::predecessor_account_id();

        NearTips::tip_transfer(self, account_id, receiver_account_id, contact, deposit);
    }

    pub fn get_tips(&self, account_id: AccountId) -> Option<Vec<Tip>> {
        self.tips.get(&account_id).map(|tips| tips.to_vec())
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

        self.get_contact_owner(contact.clone(), AUTH_ACCOUNT_ID.to_string())
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

        assert_eq!(
            contact_owner_account_id,
            recipient_account_id,
            "Current user not allowed to withdraw tip for this contact");

        env::log(format!("Transfer to @{} [{:?} account {:?}]", recipient_account_id, recipient_contact.category, recipient_contact.value).as_bytes());

        Promise::new(recipient_account_id)
            .transfer(balance_to_withdraw)
            .then(ext_self::on_withdraw_tip(
                UNDEFINED_ACCOUNT_ID.to_string(),
                recipient_contact,
                balance_to_withdraw,
                &env::current_account_id(),
                0,
                CALLBACK_GAS,
            ))
    }

    fn withdraw_tip_for_current_account(&self, contact: Contact, balance_to_withdraw: Balance) -> Promise {
        let account_id = env::predecessor_account_id();

        auth::get_contacts(account_id.clone(), &AUTH_ACCOUNT_ID.to_string(), NO_DEPOSIT, BASE_GAS)
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
                    env::log(format!("Check: [{:?} account {:?}]", contact.category, contact.value).as_bytes());
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
        self.assert_withdraw_available();
        self.assert_generic_tips_available();

        // check tips sent exactly to this account
        let account_id = env::predecessor_account_id();
        let balance_of_account: Balance = NearTips::get_tip_by_contact(self, account_id.clone(), contact.clone()).0;

        // check tips sent exactly to contacts belongs to undefined account
        let balance_of_undefined_account: Balance = NearTips::get_tip_by_contact(self, UNDEFINED_ACCOUNT_ID.to_string(), contact.clone()).0;

        env::log(format!("balance_of_account {} found", balance_of_account).as_bytes());
        env::log(format!("balance_of_undefined_account {} found", balance_of_undefined_account).as_bytes());

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
        NearTips::assert_master_account_id();
        self.withdraw_available = withdraw_available;
    }

    pub fn get_withdraw_available(self) -> bool {
        self.withdraw_available
    }

    pub fn set_tip_available(&mut self, tip_available: bool) {
        NearTips::assert_master_account_id();
        self.tip_available = tip_available;
    }

    pub fn get_tip_available(self) -> bool {
        self.tip_available
    }

    pub fn set_generic_tips_available(&mut self, generic_tips_available: bool) {
        NearTips::assert_master_account_id();
        self.generic_tips_available = generic_tips_available;
    }

    pub fn get_generic_tips_available(self) -> bool {
        self.generic_tips_available
    }

    pub fn transfer_tips_to_deposit(&mut self, telegram_account: TelegramAccountId,
                                    account_id: AccountId,
                                    token_id: Option<TokenAccountId>) {
        self.assert_withdraw_available();
        self.assert_check_whitelisted_token(&token_id);
        NearTips::assert_master_account_id();
        assert!(env::is_valid_account_id(account_id.as_bytes()), "Account @{} is invalid", account_id);

        let balance: Balance = NearTips::get_balance(self, telegram_account, token_id.clone()).0;

        let amount: Balance;

        let token_id_unwrapped = NearTips::unwrap_token_id(&token_id);
        if token_id_unwrapped == NEAR { // TODO commission for DAI withdrawals?
            assert!(balance > WITHDRAW_COMMISSION, "Not enough tokens to pay transfer commission");
            amount = balance - WITHDRAW_COMMISSION;
            Promise::new(MASTER_ACCOUNT_ID.to_string()).transfer(WITHDRAW_COMMISSION);
        } else {
            amount = balance;
        }

        let account_id_prepared : ValidAccountId = ValidAccountId::try_from(account_id.clone()).unwrap();

        let deposit: Balance = NearTips::get_deposit(self, account_id_prepared, token_id).0;
        self.deposits.insert(
            &TokenByNearAccount {
                account_id: account_id.clone(),
                token_account_id: token_id_unwrapped.clone(),
            },
            &(deposit + amount));

        self.telegram_tips.insert(&TokenByTelegramAccount {
            telegram_account,
            token_account_id: token_id_unwrapped.clone(),
        }, &0);

        env::log(format!("@{} transfer {} of {:?} from telegram account {}. Transfer commission: {} yNEAR",
                         account_id, balance, token_id_unwrapped, telegram_account, WITHDRAW_COMMISSION).as_bytes());
    }

    pub fn assert_self() {
        assert_eq!(env::predecessor_account_id(), env::current_account_id());
    }

    pub fn assert_master_account_id(){
        assert_eq!(env::predecessor_account_id(), MASTER_ACCOUNT_ID, "No access");
    }

    pub fn get_master_account_id() -> String {
        MASTER_ACCOUNT_ID.to_string()
    }

    pub fn get_version(&self) -> u16 {
        self.version
    }

    /*
    #[init(ignore_state)]
    #[allow(dead_code)]
    pub fn migrate_state_3_1() -> Self { // add telegram_users_in_chats, Migration to token balances / deposits
        let migration_version: u16 = 2;
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
        let telegram_users_in_chats = LookupSet::new(StorageKey::TelegramUsersInChats);
        let tips_new = LookupMap::new(StorageKey::TipsLookupMap);
        let chat_points = LookupMap::new(StorageKey::ChatPointsLookupMap);

        let near_account_id: TokenAccountId = NEAR.to_string();
        let telegram_tips_new = LookupMap::new(b"a".to_vec());

        let mut deposits_new = LookupMap::new(StorageKey::TelegramDepositsLookupMap);
        for (account_id, deposit) in &old_contract.deposits {
            deposits_new.insert(
                &TokenByNearAccount {
                    account_id: account_id.to_string(),
                    token_account_id: near_account_id.clone(),
                },
                deposit,
            );
        }

        let mut whitelisted_tokens_new = LookupSet::new(StorageKey::WhitelistedTokensLookupSet);
        whitelisted_tokens_new.insert(&near_account_id);

        Self {
            deposits: deposits_new,
            telegram_tips: telegram_tips_new,
            tips: tips_new,
            telegram_users_in_chats,
            chat_points,
            whitelisted_tokens: whitelisted_tokens_new,
            version: migration_version,
            withdraw_available: old_contract.withdraw_available,
            tip_available: old_contract.tip_available,
            generic_tips_available: false,

            telegram_tips_v1: old_contract.telegram_tips
        }
    }

    #[init(ignore_state)]
    #[allow(dead_code)]
    pub fn migrate_state_3_2(iteration: u16) -> Self { // telegram_tips_v1 transition
        let migration_version: u16 = 3;
        assert_eq!(env::predecessor_account_id(), env::current_account_id(), "Private function");

        #[derive(BorshDeserialize)]
        struct OldContract {
            deposits: LookupMap<TokenByNearAccount, Balance>,
            telegram_tips: LookupMap<TokenByTelegramAccount, Balance>,
            tips: LookupMap<AccountId, Vec<Tip>>,
            telegram_users_in_chats: LookupSet<TelegramUserInChat>,
            chat_points: LookupMap<TelegramChatId, RewardPoint>,
            whitelisted_tokens: LookupSet<TokenAccountId>,
            version: u16,
            withdraw_available: bool,
            tip_available: bool,
            generic_tips_available: bool,

            telegram_tips_v1: HashMap<String, Balance>,
        }

        let old_contract: OldContract = env::state_read().expect("Old state doesn't exist");

        let near_account_id: TokenAccountId = NEAR.to_string();
        let mut telegram_tips_new =
            if iteration == 0 {
                LookupMap::new(StorageKey::TelegramTipsLookupMap)
            } else {
                old_contract.telegram_tips
            };
        let mut telegram_tips_v1 = old_contract.telegram_tips_v1.clone();
        let mut x: u32 = 1;

        for (telegram_account, amount) in &old_contract.telegram_tips_v1 {
            let telegram_id = telegram_account.parse::<u64>().unwrap_or(0);
            if telegram_id > 0 {
                telegram_tips_new.insert(
                    &TokenByTelegramAccount {
                        telegram_account: telegram_id,
                        token_account_id: near_account_id.clone(),
                    },
                    amount);

                telegram_tips_v1.remove(telegram_account);

                if x >= 250 {
                    break;
                }

                x += 1;
            } else {
                env::log(format!("Invalid telegram_account {}", telegram_account).as_bytes());
            }
        }

        env::log(format!("Pending items: {}", telegram_tips_v1.len()).as_bytes());


        Self {
            deposits: old_contract.deposits,
            telegram_tips: telegram_tips_new,
            tips: old_contract.tips,
            telegram_users_in_chats: old_contract.telegram_users_in_chats,
            chat_points: old_contract.chat_points,
            whitelisted_tokens: old_contract.whitelisted_tokens,
            version: migration_version,
            withdraw_available: old_contract.withdraw_available,
            tip_available: old_contract.tip_available,
            generic_tips_available: false,

            telegram_tips_v1
        }
    }
    */

    #[init(ignore_state)]
    #[allow(dead_code)]
    pub fn migrate_state_4() -> Self { // RewardPoint type updated , total_chat_points added
        let migration_version: u16 = 4;
        assert_eq!(env::predecessor_account_id(), env::current_account_id(), "Private function");

        #[derive(BorshDeserialize)]
        struct OldContract {
            deposits: LookupMap<TokenByNearAccount, Balance>,
            telegram_tips: LookupMap<TokenByTelegramAccount, Balance>,
            tips: LookupMap<AccountId, Vec<Tip>>,
            telegram_users_in_chats: LookupSet<TelegramUserInChat>,
            chat_points: LookupMap<TokenByTelegramChat, RewardPoint>,
            whitelisted_tokens: LookupSet<TokenAccountId>,
            version: u16,
            withdraw_available: bool,
            tip_available: bool,
            generic_tips_available: bool,

            telegram_tips_v1: HashMap<String, Balance>,
        }

        let old_contract: OldContract = env::state_read().expect("Old state doesn't exist");

        Self {
            deposits: old_contract.deposits,
            telegram_tips: old_contract.telegram_tips,
            tips: old_contract.tips,
            telegram_users_in_chats: old_contract.telegram_users_in_chats,
            chat_points: LookupMap::new(StorageKey::ChatPointsLookupMapU128),
            whitelisted_tokens: old_contract.whitelisted_tokens,
            version: migration_version,
            withdraw_available: old_contract.withdraw_available,
            tip_available: old_contract.tip_available,
            generic_tips_available: false,

            telegram_tips_v1: old_contract.telegram_tips_v1,
            total_chat_points: 0,
            chat_settings: LookupMap::new(StorageKey::ChatSettingsLookupMap),
            treasure: LookupMap::new(StorageKey::TreasureLookupMap),
        }
    }

    #[allow(unused_variables)]
    pub fn ft_on_transfer(
        &mut self,
        sender_id: ValidAccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let token_account_id = Some(env::predecessor_account_id());
        self.assert_check_whitelisted_token(&token_account_id);

        self.deposit_amount_to_account(sender_id.as_ref(), amount.0, token_account_id);

        PromiseOrValue::Value(0.into())
    }
}

/*
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
*/