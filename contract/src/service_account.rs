use std::fmt;
use crate::*;

/// Don't change the order of existing records
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum Service {
    Telegram,
    Twitter,
    Discord,
    Github,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct ServiceAccount {
    pub service: Service,
    pub account_id: Option<ServiceAccountId>,
    pub account_name: Option<String>,
}

impl ServiceAccount {
    pub fn verify(&self) {
        match self.service {
            Service::Telegram => assert!(self.account_id.is_some() && self.account_name.is_none(), "ERR_ILLEGAL_TELEGRAM_ACCOUNT"),
            Service::Twitter => assert!(self.account_id.is_none() && self.account_name.is_some(), "ERR_ILLEGAL_TWITTER_ACCOUNT"),
            Service::Discord => assert!(self.account_id.is_none() && self.account_name.is_some(), "ERR_ILLEGAL_DISCORD_ACCOUNT"),
            Service::Github => assert!(self.account_id.is_none() && self.account_name.is_some(), "ERR_ILLEGAL_GITHUB_ACCOUNT"),
        }
    }
}

impl fmt::Display for ServiceAccount {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.service {
            Service::Telegram => write!(f, "{{\"telegram\": \"{}\"}}", self.account_id.unwrap()),
            Service::Twitter => write!(f, "{{\"twitter\": \"{}\"}}", self.account_name.clone().unwrap()),
            Service::Discord => write!(f, "{{\"discord\": \"{}\"}}", self.account_name.clone().unwrap()),
            Service::Github => write!(f, "{{\"github\": \"{}\"}}", self.account_name.clone().unwrap()),
            /*
             Service::Telegram => write!(f, "[Telegram/{}]", self.account_id.unwrap()),
            Service::Twitter => write!(f, "[Twitter/{}]", self.account_name.clone().unwrap()),
            Service::Discord => write!(f, "[Discord/{}]", self.account_name.clone().unwrap()),
            Service::Github => write!(f, "[Github/{}]", self.account_name.clone().unwrap()),
             */
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct TokenByServiceAccount {
    pub account: ServiceAccount,
    pub token_id: TokenAccountId,
}

#[near_bindgen]
impl NearTips {
    pub fn insert_service_account_to_near_account(&mut self, account_id: AccountId, service_account: ServiceAccount) {
        self.assert_operator();
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

    pub fn remove_service_account_from_near_account(&mut self, account_id: AccountId, service_account: ServiceAccount) {
        self.assert_operator();
        service_account.verify();

        let existing_service_with_same_type = self.get_service_accounts_by_service(account_id.clone(), service_account.service.clone());
        assert!(existing_service_with_same_type.is_some(), "ERR_SERVICE_ACCOUNT_NOT_SET");

        self.service_accounts.remove(&service_account);

        events::emit::remove_service_account(&account_id, &service_account);

        let other_service_accounts = self.get_service_accounts_except_service(account_id.clone(), service_account.service);
        self.internal_set_service_accounts_by_near_account(&account_id, &other_service_accounts);
    }
    /*
        pub fn insert_near_account_to_service_account(&mut self, account_id: AccountId, service_account: ServiceAccount) {
            self.assert_operator();

            let service_account_owner = self.internal_get_near_account_by_service_account(&service_account);
            assert!(service_account_owner.is_none(), "ERR_SERVICE_ACCOUNT_ALREADY_HAS_NEAR_ACCOUNT");

            self.service_accounts.insert(&service_account, &account_id);

            let mut existing_service_accounts = self.internal_get_service_accounts_by_near_account(&account_id);
            existing_service_accounts.push(service_account);
            self.internal_set_service_accounts_by_near_account(&account_id, &existing_service_accounts);
        }

        pub fn remove_near_account_from_service_account(&mut self, account_id: AccountId, service_account: ServiceAccount) {
            self.assert_operator();

            let service_account_owner = self.internal_get_near_account_by_service_account(&service_account);
            assert!(service_account_owner.is_none(), "ERR_SERVICE_ACCOUNT_HAS_NO_NEAR_ACCOUNT");

            self.service_accounts.remove(&service_account);

            let other_service_accounts = self.get_service_accounts_except_service(account_id.clone(), service_account.service);
            self.internal_set_service_accounts_by_near_account(&account_id, &other_service_accounts);
        }
    */
    pub fn get_service_accounts_by_near_account(&self, account_id: AccountId) -> Vec<ServiceAccount> {
        self.internal_get_service_accounts_by_near_account(&account_id)
    }

    pub fn get_service_accounts_by_service(&self, account_id: AccountId, service: Service) -> Option<ServiceAccount> {
        let all_service_accounts = self.internal_get_service_accounts_by_near_account(&account_id);

        all_service_accounts
            .into_iter()
            .find(|account| account.service == service)
    }

    pub fn get_service_accounts_except_service(&self, account_id: AccountId, service: Service) -> Vec<ServiceAccount> {
        let all_service_accounts = self.internal_get_service_accounts_by_near_account(&account_id);

        all_service_accounts
            .into_iter()
            .filter(|account| account.service != service)
            .collect()
    }

    pub fn get_near_account_by_service_account(&self, service_account: ServiceAccount) -> Option<AccountId> {
        self.internal_get_near_account_by_service_account(&service_account)
    }
}

impl NearTips {
    pub fn internal_get_service_accounts_by_near_account(&self, account_id: &AccountId) -> Vec<ServiceAccount> {
        self.service_accounts_by_near_account.get(account_id).unwrap_or_default()
    }

    pub fn internal_set_service_accounts_by_near_account(&mut self, account_id: &AccountId, service_accounts: &Vec<ServiceAccount>) {
        self.service_accounts_by_near_account.insert(account_id, service_accounts);
    }

    pub fn internal_get_near_account_by_service_account(&self, service_account: &ServiceAccount) -> Option<AccountId> {
        self.service_accounts.get(service_account)
    }
}