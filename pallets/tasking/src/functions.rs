use super::*;

use frame_support::{
	ensure,
	sp_runtime::{DispatchError},
	traits::{tokens::ExistenceRequirement, Currency},
};

use sp_std::vec::Vec;
use tasking_primitives::{time::*, TaskId};
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
		ensure!(task.get_status() == &TaskStatus::Open, <Error<T>>::NotAllowed);

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

	pub fn do_accept_bid(
		who: AccountOf<T>,
		task_id: TaskId,
		bidder: AccountOf<T>,
	) -> Result<(), DispatchError> {
		ensure!(<TaskStorage<T>>::contains_key(task_id), <Error<T>>::TaskDoesNotExist);
		let mut task = <TaskStorage<T>>::get(task_id).unwrap();

		ensure!(task.check_ownership(&who), <Error<T>>::NoPermission);

		// // TODO: check if a bid is already pending
		ensure!(task.get_status() == &TaskStatus::Open, <Error<T>>::NotAllowed);
		// ensure!(!(<AcceptedBid<T>>::contains_key(task_id)), <Error<T>>::NotAllowed);

		// get bidder list
		let mut bidder_list = <BidderList<T>>::get(task_id);
		ensure!(bidder_list.remove(&bidder), <Error<T>>::BidDoesNotExist);

		<AcceptedBid<T>>::insert(task_id, bidder.clone());
		task.update_status(TaskStatus::AwaitingBidderResponse);

		<TaskStorage<T>>::insert(task_id, task);
		<BidderList<T>>::insert(task_id, bidder_list);

		Self::deposit_event(Event::<T>::BidAccepted { task_id, bidder });

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

	pub fn do_reject_work(who: AccountOf<T>, task_id: TaskId) -> Result<(), DispatchError> {
		ensure!(<AcceptedBid<T>>::contains_key(task_id), <Error<T>>::NotAllowed);

		let accepted_bid = <AcceptedBid<T>>::get(task_id).unwrap();

		ensure!(accepted_bid == who, <Error<T>>::NotAllowed);

		ensure!(<TaskStorage<T>>::contains_key(task_id), <Error<T>>::TaskDoesNotExist);

		let mut task = <TaskStorage<T>>::get(task_id).unwrap();

		let task_cost = task.get_cost();

		task.update_status(TaskStatus::Open);

		<AcceptedBid<T>>::remove(task_id);

		<TaskStorage<T>>::insert(task_id, task);

		let escrow_id = Self::escrow_account_id(task_id);

		T::Currency::transfer(&escrow_id, &who, task_cost, ExistenceRequirement::KeepAlive)?;

		Self::deposit_event(Event::<T>::WorkRejected { task_id, bidder: who });

		Ok(())
	}

	pub fn do_accept_work(who: AccountOf<T>, task_id: TaskId) -> Result<(), DispatchError> {
		ensure!(<AcceptedBid<T>>::contains_key(task_id), <Error<T>>::NotAllowed);

		let accepted_bid = <AcceptedBid<T>>::get(task_id).unwrap();

		ensure!(accepted_bid == who.clone(), <Error<T>>::NotAllowed);

		ensure!(<TaskStorage<T>>::contains_key(task_id), <Error<T>>::TaskDoesNotExist);
		let mut task = <TaskStorage<T>>::get(task_id).unwrap();

		task.update_status(TaskStatus::InProgress);
		task.add_worker(who.clone());

		// add deadline
		let completion =
			Self::get_current_block_number() + (task.get_deadline() as u32 * DAYS).into();

		task.extend_completion(Some(completion));

		<AcceptedBid<T>>::remove(task_id);
		<TaskStorage<T>>::insert(task_id, task);

		Self::reject_all_bids(task_id);

		Self::deposit_event(Event::<T>::WorkAccepted { task_id, bidder: who });

		Ok(())
	}

	pub fn do_complete_work(
		who: AccountOf<T>,
		task_id: TaskId,
		worker_attachments: Vec<u8>,
	) -> Result<(), DispatchError> {
		ensure!(<TaskStorage<T>>::contains_key(task_id), <Error<T>>::TaskDoesNotExist);
		let task = <TaskStorage<T>>::get(task_id).unwrap();

		ensure!(task.get_worker_details() == &Some(who.clone()), <Error<T>>::NotWorker);

		// TODO: deadline slashing logic

		let mut updated_task = task.clone();

		updated_task.update_status(TaskStatus::PendingApproval);
		updated_task.add_worker_attachments(worker_attachments);

		<TaskStorage<T>>::insert(task_id, updated_task);

		Self::deposit_event(Event::<T>::WorkCompleted { task_id, worker: who });

		Ok(())
	}

	pub fn do_complete_task(who: AccountOf<T>, task_id: TaskId) -> Result<(), DispatchError> {
		ensure!(<TaskStorage<T>>::contains_key(task_id), <Error<T>>::TaskDoesNotExist);
		let mut task = <TaskStorage<T>>::get(task_id).unwrap();

		ensure!(task.check_ownership(&who), <Error<T>>::NoPermission);

		ensure!(task.get_status() == &TaskStatus::AwaitingCompletion, <Error<T>>::NotAllowed);

		task.update_status(TaskStatus::Completed);

		let _escrow_account = Self::escrow_account_id(task_id);
		let worker = task.get_worker_details().clone().unwrap();
		let amount = task.get_cost() + task.get_cost();
		T::Currency::transfer(&who, &worker, amount, ExistenceRequirement::AllowDeath)?;

		Self::deposit_event(Event::<T>::TaskCompleted { task_id });
		Ok(())
	}

	pub fn get_current_block_number() -> BlockNumberOf<T> {
		let block_number = <frame_system::Pallet<T>>::block_number();
		block_number
	}

	pub fn reject_all_bids(task_id: TaskId) {
		let mut bidder_list = <BidderList<T>>::get(task_id);

		let escrow_id = Self::escrow_account_id(task_id);

		let task = <TaskStorage<T>>::get(task_id).unwrap();

		// first return all locked tokens
		for bidder in bidder_list.iter() {
			T::Currency::transfer(
				&escrow_id,
				&bidder,
				task.get_cost(),
				ExistenceRequirement::KeepAlive,
			);
		}
		// then clear the list
		bidder_list.clear();
		<BidderList<T>>::insert(task_id, bidder_list);
	}
}
