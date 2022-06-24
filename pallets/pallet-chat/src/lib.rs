#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use frame_support::{
        traits::{ 
            LockableCurrency 
        },
    };
	use sp_std::vec::Vec;
	
	// type AccountId<T> = <T as frame_system::Config>::AccountId;
	// type Balance<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	
	#[derive(Encode, Decode, Default, PartialEq, Eq, Debug, Clone, TypeInfo)]
	pub struct Message<AccountId> {
		pub message_id: u128,
		pub sender_id: AccountId,
		pub receiver_id: AccountId,
		pub message: Vec<u8>,
		pub reply: Option<Vec<u8>>,
		pub status: Status

	}

	impl<AccountId> Message<AccountId>{
		fn _new(self) -> Self {
			Self {
				message_id: self.message_id,
				sender_id: self.sender_id,
				receiver_id: self.receiver_id,
				message: self.message,
				reply: self.reply,
				status: self.status
			}
		}
	}

	#[derive(Encode, Decode, PartialEq, Eq, Debug, Clone, TypeInfo)]
	pub enum Status {
		Active,
		Replied,
		Closed
	}

	impl Default for Status {
		fn default() -> Self {
			Status::Active
		}
	}

	/// Pallet configuration 
    #[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: LockableCurrency<Self::AccountId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn get_message_count)]
    /// For storing the number of tasks
	pub type MessageCount<T> = StorageValue<_, u128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_message)]
	pub(super) type MsgStorage<T: Config> = StorageMap<_, Blake2_128Concat, u128, Message<T::AccountId>, ValueQuery>;
	
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		MessageCreated(u128,T::AccountId, T::AccountId),
		MessageReplied(u128,T::AccountId, T::AccountId),
		MessageClosed(u128,T::AccountId, T::AccountId),

	}

	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		/// Receiver should be a valid recipient and not the same as sender
		ReceiverNotValid,
		/// To make sure message exists
		MessageDoesNotExist,
		/// To make sure message is active
		ReplyAlreadyExists,
		/// To make sure only the receiver replies to the message
		UnauthorisedToReply,
		/// To make sure only the original sender can  read the reply
		UnauthorisedToClose,
		/// To make sure a reply exists
		ReplyDoesNotExist
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::weight(100)]
		pub fn write_message(
			origin: OriginFor<T>,
			receiver: T::AccountId, 
			message: Vec<u8>
		) -> DispatchResult {
			// User authentication.
			let sender = ensure_signed(origin)?;
			// Get message count.
			let message_count =  Self::get_message_count();
			// Ensure sender id is not same as receiver id.
			ensure!(sender != receiver,<Error<T>>::ReceiverNotValid);
			// Create message structure.
			let msg = Message {
				message_id: message_count,
				sender_id: sender.clone(),
				receiver_id: receiver.clone(),
				message: message,
				reply: None,
				status: Status::Active
			};
			// Update message storage.
			<MsgStorage<T>>::insert(&message_count,msg);
			// Update message count.
			<MessageCount<T>>::put(message_count + 1);
			// Notification.
			Self::deposit_event(
				Event::MessageCreated(
					message_count,
					sender,
					receiver
				)
			);

			Ok(())
		}

		#[pallet::weight(100)]
		pub fn reply_message(
			origin: OriginFor<T>, 
			message_id: u128, 
			reply: Vec<u8>
		) -> DispatchResult {
			// User authentication.
			let receiver = ensure_signed(origin)?;
			// Ensure message id exists.
			ensure! (<MsgStorage<T>>::contains_key(&message_id),<Error<T>>::MessageDoesNotExist);
			// Get message.
			let mut msg = Self::get_message(message_id.clone());
			// Ensure the recipient only replies.
			ensure! (receiver == msg.receiver_id,<Error<T>>::UnauthorisedToReply);
			// Ensure check to make sure message is active.
			ensure! (msg.status == Status::Active,<Error<T>>::ReplyAlreadyExists);
			
			// ----- Updating reply and status
			msg.reply = Some(reply);
			msg.status = Status::Replied;
			// -----
			
			// Cloning sender id.
			let original_sender = msg.sender_id.clone();
			// Updating message storage with updated message.
			<MsgStorage<T>>::insert(&message_id,msg);
			// Notification.
			Self::deposit_event(
				Event::MessageReplied(
					message_id, 
					receiver, 
					original_sender
				)
			);
			
			Ok(())
		}
		
		#[pallet::weight(100)]
		pub fn mark_as_read(
			origin: OriginFor<T>, 
			message_id: u128, 
			mode: bool
		) -> DispatchResult {
			// User authentication.
			let sender =  ensure_signed(origin)?;
			// Ensure if message exists.
			ensure!(
				<MsgStorage<T>>::contains_key(&message_id),
				<Error<T>>::MessageDoesNotExist
			);
			// Get message.
			let mut msg = Self::get_message(message_id.clone());
			// Ensure if original sender is the one reading the message.
			ensure! (sender == msg.sender_id,<Error<T>>::UnauthorisedToClose);
			// Ensure if msg was replied.
			ensure! (msg.status == Status::Replied,<Error<T>>::ReplyDoesNotExist);
			// Cloning receiver id
			let receiver = msg.receiver_id.clone();
			// Update status.
			msg.status = match mode {
				true => Status::Closed,
				false => Status::Replied
			
			};
			// If status is closed remove msg from storage
			if msg.status == Status::Closed {
				<MsgStorage<T>>::remove(message_id);
			}
			// Notification.
			Self::deposit_event(
				Event::MessageClosed(
					message_id, 
					sender, 
					receiver
				)
			);

			Ok(())
		}
	}
}

