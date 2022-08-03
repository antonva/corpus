#![cfg_attr(not(feature = "std"), no_std)]

//! ## Quadravote pallet
//!
//! Quadratic voting is the concept of TODO... <waffle here> <link here>
//! [Read more](https://en.wikipedia.org/wiki/Quadratic_voting)
//!
//! ## Overview
//! The Quadratic voting pallet provides functions for:
//!
//! - Creating and withdrawing proposals
//! - Voting on proposals.
//!
//!
//! The implementation splits time up in two periods. First, a proposal period
//! where identified account holders can bring up a proposal for voting.
//! Then a voting period where identified account holders can reserve an amount
//! of their tokens to vote on a particular subject.
//!
//! A proposer can withdraw their proposal if it is still the same proposal round
//!
//! ### Terminology:
//!
//! A period: either voting or proposing takes place within a period. The period for voting
//! is exactly as long as the period for proposing. This is due to the fact that the current
//! naive selection of pallets is the modulus of the length of a period.
//!
//! An identified account holder is an account holder that has registered to vote
//! via the `pallet-votingregistry` pallet VotingRegistry module.
//!
//! A vote is the square root of the amount of tokens reserved.
//!
//! Storage:
//! A ProposalPeriod 'boolean' is stored to denote if we are in the voting period or the proposal period.
//! Every proposal is stored in CountedProposals, a CountedStorageMap that is limited in size
//! per round (defined in Config::MaxProposals).
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use corpus_traits::IdentityInterface;
	use frame_support::{
		dispatch::DispatchResult,
		fail,
		inherent::Vec,
		pallet_prelude::*,
		traits::{Currency, ReservableCurrency},
		BoundedVec,
	};
	use frame_system::{
		ensure_signed,
		pallet_prelude::{BlockNumberFor, *},
	};

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: ReservableCurrency<Self::AccountId>;
		type IdentityProvider: IdentityInterface<Self::AccountId>;

		/// How many blocks does each period run for.
		#[pallet::constant]

		type PeriodLength: Get<u32>;
		/// How many proposals can run simultaneously.
		#[pallet::constant]

		type MaxProposals: Get<u32>;
		/// How many votes can be cast for or against a proposal by a single account.
		#[pallet::constant]
		type MaxVotesPerAccount: Get<u32>;

		/// How many voters can participate in a single voting period.
		/// This is of course not very democratic but there's a tradeoff
		/// to be made and by having the bound, it is possible to make
		/// decisions informed by benchmarking later.
		#[pallet::constant]
		type MaxVotersPerSession: Get<u32>;
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
	#[pallet::unbounded]
	pub type LeftoverVoterCursor<T: Config> = StorageValue<_, Vec<u8>>;

	#[pallet::storage]
	pub(super) type CountedProposals<T: Config> =
		CountedStorageMap<_, Blake2_128Concat, [u8; 32], T::AccountId>;

	#[derive(Encode, Decode, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct Voter<T: Config> {
		total_votes: u32,
		amount_reserved: BalanceOf<T>,
		votes_per_proposal: BoundedVec<u32, T::MaxVotesPerAccount>,
	}
	#[derive(Encode, Decode, Debug, TypeInfo, MaxEncodedLen, Eq, PartialEq, Clone)]
	pub struct VotingProposal {
		proposal: [u8; 32],
		votes_for: u32,
		votes_against: u32,
	}

	#[pallet::storage]
	#[pallet::getter(fn get_all_voters)]
	pub type Voters<T: Config> = CountedStorageMap<_, Blake2_128Concat, T::AccountId, Voter<T>>;

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
		WinningProposals { winners: BoundedVec<VotingProposal, T::MaxProposals> },
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
		// The proposal index is out of bounds
		ProposalIndexOutOfBounds,
		// This account has used up all their votes for this voting period
		AllVotesCastForAccount,
		// User input at fault
		MathError,
		// The Sky is falling, all bets are off
		SkyIsFalling,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(now: BlockNumberFor<T>) -> Weight {
			let period_length = T::PeriodLength::get();
			match ProposalPeriod::<T>::exists() {
				true => {
					if now % period_length.into() == 0u32.into() {
						// The proposal period has ended, this block and forward will not
						// validate any new proposals or withdrawal requests.
						ProposalPeriod::<T>::kill();
						Self::deposit_event(Event::ProposalPeriodEnded { block: now })
					}
				},
				false => {
					if now % period_length.into() == 0u32.into() {
						// The voting period has ended, this block and forward will not
						// validate any votes cast.
						ProposalPeriod::<T>::put(());
						Self::deposit_event(Event::VotingPeriodEnded { block: now });
						// TODO: Calculate winning proposals
						let maybe_proposals = Proposals::<T>::get();
						match maybe_proposals {
							Some(proposals) => {
								let winners: BoundedVec<VotingProposal, T::MaxProposals> =
									BoundedVec::truncate_from(
										proposals
											.into_inner()
											.into_iter()
											.filter(|p| p.votes_for > p.votes_against)
											.collect(),
									);
								Self::deposit_event(Event::WinningProposals { winners })
							},
							None => (), //No proposals, no winners.
						};
						// TODO: Store the winning proposals

						Proposals::<T>::kill();
						// The only time this is called from here is the beginning of the voting
						// period. Therefore we can safely assume that None can always be passed
						// as long as `MaxProposals` is > 1
						let voter_result = Voters::<T>::clear(T::MaxVotersPerSession::get(), None);
						let proposal_result =
							CountedProposals::<T>::clear(T::MaxProposals::get(), None);
						LeftoverVoterCursor::<T>::set(voter_result.maybe_cursor);
						LeftoverProposalCursor::<T>::set(proposal_result.maybe_cursor);
					} else {
						// Continue to clean up the proposals and voters for as long as there
						// exists a cursor.
						let leftover_proposals = LeftoverProposalCursor::<T>::get();
						if let Some(cursor) = leftover_proposals {
							let result =
								CountedProposals::<T>::clear(T::MaxProposals::get(), Some(&cursor));
							LeftoverProposalCursor::<T>::set(result.maybe_cursor);
						}
						let leftover_voters = LeftoverVoterCursor::<T>::get();
						if let Some(cursor) = leftover_voters {
							let result =
								Voters::<T>::clear(T::MaxVotersPerSession::get(), Some(&cursor));
							LeftoverVoterCursor::<T>::set(result.maybe_cursor);
						}
					}
				},
			}
			// TODO: Figure out weight
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
		pub fn create_proposal(origin: OriginFor<T>, proposal: [u8; 32]) -> DispatchResult {
			// Is the transaction signed
			let creator = ensure_signed(origin)?;

			// Is the runtime in a proposal period
			ensure!(ProposalPeriod::<T>::exists(), Error::<T>::NotInProposalPeriod);

			// Is the creator identified
			ensure!(
				<<T as Config>::IdentityProvider as IdentityInterface<T::AccountId>>::is_identified(
					&creator
				),
				Error::<T>::NotIdentified
			);

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
			let res = Proposals::<T>::try_append(VotingProposal {
				proposal,
				votes_for: 0,
				votes_against: 0,
			});

			match res {
				Ok(_) => (),
				_Error => (), // TODO: catch and roll back here for extra defensive coding
			};

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
			// already been made in create_proposal and the account might have deregistered
			// their identity while still having a proposal active.

			// Is the runtime in a proposal period
			ensure!(ProposalPeriod::<T>::exists(), Error::<T>::NotInProposalPeriod);

			// Did this account create this proposal? Does it exist?
			match CountedProposals::<T>::get(proposal) {
				Some(creator) => ensure!(sender == creator, Error::<T>::NotYourProposal),
				None => fail!(Error::<T>::ProposalDoesNotExist),
			}
			CountedProposals::<T>::remove(proposal);
			Self::deposit_event(Event::ProposalWithdrawn { proposal });
			Ok(())
		}

		/// Cast a vote on a proposal in the current period.
		/// When a vote is cast, an amount of tokens equal to the number of votes multiplied by itself
		/// will be reserved. Currency reserved to cast votes will not be released until the end of the
		/// voting period.
		/// TODO: Weights.
		#[pallet::weight(1_000)]
		pub fn cast_vote(
			origin: OriginFor<T>,
			proposal_index: u32,
			votes_for: u32,
			votes_against: u32,
		) -> DispatchResult {
			// Is the transaction signed
			let sender = ensure_signed(origin)?;

			// Is the voting period active
			ensure!(!ProposalPeriod::<T>::exists(), Error::<T>::NotInVotingPeriod);

			// Is the voter identified
			ensure!(
				<<T as Config>::IdentityProvider as IdentityInterface<T::AccountId>>::is_identified(
					&sender
				),
				Error::<T>::NotIdentified
			);

			// Is the proposal index within bounds
			let is_index_inside_bounds =
				proposal_index.ge(&0) && proposal_index.lt(&T::MaxProposals::get());
			ensure!(is_index_inside_bounds, Error::<T>::ProposalIndexOutOfBounds);

			let mut voter;
			match Voters::<T>::get(&sender) {
				Some(v) => voter = v,
				None => {
					// This should be okay due to the bounds
					let mut bv = Vec::new();
					for _ in 0..T::MaxProposals::get() {
						bv.push(0u32);
					}
					voter = Voter::<T> {
						total_votes: 0u32,
						amount_reserved: 0u32.into(),
						votes_per_proposal: BoundedVec::truncate_from(bv),
					}
				},
			};

			// Tally up account's votes
			let sum_votes_in;
			match votes_for.checked_add(votes_against) {
				Some(sum) => sum_votes_in = sum,
				None => fail!(Error::<T>::MathError),
			};

			let new_total_votes;
			match voter.total_votes.checked_add(sum_votes_in) {
				Some(sum) => new_total_votes = sum,
				None => fail!(Error::<T>::MathError),
			};

			// Is the account's total account tally  greater than MaxVotesPerAccount
			ensure!(
				new_total_votes <= T::MaxVotesPerAccount::get(),
				Error::<T>::AllVotesCastForAccount
			);

			// Update the vote state for this account.
			let old_votes = voter.votes_per_proposal[proposal_index as usize];
			voter.votes_per_proposal[proposal_index as usize] += sum_votes_in;

			// The number of votes this account has for this index
			// NOTE: We count votes for and votes against equally, so
			// votes_for: 2, votes_against: 2 in 2 or more calls will count
			// as 4 votes and not 0 votes. Consider this a price on indecision :-)
			let new_votes;
			match old_votes.checked_add(sum_votes_in) {
				Some(sum) => new_votes = sum,
				None => fail!(Error::<T>::MathError),
			}

			// Since there might already be old reserves, we subtract the
			// new total from the old total to get the new reserve to add.
			let old_reserve = old_votes.pow(2);
			let new_full_reserve: u32 = new_votes.pow(2);
			let new_reserve: u32;
			match new_full_reserve.checked_sub(old_reserve) {
				Some(sub) => new_reserve = sub,
				None => fail!(Error::<T>::MathError),
			}
			T::Currency::reserve(&sender, new_reserve.into())?;
			// Update voter total balance
			voter.amount_reserved = new_full_reserve.into();
			voter.total_votes = new_total_votes;

			// Finally we fetch the proposals
			let mut proposals;
			match Proposals::<T>::get() {
				Some(ps) => proposals = ps,
				None => fail!(Error::<T>::SkyIsFalling),
			}
			proposals[proposal_index as usize].votes_for += votes_for;
			proposals[proposal_index as usize].votes_against += votes_against;
			let proposal_hash = proposals[proposal_index as usize].proposal.clone();

			Proposals::<T>::set(Some(proposals));
			Voters::<T>::set(&sender, Some(voter));

			Self::deposit_event(Event::VoteRegistered { proposal: proposal_hash, who: sender });

			Ok(())
		}
	}
}
