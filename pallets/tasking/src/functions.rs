use super::*;

use frame_support::{
	ensure,
	sp_runtime::{DispatchError, DispatchResult},
	traits::{tokens::ExistenceRequirement, Currency},
};
use frame_system::{ensure_signed, pallet_prelude::OriginFor};
use sp_std::vec::Vec;
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
		<TaskStorage<T>>::insert(task_id, task);
		<TaskNumber<T>>::set(task_id);
		Self::deposit_event(Event::<T>::TaskCreated { task_id, owner: who });
		Ok(())
	}

	pub fn get_current_block_number() -> BlockNumberOf<T> {
		let block_number = <frame_system::Pallet<T>>::block_number();
		block_number
	}
}
