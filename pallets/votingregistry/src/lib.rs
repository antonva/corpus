#![cfg_attr(not(feature = "std"), no_std)]

use corpus_traits::IdentityInterface;
pub use pallet::*;

/// Custom type to simplify Config specification.
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::traits::Currency;
	use frame_support::{
		dispatch::DispatchResult, fail, pallet_prelude::*, traits::ReservableCurrency,
	};
	use frame_system::pallet_prelude::*;

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Currency: ReservableCurrency<Self::AccountId>;

		/// The amount each registered voter has to have bonded to vote
		#[pallet::constant]
		type ReserveAmount: Get<BalanceOf<Self>>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// TODO: Figure out OptionQuery vs ValueQuery. The ReserveAmount traits
	// require a default value so we go for valuequery and handling the default
	// in code.
	#[pallet::storage]
	pub(super) type VotingRegistry<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, OptionQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		RegisteredToVote { who: T::AccountId },
		DeRegisteredToVote { who: T::AccountId },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// If an account tries to register while already existing in the registry.
		AlreadyRegistered,
		/// If an account tries to deregister while not existing in the registry.
		NotRegistered,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Register to vote.
		#[pallet::weight(1_000)]
		pub fn register(origin: OriginFor<T>) -> DispatchResult {
			// Transaction is signed
			let who = ensure_signed(origin)?;
			// Is the sender registered
			let is_registered = VotingRegistry::<T>::contains_key(&who);
			match is_registered {
				true => {
					fail!(Error::<T>::AlreadyRegistered)
				},
				false => {
					// Reserve amount
					T::Currency::reserve(&who, T::ReserveAmount::get())?;
					VotingRegistry::<T>::insert(&who, T::ReserveAmount::get());
					Self::deposit_event(Event::RegisteredToVote { who })
				},
			}
			Ok(())
		}

		/// Unbond the tokens behind your registration.
		#[pallet::weight(1_000)]
		pub fn deregister(origin: OriginFor<T>) -> DispatchResult {
			// Transaction is signed
			let who = ensure_signed(origin)?;
			// Is the sender registered
			let maybe_sender = VotingRegistry::<T>::take(&who);
			match maybe_sender {
				Some(amount) => {
					// Reserve amount.
					// NOTE: A runtime upgrade could change the reserve amount, therefore
					// the amount is kept in storage, where it can be fetched before removing
					// the entry.
					T::Currency::unreserve(&who, amount);
					Self::deposit_event(Event::DeRegisteredToVote { who })
				},
				None => fail!(Error::<T>::NotRegistered),
			}
			Ok(())
		}
	}
}

impl<T: Config> IdentityInterface<T::AccountId> for Pallet<T> {
	fn is_identified(who: &T::AccountId) -> bool {
		VotingRegistry::<T>::contains_key(who)
	}
}
