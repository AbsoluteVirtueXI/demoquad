#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::{Currency, ReservableCurrency};
use frame_support::{
	pallet_prelude::{ValueQuery, *},
	Twox64Concat,
};
use frame_system::pallet_prelude::*;
use identity_primitives::Identifiable;
pub use pallet::*;
use sp_runtime::ArithmeticError;
use sp_std::vec::Vec;

/*
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
*/

type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The minimum length of a proposal.
		#[pallet::constant]
		type MinLength: Get<u32>;

		/// The maximum length of a proposal.
		#[pallet::constant]
		type MaxLength: Get<u32>;

		/// Number of blocks for the proposal duration.
		#[pallet::constant]
		type Duration: Get<u32>;

		/// Nb Votes per users
		#[pallet::constant]
		type MaxVotes: Get<u32>;

		/// Max proposals submisations per block;
		type MaxProposalsPerBlock: Get<u32>;

		/// The currency trait.
		type Currency: ReservableCurrency<Self::AccountId>;

		type Identity: Identifiable<Self::AccountId, (Self::Hash, BalanceOf<Self>)>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, TypeInfo)]
	pub enum Choice {
		Yes,
		No,
	}

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct Proposal<T: Config> {
		pub proposer: T::AccountId,
		pub proposal: BoundedVec<u8, T::MaxLength>,
		pub nb_yes: u32,
		pub nb_no: u32,
		pub start: T::BlockNumber,
		pub end: T::BlockNumber,
	}

	#[pallet::storage]
	#[pallet::getter(fn nb_proposals)]
	pub type NextProposalId<T> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	pub type Proposals<T> = StorageMap<_, Twox64Concat, u32, Proposal<T>, OptionQuery>;

	#[pallet::storage]
	pub type ProposalsPerBlock<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::BlockNumber,
		BoundedVec<u32, T::MaxProposalsPerBlock>,
		ValueQuery,
	>;

	#[pallet::storage]
	pub type DidVote<T: Config> =
		StorageDoubleMap<_, Twox64Concat, u32, Twox64Concat, T::AccountId, bool, ValueQuery>;

	#[pallet::storage]
	pub type RemainingVote<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, u32>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ProposalSubmited(u32, T::AccountId),
		VoteSubmited(u32, T::AccountId),
		ProposalEnded(u32),
		WinProposal(u32),
		LostProposal(u32),
	}

	#[pallet::error]
	pub enum Error<T> {
		NoIdentityFound,
		ProposalTooLong,
		ProposalTooShort,
		ProposalNotExists,
		ProposalExpired,
		ProposalAlreadyExists,
		TooManyProposalsInBloc,
		AlreadyVoted,
		Unknown,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(block_number: BlockNumberFor<T>) -> Weight {
			if let Ok(proposals) = <ProposalsPerBlock<T>>::try_get(block_number) {
				for proposal_id in proposals {
					let proposal = <Proposals<T>>::get(proposal_id).expect("it is chechk!!");
					if proposal.nb_yes > proposal.nb_no {
						Self::deposit_event(Event::WinProposal(proposal_id));
					} else {
						Self::deposit_event(Event::LostProposal(proposal_id))
					}
					Self::deposit_event(Event::ProposalEnded(proposal_id));
				}
			}
			0
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn submit_proposal(origin: OriginFor<T>, text: Vec<u8>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(T::Identity::is_identified(&who), Error::<T>::NoIdentityFound);

			// Check proposal length
			let bounded_text: BoundedVec<_, _> =
				text.try_into().map_err(|()| Error::<T>::ProposalTooLong)?;
			ensure!(
				bounded_text.len() >= T::MinLength::get() as usize,
				Error::<T>::ProposalTooShort
			);

			// Calculate end of proposal lifetime
			let current_block_number = <frame_system::Pallet<T>>::block_number();
			let end_block_number =
				current_block_number + T::BlockNumber::from(T::Duration::get() + 1);

			// Fill new proposal
			let proposal = Proposal {
				proposer: who.clone(),
				proposal: bounded_text,
				nb_yes: 0,
				nb_no: 0,
				start: current_block_number,
				end: end_block_number,
			};

			// Save proposal
			let proposal_id = Self::get_next_proposal_id()?;
			<Proposals<T>>::insert(proposal_id, proposal);

			// Register proposal in on_initialize hook
			<ProposalsPerBlock<T>>::try_append(end_block_number, proposal_id)
				.map_err(|_| Error::<T>::TooManyProposalsInBloc)?;

			Self::deposit_event(Event::ProposalSubmited(proposal_id, who));
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn vote_proposal(
			origin: OriginFor<T>,
			proposal_id: u32,
			choice: Choice,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(T::Identity::is_identified(&who), Error::<T>::NoIdentityFound);

			let mut proposal = &mut <Proposals<T>>::try_get(proposal_id)
				.map_err(|_| Error::<T>::ProposalNotExists)?;

			if <DidVote<T>>::get(proposal_id, who.clone()) {
				return Err(Error::<T>::AlreadyVoted.into());
			} else {
				<DidVote<T>>::insert(proposal_id, who.clone(), true);
			}

			match choice {
				Choice::Yes => proposal.nb_yes += 1, //TODO safe math add here
				Choice::No => proposal.nb_no += 1,   //TODO safe math add here
			}

			Self::deposit_event(Event::VoteSubmited(proposal_id, who));
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn get_next_proposal_id() -> Result<u32, DispatchError> {
		NextProposalId::<T>::try_mutate(|next_id| -> Result<u32, DispatchError> {
			let current_id = *next_id;
			*next_id = next_id.checked_add(1).ok_or(ArithmeticError::Overflow)?;
			Ok(current_id)
		})
	}
}
