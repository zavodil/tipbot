use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{wee_alloc, env, near_bindgen, AccountId, Balance, Promise, Gas, ext_contract, PromiseResult, PromiseOrValue, PanicOnDefault, BorshStorageKey,
               log, assert_one_yocto};
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::collections::{LookupSet, LookupMap};
use std::collections::HashMap;
use std::convert::TryFrom;
use near_contract_standards::fungible_token::core_impl::ext_fungible_token;

use crate::internal::*;
pub use crate::generic_tips::*;
pub use crate::auth_tips::*;
pub use crate::tiptoken::*;
pub use crate::migration::*;

mod internal;
mod auth_tips;
mod generic_tips;
mod tiptoken;
mod migration;

// TelegramAccountId may potentially overflow the u64 limit
pub type TelegramAccountId = u64;
pub type WrappedBalance = U128;
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


const MASTER_ACCOUNT_ID: &str = "zavodil.testnet";
const LINKDROP_ACCOUNT_ID: &str = "linkdrop.zavodil.testnet";
const AUTH_ACCOUNT_ID: &str = "dev-1625611642901-32969379055293";
*/

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

    // should be removed, bad approach
    tips: LookupMap<AccountId, Vec<Tip>>,
    telegram_users_in_chats: LookupSet<TelegramUserInChat>,

    chat_points_v1: LookupMap<TokenByTelegramChat, RewardPoint>,

    whitelisted_tokens: LookupSet<TokenAccountId>,
    version: u16,
    withdraw_available: bool,
    tip_available: bool,
    generic_tips_available: bool,

    // empty, used for migration
    telegram_tips_v1: HashMap<String, Balance>,

    //total_chat_points: RewardPoint,
    // not used, TODO for each token?
    chat_settings: LookupMap<TelegramChatId, ChatSettings>,
    treasure: LookupMap<TokenAccountId, Balance>,
    chat_points: LookupMap<TelegramChatId, RewardPoint>,

    user_tokens_to_claim: LookupMap<TokenByNearAccount, Balance>,
    master_account_id: AccountId,
    linkdrop_account_id: AccountId,
    auth_account_id: AccountId,
    tiptoken_account_id: TokenAccountId,
    total_tiptokens: Balance,
    tiptokens_burned: Balance,
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
    pub track_chat_points: bool,
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
    fn after_ft_transfer_claim_by_chat(&mut self, chat_id: TelegramChatId, amount_claimed: WrappedBalance, token_account_id: TokenAccountId) -> bool;
    fn after_ft_transfer_claim_tiptokens(&mut self, account_id: AccountId, amount_redeemed: WrappedBalance, token_account_id: TokenAccountId) -> bool;
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
    TreasureLookupMap,
    ChatTokensLookupMap,
    UserTokensToClaimLookupMap,
}

#[near_bindgen]
impl NearTips {
    #[init]
    pub fn new(master_account_id: ValidAccountId,
               linkdrop_account_id: ValidAccountId,
               auth_account_id: ValidAccountId,
               tiptoken_account_id: ValidAccountId) -> Self {
        Self {
            deposits: LookupMap::new(StorageKey::TelegramDepositsLookupMap),
            telegram_tips: LookupMap::new(StorageKey::TelegramTipsLookupMap), // first object only for telegram tips
            tips: LookupMap::new(StorageKey::TipsLookupMap), // generic object for any tips
            telegram_users_in_chats: LookupSet::new(StorageKey::TelegramUsersInChats),
            chat_points_v1: LookupMap::new(StorageKey::ChatPointsLookupMap),
            //chat_tokens: LookupMap::new(StorageKey::ChatTokensLookupMap),
            whitelisted_tokens: LookupSet::new(StorageKey::WhitelistedTokensLookupSet),
            version: 0,
            withdraw_available: true,
            tip_available: true,
            generic_tips_available: false,
            telegram_tips_v1: HashMap::new(),
            //total_chat_points: 0,
            chat_settings: LookupMap::new(StorageKey::ChatSettingsLookupMap),
            treasure: LookupMap::new(StorageKey::TreasureLookupMap),

            chat_points: LookupMap::new(StorageKey::ChatPointsLookupMap), // fix storage
            user_tokens_to_claim: LookupMap::new(StorageKey::UserTokensToClaimLookupMap),
            master_account_id: master_account_id.into(),
            linkdrop_account_id:linkdrop_account_id.into(),
            auth_account_id: auth_account_id.into(),
            tiptoken_account_id: tiptoken_account_id.into(),
            total_tiptokens: 0,
            tiptokens_burned: 0
        }
    }

    #[payable]
    /* DEPOSIT */
    pub fn deposit(&mut self, account_id: Option<ValidAccountId>) {
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

        self.increase_deposit(account_id.clone(), token_id_unwrapped.clone(), amount);

        env::log(format!("@{} deposited {} of {:?}", account_id, amount, token_id_unwrapped).as_bytes());
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

    pub fn transfer_tips_to_deposit(&mut self, telegram_account: TelegramAccountId,
                                    account_id: ValidAccountId,
                                    token_id: Option<TokenAccountId>) {
        self.assert_withdraw_available();
        self.assert_check_whitelisted_token(&token_id);
        self.assert_master_account_id();

        let balance: Balance = self.get_balance(telegram_account, token_id.clone()).0;

        let amount: Balance;

        let token_id_unwrapped = NearTips::unwrap_token_id(&token_id);
        if token_id_unwrapped == NEAR { // TODO commission for DAI withdrawals?
            assert!(balance > WITHDRAW_COMMISSION, "Not enough tokens to pay transfer commission");
            amount = balance - WITHDRAW_COMMISSION;
            Promise::new(self.master_account_id.clone()).transfer(WITHDRAW_COMMISSION);
        } else {
            amount = balance;
        }

        self.increase_deposit(account_id.clone().into(), token_id_unwrapped.clone(), amount);
        self.set_balance_to_zero(telegram_account, token_id_unwrapped.clone());

        env::log(format!("@{} transfer {} of {:?} from telegram account {}. Transfer commission: {} yNEAR",
                         account_id, balance, token_id_unwrapped, telegram_account, WITHDRAW_COMMISSION).as_bytes());
    }

    /* SEND TIPS */
    pub fn send_tip_to_telegram(&mut self,
                                telegram_account: TelegramAccountId,
                                amount: WrappedBalance,
                                chat_id: Option<TelegramChatId>,
                                token_id: Option<TokenAccountId>) {
        let account_id = env::predecessor_account_id();
        self.send_tip_to_telegram_from_account(account_id, telegram_account, amount, chat_id, token_id);
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

        let token_id_unwrapped = NearTips::unwrap_token_id(&token_id);
        let deposit = self.get_deposit_for_account_id_and_token_id(&sender_account_id, &token_id_unwrapped);

        assert!(amount.0 <= deposit, "Not enough tokens deposited to tip (Deposit: {}. Requested: {})", deposit, amount.0);

        let mut tip_amount: Balance = amount.0;

        // treasure fee & points
        if let Some(chat_id_unwrapped) = chat_id {
            if chat_id_unwrapped > 0 {
                let chat_settings = self.get_chat_settings(chat_id_unwrapped);

                if let Some(chat_settings_unwrapped) = chat_settings {
                    let treasure_fee_numerator = chat_settings_unwrapped.treasure_fee_numerator;
                    NearTips::assert_valid_treasure_fee_numerator(treasure_fee_numerator);

                    if chat_settings_unwrapped.track_chat_points {
                        if amount.0 > MIN_AMOUNT_TO_REWARD_CHAT {
                            let user_in_chat: TelegramUserInChat = TelegramUserInChat {
                                telegram_id: telegram_account,
                                chat_id: chat_id_unwrapped,
                            };

                            if !self.telegram_users_in_chats.contains(&user_in_chat) {
                                let chat_score: RewardPoint = self.chat_points.get(&chat_id_unwrapped).unwrap_or(0);
                                let new_score = chat_score + 1;
                                self.chat_points.insert(&chat_id_unwrapped, &new_score);
                                self.telegram_users_in_chats.insert(&user_in_chat);
                                env::log(format!("Reward point for chat {} added. Total: {}", chat_id_unwrapped, new_score).as_bytes());
                            }
                        }
                    }

                    if treasure_fee_numerator > 0 {
                        let treasure_fee = NearTips::get_treasure_fee(amount.0, treasure_fee_numerator);
                        tip_amount = amount.0 - treasure_fee; // overwrite tip amount

                        if treasure_fee > 0 {
                            self.distribute_tiptokens(chat_id_unwrapped, chat_settings_unwrapped.admin_account_id, token_id_unwrapped.clone(), treasure_fee, sender_account_id.clone());
                        }
                        env::log(format!("@{} tipped {} of {:?} for telegram account {}.",
                                         sender_account_id, tip_amount, token_id_unwrapped, telegram_account).as_bytes());
                    }
                }
            }
        } else {
            env::log(format!("@{} tipped {} of {:?} for telegram account {}", sender_account_id, tip_amount, token_id_unwrapped, telegram_account).as_bytes());
        }

        // perform a tip
        self.increase_balance(telegram_account, token_id_unwrapped.clone(), tip_amount);

        self.deposits.insert( // don't use helper to avoid second deposit check
                              &TokenByNearAccount {
                                  account_id: sender_account_id.clone(),
                                  token_account_id: token_id_unwrapped.clone(),
                              },
                              &(deposit - amount.0));
    }

    /* WITHDRAW */

    // centralized tips withdraw, with master_account authorisation
    pub fn withdraw_from_telegram(&mut self,
                                  telegram_account: TelegramAccountId,
                                  account_id: ValidAccountId,
                                  token_id: Option<TokenAccountId>) -> Promise { // TODO FT ft_on_transfer
        self.assert_withdraw_available();
        self.assert_master_account_id();

        let balance: Balance = self.get_balance(telegram_account, token_id.clone()).0;

        let token_id_unwrapped = NearTips::unwrap_token_id(&token_id);

        let amount: Balance;

        if token_id_unwrapped == NEAR {
            assert!(balance > WITHDRAW_COMMISSION, "Not enough tokens to pay withdraw commission");
            amount = balance - WITHDRAW_COMMISSION;
            Promise::new(self.master_account_id.to_string()).transfer(WITHDRAW_COMMISSION);
        } else {  // TODO COMMISSION IN NEAR?
            amount = balance;
        }

        self.set_balance_to_zero(telegram_account, token_id_unwrapped.clone());

        env::log(format!("@{} is withdrawing {} of {:?} from telegram account {}",
                         account_id, amount, token_id_unwrapped, telegram_account).as_bytes());

        if token_id_unwrapped == NEAR {
            Promise::new(account_id.into()).transfer(amount)
        } else {
            ext_fungible_token::ft_transfer(
                account_id.into(),
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
            log!("Token {} withdraw by telegram account {} failed. Amount to recharge: {}",
                 token_account_id, telegram_account, amount.0);

            self.increase_balance(telegram_account, token_account_id, amount.0);
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

            self.increase_deposit(account_id, token_account_id, amount.0);
        }
        promise_success
    }

    // withdraw from deposit
    pub fn withdraw(&mut self, token_id: Option<TokenAccountId>) -> Promise {
        self.assert_withdraw_available();
        self.assert_check_whitelisted_token(&token_id);

        let account_id = env::predecessor_account_id();
        let account_id_prepared: ValidAccountId = ValidAccountId::try_from(account_id.clone()).unwrap();
        let deposit: Balance = self.get_deposit(account_id_prepared, token_id.clone()).0;

        assert!(deposit > 0, "Missing deposit");

        let token_id_unwrapped = NearTips::unwrap_token_id(&token_id);

        self.set_deposit_to_zero(account_id.clone(), token_id_unwrapped.clone());

        env::log(format!("@{} withdrew {} of {:?} from internal deposit", account_id, deposit, token_id_unwrapped).as_bytes());

        if token_id_unwrapped == NEAR {
            Promise::new(account_id).transfer(deposit)
        } else {
            ext_fungible_token::ft_transfer(
                account_id.clone(),
                deposit.into(),
                Some(format!("Claiming tips: {} of {:?} from @{}", deposit, token_id_unwrapped, env::current_account_id())),
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
        self.assert_master_account_id();
        // TODO Linkdrop for NOT NEAR???
        let balance: Balance = NearTips::get_balance(self, telegram_account, Some(NEAR.to_string())).0;
        assert!(balance > WITHDRAW_COMMISSION + ACCESS_KEY_ALLOWANCE, "Not enough tokens to pay for key allowance and withdraw commission");

        let amount = balance - WITHDRAW_COMMISSION;

        self.set_balance_to_zero(telegram_account, NEAR.to_string());

        Promise::new(self.master_account_id.to_string()).transfer(WITHDRAW_COMMISSION);

        env::log(format!("Telegram account {} withdrew {} yNEAR with linkDrop for public key {}. Withdraw commission: {} yNEAR",
                         telegram_account, amount, public_key, WITHDRAW_COMMISSION).as_bytes());

        linkdrop::send(public_key, &self.linkdrop_account_id, amount, BASE_GAS)
    }


    /* VIEW METHODS */
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


    pub fn whitelist_token(&mut self, token_id: TokenAccountId) {
        self.assert_master_account_id();

        self.whitelisted_tokens.insert(&token_id);
    }

    pub fn is_whitelisted_token(&self, token_id: TokenAccountId) -> bool {
        self.whitelisted_tokens.contains(&token_id)
    }

    pub(crate) fn unwrap_token_id(token_id: &Option<TokenAccountId>) -> TokenAccountId {
        token_id.clone().unwrap_or_else(|| NEAR.to_string())
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
        self.assert_master_account_id();
        self.withdraw_available = withdraw_available;
    }

    pub fn get_withdraw_available(&self) -> bool {
        self.withdraw_available
    }

    pub fn set_tip_available(&mut self, tip_available: bool) {
        self.assert_master_account_id();
        self.tip_available = tip_available;
    }

    pub fn get_tip_available(&self) -> bool {
        self.tip_available
    }

    pub fn assert_master_account_id(&self) {
        assert_eq!(env::predecessor_account_id(), self.master_account_id, "No access");
    }

    pub fn get_version(self) -> u16 {
        self.version
    }
}