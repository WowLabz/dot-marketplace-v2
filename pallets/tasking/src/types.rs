use super::*;
use frame_support::traits::Currency;

pub type AccountOf<T> = <T as frame_system::Config>::AccountId;
pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

pub type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;
