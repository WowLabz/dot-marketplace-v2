use codec::{Decode, Encode};
use frame_support::pallet_prelude::TypeInfo;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_std::{vec::Vec, collections::btree_set::BTreeSet};

#[derive(Encode, Decode, PartialEq, Eq, Debug, Clone, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum TaskStatus {
	// The task is Open
	Open,
	//Awaiting Bidder Response
	AwaitingBidderResponse,
	// The task is in progress
	InProgress,
	//Pending Approval from Publisher
	PendingApproval,
	// Task is in Revision by the worker
	InRevision,
	//Customer Rating Pending
	CustomerRatingPending,
	// Awating Completion from customer
	AwaitingCompletion,
	// The task is completed
	Completed,
}

#[derive(Encode, Decode, PartialEq, Eq, Debug, Clone, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Task<AccountId, Balance, BlockNumber> {
	owner: AccountId,
	metadata: Vec<u8>,
	status: TaskStatus,
	cost: Balance,
	tags: BTreeSet<Vec<u8>>,
	created_at: BlockNumber,
	finishes_at: Option<BlockNumber>,
	deadline: u8,
	revisions: u8,
	worker: Option<AccountId>,
	worker_attachments: Option<Vec<u8>>,
}

impl<AccountId: PartialEq, Balance: Copy, BlockNumber> Task<AccountId, Balance, BlockNumber> {
	pub fn new(
		owner: AccountId,
		metadata: Vec<u8>,
		cost: Balance,
		tags: BTreeSet<Vec<u8>>,
		created_at: BlockNumber,
		deadline: u8,
	) -> Self {
		Self {
			owner,
			metadata,
			status: TaskStatus::Open,
			cost,
			tags,
			created_at,
			finishes_at: None,
			deadline,
			revisions: 0,
			worker: None,
			worker_attachments: None,
		}
	}

	pub fn check_ownership(&self, account_id: &AccountId) -> bool {
		&self.owner == account_id
	}

	pub fn extend_completion(&mut self, new_deadline: Option<BlockNumber>) {
		self.finishes_at = new_deadline
	}

	pub fn get_cost(&self) -> Balance {
		self.cost
	}

	pub fn get_status(&self) -> &TaskStatus {
		&self.status
	}

	pub fn get_deadline(&self) -> u8 {
		self.deadline
	}

	pub fn get_revisions(&self) -> u8 {
		self.revisions
	}

	pub fn increment_revisions(&mut self) {
		self.revisions += 1;
	}

	pub fn get_worker_details(&self) -> &Option<AccountId> {
		&self.worker
	}

	pub fn get_owner_details(&self) -> &AccountId {
		&self.owner
	}

	pub fn update_cost(&mut self, new_cost: Balance) {
		self.cost = new_cost
	}

	pub fn update_status(&mut self, new_status: TaskStatus) {
		self.status = new_status
	}

	pub fn add_worker(&mut self, worker: AccountId) {
		self.worker = Some(worker)
	}

	pub fn add_worker_attachments(&mut self, attachment_url: Vec<u8>) {
		self.worker_attachments = Some(attachment_url)
	}
}
