use crate::*;

use uint::construct_uint;
construct_uint! {
    /// 256-bit unsigned integer.
    pub struct U256(4);
}

/// Contract config
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Config {
    owner_id: AccountId,
    operator_id: AccountId,

    withdraw_available: bool,
    tip_available: bool,

    // part of every tip goes to treasury
    treasury_fee: FeeFraction,
    // part of treasury_fee goes to service
    service_fee: FeeFraction,

    // token account_id, tiptoken.near
    tiptoken_account_id: AccountId,

    // wnear contract
    wrap_near_contract_id: AccountId,
}

#[near_bindgen]
impl NearTips {
    pub fn update_config(&mut self, config: Config) {
        self.assert_owner();
        self.config = LazyOption::new(StorageKey::Config, Some(&config));
    }

    pub fn set_withdraw_available(&mut self, withdraw_available: bool) {
        self.assert_owner();
        let mut config = self.internal_config();
        config.withdraw_available = withdraw_available;
        self.update_config(config);
    }

    pub fn get_withdraw_available(self) -> bool {
        self.internal_config().withdraw_available
    }

    pub fn set_tip_available(&mut self, tip_available: bool) {
        self.assert_owner();
        let mut config = self.internal_config();
        config.tip_available = tip_available;
        self.update_config(config);
    }

    pub fn get_tip_available(self) -> bool {
        self.internal_config().tip_available
    }

    pub fn get_config(&self) -> Config {
        self.internal_config()
    }

    pub fn get_operator_id(&self) -> AccountId { self.internal_config().operator_id }
}

impl NearTips {
    pub fn internal_config(&self) -> Config {
        self.config.get().unwrap()
    }

    pub fn assert_owner(&self) {
        assert_eq!(
            &env::predecessor_account_id(),
            &self.internal_config().owner_id,
            "ERR_NOT_AN_OWNER"
        );
    }

    pub fn assert_operator(&self) {
        assert_eq!(
            &env::predecessor_account_id(),
            &self.internal_config().operator_id,
            "No access"
        );
    }

    pub(crate) fn assert_tip_available(&self) {
        assert!(&self.internal_config().tip_available, "Tips paused");
    }

    pub(crate) fn assert_withdraw_available(&self) {
        assert!(self.check_withdraw_available(), "Deposits/Withdrawals paused");
    }

    pub(crate) fn check_withdraw_available(&self) -> bool {
        self.internal_config().withdraw_available
    }

    pub(crate) fn get_treasure_fee_fraction(&self, tip: Balance) -> Balance {
        self.internal_config().treasury_fee.multiply(tip)
    }

    pub(crate) fn get_service_fee_fraction(&self, amount: Balance) -> Balance {
        self.internal_config().service_fee.multiply(amount)
    }

    pub(crate) fn get_tiptoken_account_id(&self) -> AccountId {
        self.internal_config().tiptoken_account_id
    }

    pub(crate) fn get_wrap_near_contract_id(&self) -> AccountId {
        self.internal_config().wrap_near_contract_id
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct FeeFraction {
    pub numerator: u32,
    pub denominator: u32,
}

impl FeeFraction {
    pub fn assert_valid(&self) {
        assert_ne!(self.denominator, 0, "Denominator must be a positive number");
        assert!(
            self.numerator <= self.denominator,
            "The treasure fee must be less or equal to 1"
        );
    }

    pub fn multiply(&self, value: Balance) -> Balance {
        (U256::from(self.numerator) * U256::from(value) / U256::from(self.denominator)).as_u128()
    }
}