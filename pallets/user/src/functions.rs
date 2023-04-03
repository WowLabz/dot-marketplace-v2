use super::*;

use frame_support::{
	ensure,
	sp_runtime::{DispatchError, DispatchResult},
	traits::{tokens::ExistenceRequirement, Currency},
	PalletId,
};
use frame_system::{ensure_signed, pallet_prelude::OriginFor};
use sp_std::{collections::btree_set::BTreeSet, vec::Vec};
use tasking_traits::user::*;

impl<T: Config> TagsTrait for Pallet<T> {
	fn get_tags_from_storage() -> BTreeSet<Vec<u8>> {
		Self::get_tags()
	}

	fn save_tags_to_storage(tags: BTreeSet<Vec<u8>>) {
		<TagsStorage<T>>::set(tags);
	}
}

impl<T: Config> UserTrait<T::AccountId> for Pallet<T> {
	fn get_user_from_storage(id: &T::AccountId) -> User {
		Self::get_user(id)
	}

	fn save_user_to_storage(id: T::AccountId, user: User) {
		<UserStorage<T>>::insert(id, user);
	}
}
