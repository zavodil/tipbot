use crate::*;

pub mod emit {
    use super::*;
    use near_sdk::serde_json::json;

    fn log_event<T: Serialize>(event: &str, data: T) {
        let event = json!({
            "standard": "tipbot",
            "version": "1.0.0",
            "event": event,
            "data": [data]
        });

        log!("EVENT_JSON:{}", event.to_string());
    }

    pub fn insert_service_account(account_id: &AccountId, service_account: &ServiceAccount) {
        log_event(
            "insert_service_account",
            json!({
                "account_id": account_id,
                "service_account": service_account,
            }),
        );
    }

    pub fn remove_service_account(account_id: &AccountId, service_account: &ServiceAccount) {
        log_event(
            "remove_service_account",
            json!({
                "account_id": account_id,
                "service_account": service_account,
            }),
        );
    }

    pub fn increase_balance(sender_account_id: &AccountId,
                            receiver_account_id: &Option<AccountId>,
                            receiver_service_account: &Option<ServiceAccount>,
                            amount: Balance,
                            token_id: &TokenAccountId) {
        log_event(
            "increase_deposit",
            json!({
                "sender_account_id": sender_account_id,
                "receiver_near_account": receiver_account_id,
                "receiver_service_account": receiver_service_account,
                "amount": U128::from(amount),
                "token_id": get_token_name(&token_id)
            }),
        );
    }

    pub fn deposit(account_id: &AccountId,
                    amount: Balance,
                    token_id: &TokenAccountId) {
        log_event(
            "deposit",
            json!({
                "account_id": account_id,
                "amount": U128::from(amount),
                "token_id": get_token_name(&token_id)
            }),
        );
    }

    pub fn withdraw(account_id: &AccountId,
                            amount: Balance,
                            token_id: &TokenAccountId) {
        log_event(
            "withdraw",
            json!({
                "account_id": account_id,
                "amount": U128::from(amount),
                "token_id": get_token_name(&token_id)
            }),
        );
    }

    pub fn withdraw_from_service_account(account_id: &AccountId,
                                         service_account: &ServiceAccount,
                    amount: Balance,
                    token_id: &TokenAccountId) {
        log_event(
            "withdraw_from_service_account",
            json!({
                "account_id": account_id,
                "service_account": service_account,
                "amount": U128::from(amount),
                "token_id": get_token_name(&token_id)
            }),
        );
    }



    pub fn service_fees_add(amount: Balance, token_id: &TokenAccountId) {
        log_event(
            "service_fees_add",
            json!({
                "amount": U128::from(amount),
                "token_id": get_token_name(&token_id)
            }),
        );
    }

    pub fn service_fees_remove(amount: Balance, token_id: &TokenAccountId) {
        log_event(
            "service_fees_remove",
            json!({
                "amount": U128::from(amount),
                "token_id": get_token_name(&token_id)
            }),
        );
    }

    pub fn treasury_add(amount: Balance, token_id: &TokenAccountId, account_id: &AccountId) {
        log_event(
            "treasury_add",
            json!({
                "amount": U128::from(amount),
                "token_id": get_token_name(&token_id),
                "account_id": account_id
            }),
        );
    }

    pub fn treasury_remove(amount: Balance, token_id: &TokenAccountId, account_id: &AccountId) {
        log_event(
            "treasury_remove",
            json!({
                "amount": U128::from(amount),
                "token_id": get_token_name(&token_id),
                "account_id": account_id
            }),
        );
    }

}