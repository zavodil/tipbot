use crate::*;

#[derive(BorshSerialize, BorshDeserialize)]
pub struct WhitelistedToken {
    pub tips_available: bool,
    pub min_deposit: Balance,
    pub min_tip: Balance,
    pub withdraw_commission: Balance,
    pub dex: Option<DEX>,
    pub swap_contract_id: Option<AccountId>,
    pub swap_pool_id: Option<u32>,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum DEX {
    RefFinance
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum VWhitelistedToken {
    Current(WhitelistedToken),
}

impl From<VWhitelistedToken> for WhitelistedToken {
    fn from(v: VWhitelistedToken) -> Self {
        match v {
            VWhitelistedToken::Current(c) => c,
        }
    }
}

#[near_bindgen]
impl NearTips {
    pub fn whitelist_token(&mut self,
                           tips_available: bool,
                           token_id: TokenAccountId,
                           min_deposit: WrappedBalance,
                           min_tip: WrappedBalance,
                           withdraw_commission: WrappedBalance,
                           dex: Option<DEX>,
                           swap_contract_id: Option<AccountId>,
                           swap_pool_id: Option<u32>) {
        self.assert_owner();

        let token = WhitelistedToken {
            tips_available,
            min_deposit: min_deposit.0,
            min_tip: min_tip.0,
            withdraw_commission: withdraw_commission.0,
            dex,
            swap_contract_id,
            swap_pool_id,
        };

        self.whitelisted_tokens.insert(&token_id, &VWhitelistedToken::Current(token));
    }

    pub fn get_whitelisted_token(&self, token_id: TokenAccountId) -> WhitelistedTokenOutput {
        self.internal_get_whitelisted_token(&token_id)
    }

    pub fn is_whitelisted_tokens(&self, token_id: TokenAccountId) -> bool {
        self.check_whitelisted_token(&token_id)
    }
}


impl NearTips {
    pub(crate) fn get_token_min_deposit(&self, token_id: &TokenAccountId) -> Balance {
        WhitelistedToken::from(self.whitelisted_tokens.get(token_id).unwrap()).min_deposit
    }

    pub(crate) fn get_token_min_tip(&self, token_id: &TokenAccountId) -> Balance {
        WhitelistedToken::from(self.whitelisted_tokens.get(token_id).unwrap()).min_tip
    }

    pub(crate) fn get_withdraw_commission(&self, token_id: &TokenAccountId) -> Balance {
        WhitelistedToken::from(self.whitelisted_tokens.get(token_id).unwrap()).withdraw_commission
    }

    pub(crate) fn assert_token_tip_available(&self, token_id: &TokenAccountId) {
        let token: WhitelistedToken = self.whitelisted_tokens
            .get(token_id).
            expect("ERR_TOKEN_WAS_NOT_WHITELISTED")
            .into();
        assert!(token.tips_available, "ERR_TIPS_ARE_NOT_AVAILABLE");
    }

    pub(crate) fn assert_check_whitelisted_token(&self, token_id: &TokenAccountId) {
        assert!(self.check_whitelisted_token(token_id), "ERR_TOKEN_WAS_NOT_WHITELISTED");
    }

    pub(crate) fn check_whitelisted_token(&self, token_id: &TokenAccountId) -> bool {
        self.whitelisted_tokens.get(token_id).is_some()
    }

    pub(crate) fn get_swap_contract(&self, token_id: &TokenAccountId) -> (DEX, AccountId, u32) {
        let token: WhitelistedToken = self.whitelisted_tokens
            .get(token_id)
            .expect("ERR_TOKEN_WAS_NOT_WHITELISTED")
            .into();

        if let (Some(dex), Some(swap_contract_id), Some(swap_pool_id)) = (token.dex, token.swap_contract_id, token.swap_pool_id) {
            return (dex, swap_contract_id, swap_pool_id);
        } else {
            env::panic_str("ERR_SWAP_IS_NOT_ALLOWED")
        }
    }

    pub(crate) fn get_swap_contract_id(&self, token_id: &TokenAccountId) -> AccountId {
        let token: WhitelistedToken = self.whitelisted_tokens
            .get(token_id)
            .expect("Token wasn't whitelisted")
            .into();

        token.swap_contract_id.expect("ERR_SWAP_IS_NOT_ALLOWED")
    }

    pub fn internal_get_whitelisted_token(&self, token_id: &TokenAccountId) -> WhitelistedTokenOutput {
        let token: WhitelistedToken = self.whitelisted_tokens.get(token_id).expect("ERR_TOKEN_NOT_FOUND").into();
        token.into()
    }
}

pub fn get_token_name(token: &TokenAccountId) -> String {
    if let Some(token) = token {
        token.to_string()
    } else {
        "NEAR".to_string()
    }
}

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct WhitelistedTokenOutput {
    pub min_deposit: WrappedBalance,
    pub min_tip: WrappedBalance,
    pub withdraw_commission: WrappedBalance,
    pub dex: Option<DEX>,
    pub swap_contract_id: Option<AccountId>,
    pub swap_pool_id: Option<u32>,
}

impl From<WhitelistedToken> for WhitelistedTokenOutput {
    fn from(token: WhitelistedToken) -> Self {
        Self {
            min_deposit: token.min_deposit.into(),
            min_tip: token.min_tip.into(),
            withdraw_commission: token.withdraw_commission.into(),
            dex: token.dex,
            swap_contract_id: token.swap_contract_id,
            swap_pool_id: token.swap_pool_id,
        }
    }
}