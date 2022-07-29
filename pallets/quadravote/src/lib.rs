#![cfg_attr(not(feature = "std"), no_std)]

/// Quadravote - the quadratic voting pallet.
/// Quadratic voting is the concept of TODO... <waffle here> <link here>
///
/// The implementation splits time up in two periods. First, a proposal period
/// where identified account holders can bring up a proposal for voting.
/// Then a voting period where identified account holders can reserve an amount
/// of their tokens to vote on a particular subject.
///
/// A proposer can withdraw their proposal if it is still the same proposal round
///
/// Definitions:
///
/// An identified account holder is an account holder that has registered an identity
/// via the `pallet-identity` pallet Identity module.
///
/// A vote is the square root of the amount of tokens reserved.
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		dispatch::{DispatchResult, DispatchResultWithPostInfo},
		pallet_prelude::*,
		traits::ReservableCurrency,
	};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::AtLeast32BitUnsigned;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: ReservableCurrency<Self::AccountId>;
		type ProposalRound: Parameter + AtLeast32BitUnsigned;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
		// parameters: []
		VoteRegistered(u32, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		// We're not in the proposal period.
		NotInProposalPeriod,

		// We're not in the voting period.
		NotInVotingPeriod,

		// Account holder is not Identified.
		NotIdentified,

		// Voter does not have enough funds for this vote.
		NotEnoughFunds,

		// Proposal already exists when trying to add a new one.
		ProposalAlreadyExists,

		// Proposal doesn't exist, when trying to withdraw an existing one.
		ProposalDoesNotExist,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// Create a proposal that can be voted on
		#[pallet::weight(1_000)]
		pub fn create_proposal(origin: OriginFor<T>, proposal: u8) -> DispatchResult {
			Ok(())
		}

		// Withdraw a proposal if still within the same voting period.
		// This should probably cost more to stop people from wasting others' time.
		#[pallet::weight(10_000)]
		pub fn withdraw_proposal(origin: OriginFor<T>, proposal: u8) -> DispatchResult {
			Ok(())
		}

		// Cast a vote on an existing proposal.
		#[pallet::weight(1_000)]
		pub fn cast_vote(origin: OriginFor<T>, proposal: u8, amount: u32) -> DispatchResult {
			Ok(())
		}
	}
}
