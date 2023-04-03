use codec::{Decode, Encode};
use frame_support::pallet_prelude::TypeInfo;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_std::{collections::btree_set::BTreeSet, vec::Vec};

#[derive(Encode, Decode, PartialEq, Eq, Debug, Clone, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct User {
	pub name: Vec<u8>,
	rating_as_worker: Option<u8>,
	rating_as_publisher: Option<u8>,
	overall_rating: Option<u8>,
	// tags
	tags: BTreeSet<Vec<u8>>,
}

impl Default for User {
	fn default() -> Self {
		Self {
			name: Vec::new(),
			rating_as_worker: None,
			rating_as_publisher: None,
			overall_rating: None,
			tags: BTreeSet::new(),
		}
	}
}

impl User {
	pub fn update_worker_rating(&mut self, new_rating: u8) {
		match self.rating_as_worker {
			None => self.rating_as_worker = Some(new_rating),
			Some(rating) => self.rating_as_worker = Some((rating + new_rating) / 2),
		}
		self.update_overall_rating();
	}

	pub fn update_publisher_rating(&mut self, new_rating: u8) {
		match self.rating_as_publisher {
			None => self.rating_as_publisher = Some(new_rating),
			Some(rating) => self.rating_as_publisher = Some((rating + new_rating) / 2),
		}
		self.update_overall_rating();
	}

	pub fn get_worker_rating(&self) -> Option<u8> {
		self.rating_as_worker
	}

	pub fn get_publisher_rating(&self) -> Option<u8> {
		self.rating_as_publisher
	}

	pub fn update_user_tags(&mut self, tags: BTreeSet<Vec<u8>>) {
		self.tags = tags
	}

	fn update_overall_rating(&mut self) {
		match self.rating_as_worker {
			None => match self.rating_as_publisher {
				None => self.overall_rating = None,
				Some(p_rating) => self.overall_rating = Some(p_rating),
			},
			Some(w_rating) => match self.rating_as_publisher {
				None => self.overall_rating = None,
				Some(p_rating) => self.overall_rating = Some((w_rating + p_rating) / 2),
			},
		}
	}
}

pub trait UserTrait<AccountId> {
	fn get_user_from_storage(id: &AccountId) -> User;
	fn save_user_to_storage(id: AccountId, user: User);
}

pub trait TagsTrait {
	fn get_tags_from_storage() -> BTreeSet<Vec<u8>>;
	fn save_tags_to_storage(tags: BTreeSet<Vec<u8>>);
}
