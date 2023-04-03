#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod functions;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	use tasking_traits::user::*;

	use sp_std::{collections::btree_set::BTreeSet, vec::Vec};

	#[pallet::pallet]
	#[pallet::without_storage_info]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn get_user)]
	pub(super) type UserStorage<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, User, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_tags)]
	pub(super) type TagsStorage<T: Config> = StorageValue<_, BTreeSet<Vec<u8>>, ValueQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// User updated. [AccountId]
		UserUpdated { user: T::AccountId },
		/// Tags Added
		TagsAdded,
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Invalid Tag
		InvalidTag,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn add_tags(origin: OriginFor<T>, tags: Vec<Vec<u8>>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let mut stored_tags = Self::get_tags_from_storage();

			for item in tags.iter() {
				stored_tags.insert(item.clone());
			}

			Self::save_tags_to_storage(stored_tags);

			Self::deposit_event(Event::<T>::TagsAdded);

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn update_user(
			origin: OriginFor<T>,
			name: Option<Vec<u8>>,
			tags: Option<Vec<Vec<u8>>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let stored_tags = Self::get_tags();
			let mut user_tags = BTreeSet::new();
			let mut user = Self::get_user_from_storage(&who);
			match name {
				None => (),
				Some(user_name) => user.name = user_name,
			};
			match tags {
				None => (),
				Some(given_tags) => {
					for item in given_tags.iter() {
						let cloned_item = item.clone();
						ensure!(stored_tags.contains(&cloned_item), Error::<T>::InvalidTag);
						user_tags.insert(item.clone());
					}
					user.update_user_tags(user_tags);
				},
			}
			Self::save_user_to_storage(who.clone(), user);
			Self::deposit_event(Event::<T>::UserUpdated { user: who });
			Ok(())
		}
	}
}
