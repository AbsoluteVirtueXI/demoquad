#![cfg_attr(not(feature = "std"), no_std)]
pub trait Identifiable<AccountId, IdentityDetails> {
	fn is_identified(caller: &AccountId) -> bool;

	fn set_identity(who: &AccountId, identity_details: IdentityDetails);

	fn get_identity(who: &AccountId) -> Option<IdentityDetails>;
}
