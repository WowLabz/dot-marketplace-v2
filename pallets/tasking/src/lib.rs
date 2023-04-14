#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

mod functions;
mod types;
use types::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		pallet_prelude::*, sp_runtime::traits::AccountIdConversion, traits::LockableCurrency,
		PalletId,
	};
	use frame_system::pallet_prelude::*;

	use sp_std::{collections::btree_set::BTreeSet, vec::Vec};

	use tasking_primitives::TaskId;
	use tasking_traits::{task::*, user::*};

	#[pallet::pallet]
	#[pallet::without_storage_info]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type Currency: LockableCurrency<Self::AccountId>;
		type UserTrait: UserTrait<Self::AccountId>;

		#[pallet::constant]
		type PalletId: Get<PalletId>;

		#[pallet::constant]
		type BidAmount: Get<BalanceOf<Self>>;
	}

	#[pallet::storage]
	#[pallet::getter(fn task_id)]
	pub type TaskNumber<T> = StorageValue<_, TaskId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn tasks)]
	pub type TaskStorage<T: Config> =
		StorageMap<_, Blake2_128Concat, TaskId, Task<AccountOf<T>, BalanceOf<T>, BlockNumberOf<T>>>;

	#[pallet::storage]
	pub(super) type BidderList<T: Config> =
		StorageMap<_, Blake2_128Concat, TaskId, BTreeSet<AccountOf<T>>, ValueQuery>;

	#[pallet::storage]
	pub(super) type AcceptedBid<T: Config> = StorageMap<_, Blake2_128Concat, TaskId, AccountOf<T>>;

	// for slashing
	#[pallet::storage]
	pub(super) type NextOperationDeadline<T: Config> =
		StorageMap<_, Blake2_128Concat, TaskId, BlockNumberOf<T>>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// Task Created. [TaskId, Owner]
		TaskCreated { task_id: TaskId, owner: AccountOf<T> },
		/// Bid Placed. [TaskId, Bidder]
		BidPlaced { task_id: TaskId, bidder: AccountOf<T> },
		/// Bid retracted/removed/deleted. [TaskId, Bidder]
		BidRemoved { task_id: TaskId, bidder: AccountOf<T> },
		/// Bid accepted. [TaskId, Bidder]
		BidAccepted { task_id: TaskId, bidder: AccountOf<T> },
		/// Bid Rejected. [TaskId, Bidder]
		BidRejected { task_id: TaskId, bidder: AccountOf<T> },
		/// Work accepted. [TaskId, Bidder]
		WorkAccepted { task_id: TaskId, bidder: AccountOf<T> },
		/// Work rejected. [TaskId, Bidder]
		WorkRejected { task_id: TaskId, bidder: AccountOf<T> },
		/// Task Completed. [TaskId]
		WorkCompleted { task_id: TaskId, worker: AccountOf<T> },
		/// Task Approved. [TaskId, WorkerRating]
		TaskApproved { task_id: TaskId, rating: u8 },
		/// Task Disapproved. [TaskId]
		TaskDisapproved { task_id: TaskId },
		/// Customer Rating Provided. [TaskId, CustomerRating]
		CustomerRatingProvided { task_id: TaskId, rating: u8 },
		/// Task Completed. [TaskId]
		TaskCompleted { task_id: TaskId },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// The task does not exist
		TaskDoesNotExist,
		/// Onwer cannot bid
		OwnerCannotBid,
		/// Bid already placed
		BidAlreadyPlaced,
		/// Operation not allowed
		NotAllowed,
		/// Bid does not exist
		BidDoesNotExist,
		/// No Permission
		NoPermission,
		/// Not the task worker
		NotWorker,
		/// Rating range invalid
		InvalidRatingInput,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn create_task(
			origin: OriginFor<T>,
			metadata: Vec<u8>,
			cost: BalanceOf<T>,
			deadline: u8,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_create_task(who, metadata, cost, deadline)
		}

		#[pallet::weight(10_000)]
		pub fn bid_on_task(origin: OriginFor<T>, task_id: TaskId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_bid(who, task_id)
		}

		#[pallet::weight(10_000)]
		pub fn undo_bid(origin: OriginFor<T>, task_id: TaskId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::retract_bid(who, task_id)
		}

		#[pallet::weight(10_000)]
		pub fn accept_bid(
			origin: OriginFor<T>,
			task_id: TaskId,
			bidder: AccountOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_accept_bid(who, task_id, bidder)
		}

		#[pallet::weight(10_000)]
		pub fn reject_bid(
			origin: OriginFor<T>,
			task_id: TaskId,
			bidder: AccountOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_reject_bid(who, task_id, bidder)
		}

		#[pallet::weight(10_000)]
		pub fn accept_work(origin: OriginFor<T>, task_id: TaskId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_accept_work(who, task_id)
		}

		#[pallet::weight(10_000)]
		pub fn reject_work(origin: OriginFor<T>, task_id: TaskId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_reject_work(who, task_id)
		}

		#[pallet::weight(10_000)]
		pub fn complete_work(
			origin: OriginFor<T>,
			task_id: TaskId,
			worker_attachments: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_complete_work(who, task_id, worker_attachments)
		}

		#[pallet::weight(10_000)]
		pub fn approve_task(
			origin: OriginFor<T>,
			task_id: TaskId,
			worker_ratings: u8,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_approve_task(who, task_id, worker_ratings)
		}

		#[pallet::weight(10_000)]
		pub fn disapprove_task(origin: OriginFor<T>, task_id: TaskId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_disapprove_task(who, task_id)
		}

		#[pallet::weight(10_000)]
		pub fn provide_customer_rating(
			origin: OriginFor<T>,
			task_id: TaskId,
			customer_rating: u8,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_provide_customer_rating(who, task_id, customer_rating)
		}

		#[pallet::weight(10_00)]
		pub fn complete_task(origin: OriginFor<T>, task_id: TaskId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_complete_task(who, task_id)
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn escrow_account_id(id: TaskId) -> AccountOf<T> {
			T::PalletId::get().try_into_sub_account(id).unwrap()
		}

		pub fn get_bid_amount() -> BalanceOf<T> {
			T::BidAmount::get()
		}
	}
}
