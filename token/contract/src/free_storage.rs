use near_sdk::json_types::{U128};
use near_contract_standards::storage_management::{StorageBalance, StorageBalanceBounds, StorageManagement};

use crate::*;

// https://github.com/Narwallets/pegged-near/blob/main/meta-token/src/empty_nep_145.rs
// --------------------------------------------------------------------------
// Storage Management (we chose not to require storage backup for this token)
// but ref.finance FE and the WEB wallet seems to be calling theses fns
// --------------------------------------------------------------------------
const EMPTY_STORAGE_BALANCE: StorageBalance = StorageBalance {
    total: U128 { 0: 0 },
    available: U128 { 0: 0 },
};

impl Contract {
    pub(crate) fn internal_unwrap_balance_of(&self, account_id: &AccountId) -> Balance {
        match self.token.accounts.get(&account_id) {
            Some(balance) => balance,
            // Q: This makes the contract vulnerable to the sybil attack on storage.
            // Since `ft_transfer` is cheaper than storage for 1 account, you can send
            // 1 token to a ton randomly generated accounts and it will require 125 bytes per
            // such account. So it would require 800 transactions to block 1 NEAR of the account.
            // R: making the user manage storage costs adds too much friction to account creation
            // it's better to impede sybil attacks by other means
            // there's a MIN_TRANSFER of 1/1000 to make sibyl attacks more expensive in terms of tokens
            None => 0,
        }
    }
}

#[near_bindgen]
impl StorageManagement for Contract {
    // `registration_only` doesn't affect the implementation for vanilla fungible token.
    #[payable]
    #[allow(unused_variables)]
    fn storage_deposit(&mut self, account_id: Option<AccountId>, registration_only: Option<bool>) -> StorageBalance {
        log!("Storage is free, attached tokens transferred back to sender");
        Promise::new(env::predecessor_account_id())
            .transfer(env::attached_deposit());

        EMPTY_STORAGE_BALANCE
    }

    /// * returns a `storage_balance` struct if `amount` is 0
    fn storage_withdraw(&mut self, amount: Option<U128>) -> StorageBalance {
        if let Some(amount) = amount {
            if amount.0 > 0 {
                panic!("The amount is greater than the available storage balance");
            }
        }
        StorageBalance {
            total: 0.into(),
            available: 0.into(),
        }
    }

    #[allow(unused_variables)]
    fn storage_unregister(&mut self, force: Option<bool>) -> bool {
        true
    }

    fn storage_balance_bounds(&self) -> StorageBalanceBounds {
        StorageBalanceBounds {
            min: U128 { 0: 0 },
            max: Some(U128 { 0: 0 }),
        }
    }

    #[allow(unused_variables)]
    fn storage_balance_of(&self, account_id: AccountId) -> Option<StorageBalance> {
        Some(EMPTY_STORAGE_BALANCE)
    }
}