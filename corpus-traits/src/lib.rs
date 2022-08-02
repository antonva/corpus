#![cfg_attr(not(feature = "std"), no_std)]

pub trait IdentityInterface<AccountId> {
	fn is_identified(who: &AccountId) -> bool;
}
