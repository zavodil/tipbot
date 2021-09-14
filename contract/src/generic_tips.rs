use crate::*;

/* GENERIC TIPS, using tips object */
const UNDEFINED_ACCOUNT_ID: &str = "";

#[near_bindgen]
impl NearTips {
    pub(crate) fn assert_generic_tips_available(&self) {
        assert!(self.generic_tips_available, "Generic tips and withdrawals disabled");
    }

    pub fn set_generic_tips_available(&mut self, generic_tips_available: bool) {
        self.assert_master_account_id();
        self.generic_tips_available = generic_tips_available;
    }

    pub fn get_generic_tips_available(&self) -> bool {
        self.generic_tips_available
    }


    #[payable]
    // tip attached tokens without knowing NEAR account id
    pub fn tip_contact_with_attached_tokens(&mut self, contact: Contact) -> Promise {
        self.assert_tip_available();
        self.assert_generic_tips_available();

        let deposit: Balance = near_sdk::env::attached_deposit();

        let account_id = env::predecessor_account_id();

        self.get_contact_owner(contact.clone(), self.auth_account_id.to_string()).
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
        self.assert_withdraw_available();
        self.assert_generic_tips_available();

        let account_id = env::predecessor_account_id();

        self.get_contact_owner(contact.clone(), self.auth_account_id.to_string())
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
        self.assert_withdraw_available();
        self.assert_generic_tips_available();

        let account_id = env::predecessor_account_id();

        auth::get_contacts(account_id.clone(), &self.auth_account_id, NO_DEPOSIT, BASE_GAS)
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

    #[private]
    pub fn on_withdraw_tip(&mut self, account_id: AccountId, contact: Contact, balance: Balance) -> bool {

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

}