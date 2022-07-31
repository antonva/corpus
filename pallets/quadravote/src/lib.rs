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
		dispatch::DispatchResult, pallet_prelude::*, traits::ReservableCurrency, BoundedVec,
	};
	use frame_system::{ensure_signed, pallet_prelude::*};

	/// Configure the pallet by specifying the parameters and types on which it depends.
	/// TODO: We are tight coupling the identity pallet as it doesn't implement any reusable
	/// trait that we can use for loose coupling. If time allows, copy the identity pallet and
	/// implement loose coupling via a new trait/interface.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_identity::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: ReservableCurrency<Self::AccountId>;
		// How many blocks does each proposal take?
		#[pallet::constant]
		type ProposalPeriodLength: Get<u32>;
		// How many blocks does each voting period take.
		#[pallet::constant]
		type VotingPeriodLength: Get<u32>;
		// How many proposals can we have in a single round.
		#[pallet::constant]
		type MaxProposals: Get<u32>;
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

	/// Determines if we are in the proposal or voting period.
	/// We are using the unit expression to achieve a slightly
	/// more optimized way of storing a boolean.
	/// When 'false', the pallet is considered to be in the
	/// voting period and will allow votes to be cast on existing
	/// proposals. When 'true', the opposite is true and proposals
	/// can be made up until the time is up or the maximum threshold
	/// of proposals has been reached.
	#[pallet::storage]
	pub type ProposalPeriod<T> = StorageValue<_, ()>;

	/// This storage entry is useful for dispatching all of the proposals at
	/// the same time.
	#[pallet::storage]
	#[pallet::getter(fn get_all_proposals)]
	pub(super) type AllProposals<T: Config> =
		StorageValue<_, BoundedVec<u8, T::MaxProposals>, ValueQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// parameters: [proposal, who]
		VoteRegistered(u8, T::AccountId),
		// parameters: [proposals]
		ProposalsDispatched(BoundedVec<u8, T::MaxProposals>),
		// parameters: [proposals, votes_for, votes_against]
		VotingResultsDispatched(
			BoundedVec<u8, T::MaxProposals>,
			BoundedVec<u8, T::MaxProposals>,
			BoundedVec<u8, T::MaxProposals>,
		),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		// Extrinsic not signed
		ExtrinsicNotSigned,
		// Not in the proposal period
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
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(now: BlockNumberFor<T>) -> Weight {
			match ProposalPeriod::<T>::exists() {
				true => if now % <T as Config>::ProposalPeriodLength == 0 {},
				false => if now % T::VotingPeriodLength == 0 { /* Transition to proposal period*/ },
			}
			0
		}
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a proposal that can be voted on.
		/// TODO: The weight should be adjusted in order to control for spam.
		#[pallet::weight(1_000)]
		pub fn create_proposal(origin: OriginFor<T>, proposal: u8) -> DispatchResult {
			// Is the transaction signed
			ensure_signed(origin)?;
			// Are we in the proposal period?
			ensure!(ProposalPeriod::<T>::exists(), Error::<T>::NotInProposalPeriod);
			//ensure!(AllProposals::<T>::get(&proposal), Error::<T>::ProposalAlreadyExists);
			// Cool, continue with a bog standard storage entry.
			Ok(())
		}

		/// Withdraw a proposal if still within the same voting period.
		/// TODO: This should probably cost more to stop people from wasting others' time.
		/// TODO: Adjust in relation with create_proposal weights.
		#[pallet::weight(10_000)]
		pub fn withdraw_proposal(origin: OriginFor<T>, proposal: u8) -> DispatchResult {
			// Is the transaction signed
			ensure_signed(origin);
			// Are we in the proposal period?
			ensure!(ProposalPeriod::<T>::exists(), Error::<T>::NotInProposalPeriod);
			Ok(())
		}

		// Cast a vote on an existing proposal.
		#[pallet::weight(1_000)]
		pub fn cast_vote(origin: OriginFor<T>, proposal: u8, amount: u32) -> DispatchResult {
			// Is the transaction signed
			ensure_signed(origin);
			// Are we in the proposal period?
			ensure!(!ProposalPeriod::<T>::exists(), Error::<T>::NotInVotingPeriod);
			Ok(())
		}
	}
}
