use super::*;

use frame_support::{
	ensure,
	sp_runtime::DispatchError,
	traits::{tokens::ExistenceRequirement, Currency},
};

use sp_std::vec::Vec;
use tasking_primitives::{time::*, TaskId};
use tasking_traits::{task::*, user::*};

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

	pub fn do_propose(who: AccountOf<T>, task_id: TaskId) -> Result<(), DispatchError> {
		// ensure that the task exist
		ensure!(<TaskStorage<T>>::contains_key(&task_id), <Error<T>>::TaskDoesNotExist);

		let task = <TaskStorage<T>>::get(task_id).unwrap();

		// ensure task is accepting bids
		ensure!(task.get_status() == &TaskStatus::Open, <Error<T>>::NotAllowed);

		// ensure that owner is not the bidder
		ensure!(!task.check_ownership(&who), <Error<T>>::OwnerCannotBid);

		let mut proposal_list = <ProposalList<T>>::get(task_id);

		// check if already bid
		ensure!(!proposal_list.contains(&who), <Error<T>>::BidAlreadyPlaced);

		proposal_list.insert(who.clone());

		<ProposalList<T>>::insert(task_id, proposal_list);

		Self::deposit_event(Event::<T>::BidPlaced { task_id, bidder: who });

		Ok(())
	}

	pub fn do_accept_proposal(
		who: AccountOf<T>,
		task_id: TaskId,
		bidder: AccountOf<T>,
	) -> Result<(), DispatchError> {
		ensure!(<TaskStorage<T>>::contains_key(task_id), <Error<T>>::TaskDoesNotExist);
		let mut task = <TaskStorage<T>>::get(task_id).unwrap();

		ensure!(task.check_ownership(&who), <Error<T>>::NoPermission);

		// // TODO: check if a bid is already pending
		ensure!(task.get_status() == &TaskStatus::Open, <Error<T>>::NotAllowed);
		// ensure!(!(<AcceptedProposal<T>>::contains_key(task_id)), <Error<T>>::NotAllowed);

		// get bidder list
		let mut proposal_list = <ProposalList<T>>::get(task_id);
		ensure!(proposal_list.remove(&bidder), <Error<T>>::BidDoesNotExist);

		<AcceptedProposal<T>>::insert(task_id, bidder.clone());
		task.update_status(TaskStatus::AwaitingBidderResponse);

		<TaskStorage<T>>::insert(task_id, task);
		<ProposalList<T>>::insert(task_id, proposal_list);

		Self::deposit_event(Event::<T>::BidAccepted { task_id, bidder });

		Ok(())
	}

	pub fn do_reject_proposal(
		who: AccountOf<T>,
		task_id: TaskId,
		bidder: AccountOf<T>,
	) -> Result<(), DispatchError> {
		ensure!(<TaskStorage<T>>::contains_key(task_id), <Error<T>>::TaskDoesNotExist);
		let mut task = <TaskStorage<T>>::get(task_id).unwrap();

		ensure!(task.check_ownership(&who), <Error<T>>::NoPermission);

		ensure!(
			task.get_status() == &TaskStatus::Open
				|| task.get_status() == &TaskStatus::AwaitingBidderResponse,
			<Error<T>>::NotAllowed
		);

		if AcceptedProposal::<T>::get(task_id) == Some(bidder.clone()) {
			task.update_status(TaskStatus::Open);
			AcceptedProposal::<T>::remove(task_id);
			<TaskStorage<T>>::insert(task_id, task);
		} else {
			let mut proposal_list = <ProposalList<T>>::get(task_id);
			ensure!(proposal_list.remove(&bidder), <Error<T>>::BidDoesNotExist);
			<ProposalList<T>>::insert(task_id, proposal_list);
		}
		Self::deposit_event(Event::<T>::BidRejected { task_id, bidder });
		Ok(())
	}

	pub fn retract_proposal(who: AccountOf<T>, task_id: TaskId) -> Result<(), DispatchError> {
		ensure!(<TaskStorage<T>>::contains_key(task_id), <Error<T>>::TaskDoesNotExist);

		let mut proposal_list = <ProposalList<T>>::get(task_id);

		ensure!(proposal_list.remove(&who), <Error<T>>::BidDoesNotExist);

		Self::deposit_event(Event::<T>::BidRemoved { task_id, bidder: who });

		Ok(())
	}

	pub fn do_reject_work(who: AccountOf<T>, task_id: TaskId) -> Result<(), DispatchError> {
		ensure!(<AcceptedProposal<T>>::contains_key(task_id), <Error<T>>::NotAllowed);

		let accepted_proposal = <AcceptedProposal<T>>::get(task_id).unwrap();

		ensure!(accepted_proposal == who, <Error<T>>::NotAllowed);

		ensure!(<TaskStorage<T>>::contains_key(task_id), <Error<T>>::TaskDoesNotExist);

		let mut task = <TaskStorage<T>>::get(task_id).unwrap();

		task.update_status(TaskStatus::Open);

		<AcceptedProposal<T>>::remove(task_id);

		<TaskStorage<T>>::insert(task_id, task);

		Self::deposit_event(Event::<T>::WorkRejected { task_id, bidder: who });

		Ok(())
	}

	pub fn do_accept_work(who: AccountOf<T>, task_id: TaskId) -> Result<(), DispatchError> {
		ensure!(<AcceptedProposal<T>>::contains_key(task_id), <Error<T>>::NotAllowed);

		let accepted_proposal = <AcceptedProposal<T>>::get(task_id).unwrap();

		ensure!(accepted_proposal == who.clone(), <Error<T>>::NotAllowed);

		ensure!(<TaskStorage<T>>::contains_key(task_id), <Error<T>>::TaskDoesNotExist);
		let mut task = <TaskStorage<T>>::get(task_id).unwrap();

		task.update_status(TaskStatus::InProgress);
		task.add_worker(who.clone());

		// add deadline
		let completion =
			Self::get_current_block_number() + (task.get_deadline() as u32 * DAYS).into();

		task.extend_completion(Some(completion));

		<AcceptedProposal<T>>::remove(task_id);
		<TaskStorage<T>>::insert(task_id, task);

		// escrow logic
		let escrow_account = Self::escrow_account_id(task_id);

		T::Currency::transfer(
			&who,
			&escrow_account,
			Self::get_bid_amount(),
			ExistenceRequirement::KeepAlive,
		)?;

		Self::reject_all_proposals(task_id);

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
		ensure!(
			task.get_status() == &TaskStatus::InProgress
				|| task.get_status() == &TaskStatus::InRevision,
			<Error<T>>::NotAllowed
		);

		// TODO: deadline slashing logic

		let mut updated_task = task.clone();

		updated_task.update_status(TaskStatus::PendingApproval);
		updated_task.add_worker_attachments(worker_attachments);

		<TaskStorage<T>>::insert(task_id, updated_task);

		Self::deposit_event(Event::<T>::WorkCompleted { task_id, worker: who });

		Ok(())
	}

	pub fn do_approve_work(
		who: AccountOf<T>,
		task_id: TaskId,
		worker_ratings: u8,
	) -> Result<(), DispatchError> {
		ensure!(<TaskStorage<T>>::contains_key(task_id), <Error<T>>::TaskDoesNotExist);
		let task = <TaskStorage<T>>::get(task_id).unwrap();

		ensure!(task.check_ownership(&who), <Error<T>>::NoPermission);

		ensure!(task.get_status() == &TaskStatus::PendingApproval, <Error<T>>::NotAllowed);

		ensure!(worker_ratings >= 1 && worker_ratings <= 5, <Error<T>>::InvalidRatingInput);

		let worker_address = task.get_worker_details().clone().unwrap();
		let mut worker = T::UserTrait::get_user_from_storage(&worker_address);
		worker.update_worker_rating(worker_ratings);

		T::UserTrait::save_user_to_storage(worker_address, worker);

		let mut updated_task = task.clone();

		updated_task.update_status(TaskStatus::CustomerRatingPending);

		<TaskStorage<T>>::insert(task_id, updated_task);

		Self::deposit_event(Event::<T>::TaskApproved { task_id, rating: worker_ratings });

		Ok(())
	}

	pub fn do_disapprove_work(who: AccountOf<T>, task_id: TaskId) -> Result<(), DispatchError> {
		ensure!(<TaskStorage<T>>::contains_key(task_id), <Error<T>>::TaskDoesNotExist);

		let mut task = <TaskStorage<T>>::get(task_id).unwrap();

		ensure!(task.check_ownership(&who), <Error<T>>::NoPermission);

		ensure!(task.get_status() == &TaskStatus::PendingApproval, <Error<T>>::NotAllowed);

		task.increment_revisions();
		task.update_status(TaskStatus::InRevision);

		<TaskStorage<T>>::insert(task_id, task);

		Self::deposit_event(Event::<T>::TaskDisapproved { task_id });

		Ok(())
	}

	pub fn do_provide_customer_rating(
		who: AccountOf<T>,
		task_id: TaskId,
		customer_rating: u8,
	) -> Result<(), DispatchError> {
		ensure!(<TaskStorage<T>>::contains_key(task_id), <Error<T>>::TaskDoesNotExist);
		let task = <TaskStorage<T>>::get(task_id).unwrap();

		ensure!(task.get_worker_details() == &Some(who.clone()), <Error<T>>::NoPermission);

		ensure!(task.get_status() == &TaskStatus::CustomerRatingPending, <Error<T>>::NotAllowed);

		ensure!(customer_rating >= 1 && customer_rating <= 5, <Error<T>>::InvalidRatingInput);

		let owner_address = task.get_owner_details().clone();
		let mut owner = T::UserTrait::get_user_from_storage(&owner_address);
		owner.update_publisher_rating(customer_rating);
		T::UserTrait::save_user_to_storage(owner_address, owner);

		let mut updated_task = task.clone();

		updated_task.update_status(TaskStatus::AwaitingCompletion);

		<TaskStorage<T>>::insert(task_id, updated_task);

		Self::deposit_event(Event::<T>::CustomerRatingProvided {
			task_id,
			rating: customer_rating,
		});

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
		let amount = task.get_cost() + Self::get_bid_amount();
		T::Currency::transfer(&who, &worker, amount, ExistenceRequirement::AllowDeath)?;

		Self::deposit_event(Event::<T>::TaskCompleted { task_id });
		Ok(())
	}

	pub fn get_current_block_number() -> BlockNumberOf<T> {
		let block_number = <frame_system::Pallet<T>>::block_number();
		block_number
	}

	pub fn reject_all_proposals(task_id: TaskId) {
		let mut proposal_list = <ProposalList<T>>::get(task_id);

		proposal_list.clear();
		<ProposalList<T>>::insert(task_id, proposal_list);
	}
}
