//! # Simple identity Pallet
//!
//! - [`Config`]
//! - [`Call`]

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use frame_support::traits::{Currency, OnUnbalanced, ReservableCurrency};
use identity_primitives::Identifiable;
pub use pallet::*;
use sp_runtime::traits::{StaticLookup, Zero};
use sp_std::prelude::*;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
type NegativeImbalanceOf<T> =
	<<T as Config>::Currency as Currency<AccountIdOf<T>>>::NegativeImbalance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The currency trait.
		type Currency: ReservableCurrency<Self::AccountId>;

		/// Reservation fee.
		#[pallet::constant]
		type ReservationFee: Get<BalanceOf<Self>>;

		/// Handler for when some currency "account" decreased in balance for slashing reasons.
		type Slashed: OnUnbalanced<NegativeImbalanceOf<Self>>;

		/// Origin superuser.
		type ForceOrigin: EnsureOrigin<Self::Origin>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A hash was set.
		HashSet { who: T::AccountId },
		/// A hash was forcibly set.
		HashForced { target: T::AccountId },
		/// A hash was changed.
		HashChanged { who: T::AccountId },
		/// A hash was cleared, and the given balance returned.
		HashCleared { who: T::AccountId, deposit: BalanceOf<T> },
		/// A hash was removed and the given balance slashed.
		HashKilled { target: T::AccountId, deposit: BalanceOf<T> },
	}

	/// Error for the simple identity pallet.
	#[pallet::error]
	pub enum Error<T> {
		/// An account isn't identified.
		NotExists,
	}

	/// The lookup table for hashes.
	#[pallet::storage]
	pub(super) type Identities<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, (T::Hash, BalanceOf<T>)>;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]

		/// Set a new hash or change it.
		pub fn set_hash(origin: OriginFor<T>, hash: T::Hash) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let deposit = if let Some((_, deposit)) = <Identities<T>>::get(&sender) {
				Self::deposit_event(Event::<T>::HashChanged { who: sender.clone() });
				deposit
			} else {
				let deposit = T::ReservationFee::get();
				T::Currency::reserve(&sender, deposit)?;
				Self::deposit_event(Event::<T>::HashSet { who: sender.clone() });
				deposit
			};

			<Identities<T>>::insert(&sender, (hash, deposit));
			Ok(())
		}

		/// Clear owned hash
		#[pallet::weight(70_000_000)]
		pub fn clear_hash(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let deposit = <Identities<T>>::take(&sender).ok_or(Error::<T>::NotExists)?.1;

			let err_amount = T::Currency::unreserve(&sender, deposit);
			debug_assert!(err_amount.is_zero());

			Self::deposit_event(Event::<T>::HashCleared { who: sender, deposit });
			Ok(())
		}

		/// Remove an account's hash and take charge of the deposit.
		#[pallet::weight(70_000_000)]
		pub fn kill_hash(
			origin: OriginFor<T>,
			target: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			T::ForceOrigin::ensure_origin(origin)?;

			// Figure out who we're meant to be clearing.
			let target = T::Lookup::lookup(target)?;
			// Grab their deposit (and check that they have one).
			let deposit = <Identities<T>>::take(&target).ok_or(Error::<T>::NotExists)?.1;
			// Slash their deposit from them.
			T::Slashed::on_unbalanced(T::Currency::slash_reserved(&target, deposit).0);

			Self::deposit_event(Event::<T>::HashKilled { target, deposit });
			Ok(())
		}

		/// Set a third-party account's hash with no deposit.
		#[pallet::weight(70_000_000)]
		pub fn force_hash(
			origin: OriginFor<T>,
			target: <T::Lookup as StaticLookup>::Source,
			hash: T::Hash,
		) -> DispatchResult {
			T::ForceOrigin::ensure_origin(origin)?;

			let target = T::Lookup::lookup(target)?;
			let deposit = <Identities<T>>::get(&target).map(|x| x.1).unwrap_or_else(Zero::zero);
			<Identities<T>>::insert(&target, (hash, deposit));

			Self::deposit_event(Event::<T>::HashForced { target });
			Ok(())
		}
	}
}

impl<T: Config> Identifiable<T::AccountId, (T::Hash, BalanceOf<T>)> for Pallet<T> {
	fn is_identified(who: &T::AccountId) -> bool {
		Identities::<T>::get(&who).is_some()
	}

	fn set_identity(who: &T::AccountId, identity_details: (T::Hash, BalanceOf<T>)) {
		Identities::<T>::insert(who, identity_details);
	}

	fn get_identity(who: &T::AccountId) -> Option<(T::Hash, BalanceOf<T>)> {
		Identities::<T>::get(who)
	}
}
