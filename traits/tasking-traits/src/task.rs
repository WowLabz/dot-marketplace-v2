use codec::{Decode, Encode};
use frame_support::pallet_prelude::TypeInfo;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_std::vec::Vec;

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
	//Customer Rating Pending
	CustomerRatingPending,
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
	created_at: BlockNumber,
	finishes_at: Option<BlockNumber>,
	deadline: u8,
	worker: Option<AccountId>,
	worker_attachments: Option<Vec<u8>>,
}

impl<AccountId: PartialEq, Balance: Copy, BlockNumber> Task<AccountId, Balance, BlockNumber> {
	pub fn new(
		owner: AccountId,
		metadata: Vec<u8>,
		cost: Balance,
		created_at: BlockNumber,
		deadline: u8,
	) -> Self {
		Self {
			owner,
			metadata,
			status: TaskStatus::Open,
			cost,
			created_at,
			finishes_at: None,
			deadline,
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

	pub fn get_status(&self) -> TaskStatus {
		self.status.clone()
	}

	pub fn get_deadline(&self) -> u8 {
		self.deadline
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
