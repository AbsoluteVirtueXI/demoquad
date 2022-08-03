# DemoQuad

pba final exam

## Description

Identified accounts can propose proposals and vote on them with a basic `Yes` or `No` choice.
The proposals can be voted during a certain number of blocks.
Only 3 proposals can be proposed per blocks.
The identification is managed in `pallet-simple-identity`.
The voting system is managed in `pallet-demoquad`.
`pallet-simple-identity` and `pallet-demoquad` are loose-coupled by a trait definied in `identity-primitives`.
These pallets are implemented in the `Runtime` of the node.

## a trait: identity-primitives

A basic identification standard for common shared behavior, making loose-coopling easy for `pallet-demoquad` et `pallet-simple-identity`.

```rust
#![cfg_attr(not(feature = "std"), no_std)]
pub trait Identifiable<AccountId, IdentityDetails> {
	fn is_identified(caller: &AccountId) -> bool;

	fn set_identity(who: &AccountId, identity_details: IdentityDetails);

	fn get_identity(who: &AccountId) -> Option<IdentityDetails>;
}
```

## Identification of account: pallet-simple-identity

The identification is managed in `simple-identity` pallet.
Users can send a hash (computed offchain) of their metadata and store it onchain.
A currency amount is staked during the usage of the service.
This pallet is tested and is fully implemented.

### Notable code:

Some types and macro's black magic:

```rust
type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
type NegativeImbalanceOf<T> =
	<<T as Config>::Currency as Currency<AccountIdOf<T>>>::NegativeImbalance;
```

Implementation of the `Identifiable` trait:

```rust
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
```

## voting system: pallet-demoquad

Identified accounts can submit proposals and vote on them.
User can vote on proposal during a maximum number of block: `Duration`.
A BoundedVector of maximum `MaxProposalsPerBlock` stores each proposal ids that will end in a particular block.
With the `StorageMap` bellow we can find for each block which Proposal ended or are still running.

```rust
#[pallet::storage]
pub type ProposalsPerBlock<T: Config> = StorageMap<
	_,
	Twox64Concat,
	T::BlockNumber,
	BoundedVec<u32, T::MaxProposalsPerBlock>,
	ValueQuery,
>;
```

At each block in the `on_initialize` hook we check if a proposal end at the current block and we emit corresponding events in case of Win/Lost:

```rust
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
```

The voting system itself works but is pretty basic as i didn't have time to fully integrate all features.
The pallet is just partially tested not all cases are covered.

## Amelioration

- Finish to implement all the features for a real quadratic voting system.
- implement a rpc api and ofc try to use cumulus and join a testnet
- Deal with the weight system, as i don't know if how i iterate on a BoundedVector (even small) in the on_finalize hook
  is a good practice. Maybe we can just incentivazed the process of triggering the end of a proposal.
- Even if i start to be comfortable my macro's in Substrate, sometimes i had to think twice before writting.
- I had a lot of name colision while writting my tests. The namespace was polluted and names/types were ambigious
- i mostly did it following my personnal view and feeling, i got some inspiration from pallets, but not sure what would be the best practice for this kind of exercice.

## Conclusion

the project is really interesting, because it deals with a topical subject: people's desire for more transparency and greater involvement in decision-making.
I regret not having finished, i started late and i changed my project on Monday 4am because i had trouble working with the pallet-assets and my Ethereum knowledge overlap too much on my substrate and both paragdim don't mix well sometimes.
This project allowed me to be more comfortable with FRAME and to finally be able to understand the code in a lot of FRAME pallet.
