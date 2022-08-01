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
/// A period: either voting or proposing takes place within a period. The period for voting
/// is exactly as long as the period for proposing. This is due to the fact that the current
/// naive selection of pallets is the modulus of the length of a period.
///
/// An identified account holder is an account holder that has registered an identity
/// via the `pallet-identity` pallet Identity module.
///
/// A vote is the square root of the amount of tokens reserved.
///
/// Storage:
/// A ProposalPeriod 'boolean' is stored to denote if we are in the voting period or the proposal period.
/// Every proposal is stored in CountedProposals, a CountedStorageMap that is limited in size
/// per round (defined in Config::MaxProposals).
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
		dispatch::DispatchResult,
		fail,
		inherent::Vec,
		pallet_prelude::*,
		traits::{Contains, ReservableCurrency},
		BoundedVec,
	};
	use frame_system::{
		ensure_signed,
		pallet_prelude::{BlockNumberFor, *},
	};

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: ReservableCurrency<Self::AccountId>;
		type IdentityProvider: Contains<Self::AccountId>;
		/// How many blocks does each period run for.
		#[pallet::constant]
		type PeriodLength: Get<u32>;
		/// How many proposals can run simultaneously.
		#[pallet::constant]
		type MaxProposals: Get<u32>;
		/// How many votes can be cast for or against a proposal by a single account.
		#[pallet::constant]
		type MaxVotesPerAccount: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

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

	/// There's a possibility that clearing the proposals will not work.
	/// In which case a pointer to the continuation will be stored here.
	/// The use of unbounded here should be okay here as `MaxProposals`
	/// bounds the size of the proposal storage.
	#[pallet::storage]
	#[pallet::unbounded]
	pub type LeftoverProposalCursor<T: Config> = StorageValue<_, Vec<u8>>;

	#[pallet::storage]
	#[pallet::getter(fn get_all_proposals)]
	pub(super) type CountedProposals<T: Config> =
		CountedStorageMap<_, Blake2_128Concat, [u8; 32], T::AccountId>;

	#[derive(Encode, Decode, TypeInfo, MaxEncodedLen)]
	pub struct VotingProposal {
		proposal: [u8; 32],
		votes_for: u32,
		votes_against: u32,
	}

	#[pallet::storage]
	pub type Proposals<T: Config> = StorageValue<_, BoundedVec<VotingProposal, T::MaxProposals>>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ProposalCreated { proposal: [u8; 32] },
		ProposalWithdrawn { proposal: [u8; 32] },
		VotingPeriodEnded { block: BlockNumberFor<T> },
		ProposalPeriodEnded { block: BlockNumberFor<T> },
		VoteRegistered { proposal: [u8; 32], who: T::AccountId },
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
		// Extrinsic sender is not the creator of the proposal.
		NotYourProposal,
		// Proposal already exists when trying to add a new one.
		ProposalAlreadyExists,
		// Proposal doesn't exist, when trying to withdraw an existing one.
		ProposalDoesNotExist,
		// Max proposal threshold reached for this period
		TooManyProposals,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(now: BlockNumberFor<T>) {
			let period_length = T::PeriodLength::get();
			match ProposalPeriod::<T>::exists() {
				true => {
					if now % period_length.into() == 0u32.into() {
						// The proposal period has ended, the next block and forward will not
						// validate any new proposals or withdrawal requests.
						ProposalPeriod::<T>::kill();
						Self::deposit_event(Event::ProposalPeriodEnded { block: now })
					}
				},
				false => {
					if now % period_length.into() == 0u32.into() {
						// The voting period has ended, the next block and forward will not
						// validate any votes cast.
						ProposalPeriod::<T>::put(());
						Self::deposit_event(Event::VotingPeriodEnded { block: now });
						// TODO: Calculate winning proposals
						// TODO: Store the winning proposals
						// TODO: Remove proposals and votes from storage
						// The only time this is called from here is the beginning of the voting
						// period. Therefore we can safely assume that None can always be passed
						// as long as `MaxProposals` is > 1
						let result = CountedProposals::<T>::clear(T::MaxProposals::get(), None);
						LeftoverProposalCursor::<T>::set(result.maybe_cursor);
					} else {
						let removal_result = LeftoverProposalCursor::<T>::get();
						if let Some(cursor) = removal_result {
							let result =
								CountedProposals::<T>::clear(T::MaxProposals::get(), Some(&cursor));
							LeftoverProposalCursor::<T>::set(result.maybe_cursor);
						}
					}
				},
			}
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
		pub fn create_proposal(origin: OriginFor<T>, proposal: [u8; 32]) -> DispatchResult {
			// Is the transaction signed
			let creator = ensure_signed(origin)?;
			// Is the creator identified
			// TODO
			// Is the runtime in a proposal period
			ensure!(ProposalPeriod::<T>::exists(), Error::<T>::NotInProposalPeriod);
			// Does the proposal exist already
			ensure!(
				!CountedProposals::<T>::contains_key(proposal),
				Error::<T>::ProposalAlreadyExists
			);
			// Does the runtime allow for more proposals to be added
			ensure!(
				CountedProposals::<T>::count() < T::MaxProposals::get(),
				Error::<T>::TooManyProposals
			);
			// Cool, continue with storage entry.
			CountedProposals::<T>::insert(proposal, creator);
			Self::deposit_event(Event::ProposalCreated { proposal });
			Ok(())
		}

		/// Withdraw a proposal if still within the same voting period.
		/// TODO: This should probably cost more to stop people from wasting others' time.
		/// TODO: Adjust in relation with create_proposal weights.
		/// TODO: Also read up on benchmarking.
		#[pallet::weight(10_000)]
		pub fn withdraw_proposal(origin: OriginFor<T>, proposal: [u8; 32]) -> DispatchResult {
			// Is the transaction signed
			let sender = ensure_signed(origin)?;
			// There is no need to check for identity in withdraw_proposal as the check has
			// already been made in create_proposal.

			// Is the runtime in a proposal period
			ensure!(ProposalPeriod::<T>::exists(), Error::<T>::NotInProposalPeriod);
			// Did this account create this proposal?
			// Also, does it exist.
			match CountedProposals::<T>::get(proposal) {
				Some(creator) => ensure!(sender == creator, Error::<T>::NotYourProposal),
				None => fail!(Error::<T>::ProposalDoesNotExist),
			}
			CountedProposals::<T>::remove(proposal);
			Self::deposit_event(Event::ProposalWithdrawn { proposal });
			Ok(())
		}

		/// Cast a vote on an existing proposal.
		/// Currency reserved to cast votes will not be released until the end of the voting period.
		#[pallet::weight(1_000)]
		pub fn cast_vote(origin: OriginFor<T>, proposal: [u8; 32], amount: i32) -> DispatchResult {
			// Is the transaction signed
			ensure_signed(origin);
			// Is the voting period active
			ensure!(!ProposalPeriod::<T>::exists(), Error::<T>::NotInVotingPeriod);
			// Is the voter identified
			// We take the hit of writing twice per vote cast in order to have fast voting results.
			// The reserved currency can then be released in batches.
			Ok(())
		}
	}
}
