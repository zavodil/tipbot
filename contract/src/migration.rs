use std::collections::HashMap;
use near_sdk::collections::LookupSet;
use crate::*;

const NEARV2: &str = "near";

pub type TokenAccountIdV2 = String;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct TokenByNearAccountV2 {
    pub account_id: AccountId,
    pub token_account_id: TokenAccountIdV2,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct TokenByTelegramAccount {
    pub telegram_account: TelegramAccountId,
    pub token_account_id: TokenAccountIdV2,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Tip {
    pub contact: Contact,
    pub amount: Balance,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Contact {
    pub category: ContactCategories,
    pub value: String,
    pub account_id: Option<TelegramAccountId>,
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

pub type TelegramAccountId = u64;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct TelegramUserInChat {
    pub telegram_id: TelegramAccountId,
    pub chat_id: TelegramChatId, // chat_id is negative, so don't forget * -1
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct NearTelegramAccount {
    pub telegram_id: TelegramAccountId,
    pub account_id: AccountId,
}

pub type TelegramChatId = u64;
pub type RewardPoint = u32;

#[near_bindgen]
impl NearTips {
    #[init(ignore_state)]
    #[allow(dead_code)]
    #[private]
    pub fn migrate_state(config: Config) -> Self { // add telegram_users_in_chats, Migration to token balances / deposits
        let migration_version: u16 = 3;

        #[derive(BorshDeserialize)]
        struct OldContract {
            deposits: LookupMap<TokenByNearAccountV2, Balance>,
            telegram_tips: LookupMap<TokenByTelegramAccount, Balance>,
            tips: LookupMap<AccountId, Vec<Tip>>,
            telegram_users_in_chats: LookupSet<TelegramUserInChat>,
            chat_points: LookupMap<TelegramChatId, RewardPoint>,
            whitelisted_tokens: LookupSet<TokenAccountIdV2>,
            version: u16,
            withdraw_available: bool,
            tip_available: bool,
            generic_tips_available: bool,

            telegram_tips_v1: HashMap<String, Balance>, // empty, used for migration
        }

        let old_contract: OldContract = env::state_read().expect("Old state doesn't exist");

        Self {
            deposits: LookupMap::new(StorageKey::Deposits),
            unclaimed_tips: LookupMap::new(StorageKey::UnclaimedTips),
            service_accounts: LookupMap::new(StorageKey::ServiceAccounts),
            service_accounts_by_near_account: LookupMap::new(StorageKey::ServiceAccountsByNearAccount),
            whitelisted_tokens: UnorderedMap::new(StorageKey::WhitelistedTokens),
            treasury: LookupMap::new(StorageKey::Treasury),
            treasury_claimed: LookupMap::new(StorageKey::TreasuryClaimed),
            treasury_by_account: LookupMap::new(StorageKey::TreasuryByAccount),
            service_fees: LookupMap::new(StorageKey::ServiceFees),
            config: LazyOption::new(StorageKey::Config, Some(&config)),
            version: migration_version,

            deposits_v2: old_contract.deposits,
            telegram_tips_v2: old_contract.telegram_tips,
        }
    }

    // User transfers his NEAR deposit
    pub fn migrate_near_deposit(&mut self) {
        self.assert_withdraw_available();

        let account_id = env::predecessor_account_id();

        self.internal_migrate_near_deposit(account_id);
    }

    // Bulk transfer NEAR deposit
    pub fn migrate_bulk_near_deposit(&mut self, accounts: Vec<AccountId>) {
        self.assert_withdraw_available();
        self.assert_operator();

        for account_id in accounts {
            self.internal_migrate_near_deposit(account_id);
        }
    }

    pub fn get_balance_v2(&self,
                          telegram_account: TelegramAccountId,
                          token_id: Option<TokenAccountIdV2>,
    ) -> WrappedBalance {
        self.telegram_tips_v2.get(
            &TokenByTelegramAccount {
                telegram_account,
                token_account_id: unwrap_token_id_v2(token_id.clone()).1,
            }
        ).unwrap_or(0).into()
    }

    pub fn get_deposit_v2(&self,
                          account_id: AccountId,
                          token_id: Option<TokenAccountIdV2>,
    ) -> WrappedBalance {
        let key = TokenByNearAccountV2 {
            account_id: account_id.clone(),
            token_account_id: unwrap_token_id_v2(token_id).1,
        };

        self.deposits_v2.get(&key).unwrap_or_default().into()
    }

    pub fn migrate_tips_to_deposit(&mut self, accounts: Vec<NearTelegramAccount>, token_id: Option<TokenAccountIdV2>) {
        self.assert_operator();
        let (token_id_v3, token_id_v2) = unwrap_token_id_v2(token_id.clone());
        self.assert_withdraw_available();
        self.assert_check_whitelisted_token(&token_id_v3);

        for account in accounts {
            self.internal_migrate_tips_to_deposit(token_id_v3.clone(), token_id_v2.clone(), account.telegram_id, account.account_id, token_id.clone());
        }
    }

    pub fn migrate_import_accounts_pairs(&mut self, accounts: Vec<NearTelegramAccount>) {
        self.assert_operator();

        for account in accounts {
            let service_account = ServiceAccount {
                service: Service::Telegram,
                account_id: Some(account.telegram_id),
                account_name: None
            };
            service_account.verify();

            let account_id = account.account_id;

            let existing_service_with_same_type = self.get_service_accounts_by_service(account_id.clone(), service_account.service.clone());
            assert!(existing_service_with_same_type.is_none(), "ERR_THIS_SERVICE_ACCOUNT_TYPE_ALREADY_SET_FOR_CURRENT_USER");

            require!(self.service_accounts.get(&service_account).is_none(), "ERR_SERVICE_ACCOUNT_ALREADY_SET_BY_OTHER_USER");

            self.service_accounts.insert(&service_account, &account_id);

            let mut existing_service_accounts = self.internal_get_service_accounts_by_near_account(&account_id);
            existing_service_accounts.push(service_account.clone());
            self.internal_set_service_accounts_by_near_account(&account_id, &existing_service_accounts);

            events::emit::insert_service_account(&account_id, &service_account);
        }
    }

    // import accounts
    pub fn migrate_import_accounts(&mut self, accounts: Vec<AccountId>, service_accounts: Vec<ServiceAccount>) {
        self.assert_operator();

        let service_accounts_len = service_accounts.len();
        assert_eq!(accounts.len(), service_accounts_len);

        for index in 0..service_accounts_len {
            let service_account = service_accounts[index].clone();
            let account_id = accounts[index].clone();
            service_account.verify();

            let existing_service_with_same_type = self.get_service_accounts_by_service(account_id.clone(), service_account.service.clone());
            assert!(existing_service_with_same_type.is_none(), "ERR_THIS_SERVICE_ACCOUNT_TYPE_ALREADY_SET_FOR_CURRENT_USER");

            require!(self.service_accounts.get(&service_account).is_none(), "ERR_SERVICE_ACCOUNT_ALREADY_SET_BY_OTHER_USER");

            self.service_accounts.insert(&service_account, &account_id);

            let mut existing_service_accounts = self.internal_get_service_accounts_by_near_account(&account_id);
            existing_service_accounts.push(service_account.clone());
            self.internal_set_service_accounts_by_near_account(&account_id, &existing_service_accounts);

            events::emit::insert_service_account(&account_id, &service_account);
        }
    }
}

impl NearTips {
    fn internal_migrate_near_deposit(&mut self, account_id: AccountId) {
        let key = TokenByNearAccountV2 {
            account_id: account_id.clone(),
            token_account_id: NEARV2.to_string(),
        };

        let deposit = self.deposits_v2.get(&key).expect("ERR_DEPOSIT_NOT_FOUND");

        if deposit > 0 {
            self.deposits_v2.insert(&key, &0u128);

            self.increase_deposit(account_id.clone(), NEAR, deposit);
        }

        log!("Transfer {} NEAR for {}", deposit, account_id);
    }

    fn internal_migrate_tips_to_deposit(&mut self, token_id_v3: TokenAccountId, token_id_v2: TokenAccountIdV2, telegram_account: TelegramAccountId, account_id: AccountId, token_id: Option<TokenAccountIdV2>) {
        let balance: Balance = self.get_balance_v2(telegram_account.clone(), token_id).0;

        self.telegram_tips_v2.insert(&TokenByTelegramAccount {
            telegram_account: telegram_account.clone(),
            token_account_id: token_id_v2,
        }, &0);

        self.increase_deposit(account_id.clone(), token_id_v3.clone(), balance);

        log!("Deposited {} of {} for {} (telegram {})", balance, get_token_name(&token_id_v3), account_id, telegram_account);
    }
}

fn unwrap_token_id_v2(token_id: Option<TokenAccountIdV2>) -> (TokenAccountId, TokenAccountIdV2) {
    let token_id_v3 = if let Some(token_id) = token_id.clone() {
        Some(AccountId::new_unchecked(token_id))
    } else {
        NEAR
    };

    (token_id_v3, token_id.unwrap_or(NEARV2.to_string()))
}