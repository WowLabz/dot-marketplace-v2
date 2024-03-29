//use super::*;
use crate::{Message, Status, Error, mock::*};
use frame_support::{assert_ok, assert_noop, dispatch::DispatchError};

#[test]
fn test_write_message() {
    new_test_ext().execute_with(|| {

		assert_noop!(
			Chat::write_message(
				Origin::none(),
				2, 
				b"Hello there!".to_vec()
			),
			DispatchError::BadOrigin,
		);

		assert_noop!(
			Chat::write_message(
				Origin::signed(1),
				1,
				b"Hello there!".to_vec()
			),
			Error::<Test>::ReceiverNotValid
		);
 
		assert_ok!(
            Chat::write_message(
				Origin::signed(1),
				2,
				b"Hello there!".to_vec()
			)
        );
		
		
    });
}

#[test]
fn test_reply_message(){
	new_test_ext().execute_with(||{
		
		Chat::write_message(
			Origin::signed(1),
			2,
			b"Hello there!".to_vec()
		).unwrap();

		assert_noop!(
			Chat::reply_message(
				Origin::none(),
				0, 
				b"How are you?".to_vec()
			),
			DispatchError::BadOrigin,
		);

		assert_noop!(
			Chat::reply_message(
				Origin::signed(2),
				1, 
				b"How are you?".to_vec()
			),
			Error::<Test>::MessageDoesNotExist,
		);

		assert_noop!(
			Chat::reply_message(
				Origin::signed(1),
				0, 
				b"How are you?".to_vec()
			),				
			Error::<Test>::UnauthorisedToReply,
		);

		Chat::reply_message(
			Origin::signed(2),
			0,
			b"How are you?".to_vec()
		).unwrap();

		assert_noop!(
			Chat::reply_message(
				Origin::signed(2),
				0, 
				b"Hello there!".to_vec()
			),
			Error::<Test>::ReplyAlreadyExists,
		);


		

	});

	new_test_ext().execute_with(||{

		Chat::write_message(
			Origin::signed(1),
			2,
			b"Hello there!".to_vec()
		).unwrap();

		assert_ok!(
			Chat::reply_message(
			Origin::signed(2),
			0,
			b"How are you?".to_vec()
			)
        );
	
	});
}

#[test]
fn test_mark_as_read() {
	new_test_ext().execute_with(||{

		Chat::write_message(
			Origin::signed(1),
			2,
			b"Hello there!".to_vec()
		).unwrap();

		assert_noop!(
			Chat::mark_as_read(
				Origin::signed(1),
				0,
				true
			),
			Error::<Test>::ReplyDoesNotExist
		);

		Chat::reply_message(
			Origin::signed(2),
			0,
			b"How are you?".to_vec()
		).unwrap();

		assert_noop!(
			Chat::mark_as_read(
				Origin::none(),
				0,
				true
			),
			DispatchError::BadOrigin,			
		);

		assert_noop!(
			Chat::mark_as_read(
				Origin::signed(1),
				1,
				true
			),
			Error::<Test>::MessageDoesNotExist
		);

		assert_noop!(
			Chat::mark_as_read(
				Origin::signed(2),
				0,
				true
			),
			Error::<Test>::UnauthorisedToClose
		);

		assert_ok!(
			Chat::mark_as_read(
				Origin::signed(1),
				0,
				true
			)
		);
	
	});

}

#[test]
fn test_storage_schema(){
	new_test_ext().execute_with(||{

		Chat::write_message(
			Origin::signed(1),
			2,
			b"Hello there!".to_vec()
		).unwrap();

		let mut message = Message {
			message_id: 0,
			sender_id: 1,
			receiver_id: 2, 
			message: b"Hello there!".to_vec(),
			reply: None,
			status: Status::Active

		};

		assert_eq!(Chat::get_message(0), message);

		Chat::reply_message(
			Origin::signed(2),
			0,
			b"How are you?".to_vec()
		).unwrap();

		message.reply = Some(b"How are you?".to_vec());
		message.status = Status::Replied;

		assert_eq!(Chat::get_message(0), message);
	
	});
}

/// Stress test for creating message ..
/// from sender to receiver.
#[test]
fn stress_test_write_message() {
    new_test_ext().execute_with(|| {
        let number_of_messages: u128 = 1000000;
        let mut storage_msg_count: u128 = 0;
        for _ in 0..number_of_messages {
            Chat::write_message(
                Origin::signed(1), 
                2, 
                b"Hey there!".to_vec()
            ).unwrap();
            storage_msg_count = Chat::get_message_count();    
        }
        assert_eq!(number_of_messages, storage_msg_count);
    });
}




