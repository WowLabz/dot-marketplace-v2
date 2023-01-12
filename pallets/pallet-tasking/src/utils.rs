// Random shuffling using Fischer-Yates modern method & Linear Congruential Generator

// use parity_scale_codec::alloc::string::ToString;
#[allow(unused_imports)]
use num_traits::float::Float;
use sp_std::vec::Vec;

use crate::Bid;
use codec::alloc::string::{FromUtf8Error, String, ToString};

pub fn create_milestone_id(project_id: u128, milestone_number: u8) -> Vec<u8> {
	let mut arr = project_id.to_string();
	arr.push((97 + milestone_number) as char);
	let arr = arr.as_bytes().to_vec();
	arr
}

pub fn get_milestone_and_project_id(
	milestone_id: &mut Vec<u8>,
) -> Result<(u8, u128), FromUtf8Error> {
	let milestone_number = milestone_id.pop().unwrap() - 97;
	let project_id: &[u8] = milestone_id;
	let project_id = String::from_utf8(project_id.to_vec())?;

	let project_number: u128 = project_id.parse::<u128>().unwrap();
	Ok((milestone_number, project_number))
}

pub fn roundoff(total_rating: u8, number_of_users: u8) -> u8 {
	// For carrying the result
	let output: u8;
	// Calculating the average rating in floating point value
	let avg_rating: f32 = total_rating as f32 / number_of_users as f32;
	// Converting floating point to integer
	let rounded_avg_rating: u8 = avg_rating as u8;
	// Removing the decimal from float
	let fraction = avg_rating.fract();

	// ----- Result at different conditions
	if rounded_avg_rating != 0 {
		if fraction >= 0.5 {
			output = rounded_avg_rating + 1;
		} else {
			output = rounded_avg_rating;
		}
	} else {
		output = 0;
	}
	// -----

	output
}

pub fn check_for_previous_bid<Balance, AccountId: Eq + PartialEq>(
	vector_of_bids: &Vec<Bid<Balance, AccountId>>,
	new_bid: &Bid<Balance, AccountId>,
) -> bool {
	let mut res = false;
	for current_bid in vector_of_bids {
		if current_bid.bidder_id == new_bid.bidder_id {
			res = true;
			break
		}
	}
	res
}
