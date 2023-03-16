use super::*;

use frame_support::{
	ensure,
	sp_runtime::{DispatchError, DispatchResult},
	traits::{tokens::ExistenceRequirement, Currency},
	PalletId,
};
use frame_system::{ensure_signed, pallet_prelude::OriginFor};
use sp_std::vec::Vec;
use tasking_primitives::*;
use tasking_traits::task::*;

impl<T: Config> Pallet<T> {
	pub fn do_create_task(
		who: AccountOf<T>,
		metadata: Vec<u8>,
		cost: BalanceOf<T>,
		deadline: u8,
	) -> Result<(), DispatchError> {
		let task_id = Self::task_id() + 1;
		let created_at = Self::get_current_block_number();
		let task = Task::new(who.clone(), metadata, cost, created_at, deadline);

		// escrow logic
		let escrow_account = Self::escrow_account_id(task_id);

		T::Currency::transfer(
			&who,
			&escrow_account,
			task.get_cost(),
			ExistenceRequirement::KeepAlive,
		)?;

		<TaskStorage<T>>::insert(task_id, task);

		<TaskNumber<T>>::set(task_id);

		Self::deposit_event(Event::<T>::TaskCreated { task_id, owner: who });

		Ok(())
	}

	pub fn do_bid(who: AccountOf<T>, task_id: TaskId) -> Result<(), DispatchError> {
		// ensure that the task exist
		ensure!(<TaskStorage<T>>::contains_key(&task_id), <Error<T>>::TaskDoesNotExist);

		let task = <TaskStorage<T>>::get(task_id).unwrap();

		// ensure task is accepting bids
		ensure!(task.get_status() == TaskStatus::Open, <Error<T>>::NotAllowed);

		// ensure that owner is not the bidder
		ensure!(!task.check_ownership(&who), <Error<T>>::OwnerCannotBid);

		let mut bidder_list = <BidderList<T>>::get(task_id);

		// check if already bid
		ensure!(!bidder_list.contains(&who), <Error<T>>::BidAlreadyPlaced);

		bidder_list.insert(who.clone());

		// escrow logic
		let escrow_account = Self::escrow_account_id(task_id);

		T::Currency::transfer(
			&who,
			&escrow_account,
			task.get_cost(),
			ExistenceRequirement::KeepAlive,
		)?;

		<BidderList<T>>::insert(task_id, bidder_list);

		Self::deposit_event(Event::<T>::BidPlaced { task_id, bidder: who });

		Ok(())
	}

	pub fn retract_bid(who: AccountOf<T>, task_id: TaskId) -> Result<(), DispatchError> {
		ensure!(<TaskStorage<T>>::contains_key(task_id), <Error<T>>::TaskDoesNotExist);

		let task = <TaskStorage<T>>::get(task_id).unwrap();

		let mut bidder_list = <BidderList<T>>::get(task_id);

		ensure!(bidder_list.remove(&who), <Error<T>>::BidDoesNotExist);

		let escrow_account = Self::escrow_account_id(task_id);

		T::Currency::transfer(
			&escrow_account,
			&who,
			task.get_cost(),
			ExistenceRequirement::KeepAlive,
		)?;

		Self::deposit_event(Event::<T>::BidRemoved { task_id, bidder: who });

		Ok(())
	}

	pub fn get_current_block_number() -> BlockNumberOf<T> {
		let block_number = <frame_system::Pallet<T>>::block_number();
		block_number
	}
}
