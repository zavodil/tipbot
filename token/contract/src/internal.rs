use near_contract_standards::fungible_token::events::FtTransfer;
use crate::*;

impl Contract {
    pub(crate) fn internal_deposit(&mut self, account_id: &AccountId, amount: Balance) {
        let balance = self.internal_unwrap_balance_of(account_id);
        if let Some(new_balance) = balance.checked_add(amount) {
            self.token.accounts.insert(account_id, &new_balance);
            self.token.total_supply = self
                .token
                .total_supply
                .checked_add(amount)
                .unwrap_or_else(|| env::panic_str("Total supply overflow"));
        } else {
            env::panic_str("Balance overflow");
        }
    }

    pub(crate) fn internal_withdraw(&mut self, account_id: &AccountId, amount: Balance) {
        let balance = self.internal_unwrap_balance_of(account_id);
        if let Some(new_balance) = balance.checked_sub(amount) {
            self.token.accounts.insert(account_id, &new_balance);
            self.token.total_supply = self
                .token
                .total_supply
                .checked_sub(amount)
                .unwrap_or_else(|| env::panic_str("Total supply overflow"));
        } else {
            env::panic_str("The account doesn't have enough balance");
        }
    }

    pub(crate) fn internal_transfer(
        &mut self,
        sender_id: &AccountId,
        receiver_id: &AccountId,
        amount: Balance,
        memo: Option<String>,
    ) {
        require!(sender_id != receiver_id, "Sender and receiver should be different");
        require!(amount > 0, "The amount should be a positive number");
        self.internal_withdraw(sender_id, amount);
        self.internal_deposit(receiver_id, amount);
        FtTransfer {
            old_owner_id: sender_id,
            new_owner_id: receiver_id,
            amount: &U128(amount),
            memo: memo.as_deref(),
        }
            .emit();
    }
}
