use near_contract_standards::fungible_token::{
	core::FungibleTokenCore,
	metadata::{FungibleTokenMetadata, FungibleTokenMetadataProvider},
};
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LazyOption;
use near_sdk::json_types::{U128};
use near_sdk::{env, log, near_bindgen, AccountId, Balance, BorshStorageKey, PanicOnDefault, Promise, PromiseOrValue, assert_one_yocto, require, ext_contract, Gas};

mod free_storage;
mod core_impl;
mod internal;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
	operator_id: AccountId,
	token: FungibleToken,
	metadata: LazyOption<FungibleTokenMetadata>,
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
	FungibleToken,
	Metadata,
}

#[near_bindgen]
impl Contract {
	/// Initializes the contract with the given total supply owned by the given `owner_id` with
	/// the given fungible token metadata.
	#[init]
	pub fn new(operator_id: AccountId, total_supply: U128, metadata: FungibleTokenMetadata) -> Self {
		metadata.assert_valid();
		let mut this = Self {
			operator_id: operator_id.clone(),
			token: FungibleToken::new(StorageKey::FungibleToken),
			metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
		};
		this.token.internal_register_account(&operator_id);
		this.token.internal_deposit(&operator_id, total_supply.into());
		this
	}

	pub fn update_metadata(&mut self, metadata: FungibleTokenMetadata) {
		assert_eq!(env::predecessor_account_id(), self.operator_id, "ERR_NO_ACCESS");
		metadata.assert_valid();
		self.metadata.set(&metadata);
	}

	pub fn update_operator(&mut self, new_operator_id: AccountId) {
		assert_eq!(env::predecessor_account_id(), self.operator_id, "ERR_NO_ACCESS");
		self.operator_id = new_operator_id;
	}

	pub fn get_operator_id(&self) -> AccountId {
		return self.operator_id.clone();
	}
}

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
	fn ft_metadata(&self) -> FungibleTokenMetadata {
		self.metadata.get().unwrap()
	}
}

#[near_bindgen]
impl FungibleTokenResolver for Contract {
	/// Returns the amount of burned tokens in a corner case when the sender
	/// has deleted (unregistered) their account while the `ft_transfer_call` was still in flight.
	/// Returns (Used token amount, Burned token amount)
	#[private]
	fn ft_resolve_transfer(
		&mut self,
		sender_id: AccountId,
		receiver_id: AccountId,
		amount: U128,
	) -> U128 {
		log!("ft_resolve_transfer");
		self.token.internal_ft_resolve_transfer(&sender_id, receiver_id, amount).0.into()
	}
}

#[ext_contract(ext_ft_receiver)]
pub trait FungibleTokenReceiver {
	fn ft_on_transfer(
		&mut self,
		sender_id: AccountId,
		amount: U128,
		msg: String,
	) -> PromiseOrValue<U128>;
}

#[ext_contract(ext_self)]
trait FungibleTokenResolver {
	fn ft_resolve_transfer(
		&mut self,
		sender_id: AccountId,
		receiver_id: AccountId,
		amount: U128,
	) -> U128;
}