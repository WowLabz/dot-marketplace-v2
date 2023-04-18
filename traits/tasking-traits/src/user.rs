use codec::{Decode, Encode};
use frame_support::pallet_prelude::TypeInfo;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_std::{collections::btree_set::BTreeSet, vec::Vec};

#[derive(Encode, Decode, PartialEq, Eq, Debug, Clone, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct User {
	pub name: Vec<u8>,
	pub email: Vec<u8>,
	pub phone_number: Vec<u8>,
	total_work_as_worker: u32,

	// rating is saved in 10 power 3
	rating_as_worker: Option<u32>,
	total_work_as_publisher: u32,
	rating_as_publisher: Option<u32>,
	overall_rating: Option<u32>,
	// tags
	tags: BTreeSet<Vec<u8>>,
}

impl Default for User {
	fn default() -> Self {
		Self {
			name: Vec::new(),
			email: Vec::new(),
			phone_number: Vec::new(),
			total_work_as_worker: 0,
			rating_as_worker: None,
			total_work_as_publisher: 0,
			rating_as_publisher: None,
			overall_rating: None,
			tags: BTreeSet::new(),
		}
	}
}

impl User {
	pub fn update_worker_rating(&mut self, new_rating: u8) {
		match self.rating_as_worker {
			None => {
				self.total_work_as_worker += 1;
				self.rating_as_worker = Some(new_rating as u32 * 1000);
			},
			Some(rating) => {
				let total_rating = self.total_work_as_worker * rating + new_rating as u32 * 1000;
				self.total_work_as_worker += 1;
				let avg_rating = total_rating / self.total_work_as_worker;
				self.rating_as_worker = Some(avg_rating);
			},
		}
		self.update_overall_rating();
	}

	pub fn update_publisher_rating(&mut self, new_rating: u8) {
		match self.rating_as_publisher {
			None => {
				self.total_work_as_publisher += 1;
				self.rating_as_publisher = Some(new_rating as u32 * 1000);
			},
			Some(rating) => {
				let total_rating = self.total_work_as_publisher * rating + new_rating as u32 * 1000;
				self.total_work_as_publisher += 1;
				let avg_rating = total_rating / self.total_work_as_publisher;
				self.rating_as_publisher = Some(avg_rating);
			},
		}
		self.update_overall_rating();
	}

	pub fn get_worker_rating(&self) -> Option<u32> {
		self.rating_as_worker
	}

	pub fn get_publisher_rating(&self) -> Option<u32> {
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
				None => self.overall_rating = Some(w_rating),
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
