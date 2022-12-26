#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod utils;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		pallet_prelude::*,
		sp_runtime,
		sp_runtime::traits::{AccountIdConversion, SaturatedConversion},
		traits::{tokens::ExistenceRequirement, Currency, LockableCurrency},
		transactional, PalletId,
	};
	use frame_system::pallet_prelude::*;

	use crate::utils::{create_milestone_id, get_milestone_and_project_id, roundoff};
	use codec::{Decode, Encode};
	#[cfg(feature = "std")]
	use frame_support::serde::{Deserialize, Serialize};
	use sp_std::vec::Vec;

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	pub type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;

	#[derive(Encode, Decode, PartialEq, Eq, Debug, Clone, TypeInfo)]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub enum TaskTypeTags {
		WebDevelopment,
		MobileDevelopment,
		MachineLearning,
		DeepLearning,
		FullStackDevelopment,
		CoreBlockchainDevelopment,
	}

	#[derive(Encode, Decode, PartialEq, Eq, Debug, Clone, TypeInfo)]
	pub enum Status {
		Open,
		InProgress,
		PendingApproval,
		CustomerRatingPending,
		CustomerRatingProvided,
		Completed,
	}

	impl Default for Status {
		fn default() -> Self {
			Status::Open
		}
	}

	// Enum for the status of the Project
	#[derive(Encode, Decode, PartialEq, Eq, Debug, Clone, TypeInfo)]
	pub enum ProjectStatus {
		/// Project is ready to be submitted into the marketplace
		Ready,
		/// Project is Open
		Open,
		/// Project has been completed or closed
		Closed,
	}

	// Default for the project status
	impl Default for ProjectStatus {
		fn default() -> Self {
			ProjectStatus::Ready
		}
	}

	// a struct for the input of milestones
	#[derive(Encode, Decode, PartialEq, Eq, Debug, Clone, TypeInfo)]
	pub struct MilestoneHelper<Balance> {
		pub name: Vec<u8>,
		pub cost: Balance,
		pub tags: Vec<TaskTypeTags>,
		pub deadline: u8,
		pub publisher_attachments: Vec<Vec<u8>>,
	}

	// Struct for Project Description
	#[derive(Encode, Decode, Default, Debug, PartialEq, Clone, Eq, TypeInfo)]
	pub struct ProjectDetails<AccountId, Balance> {
		pub project_id: u128,
		pub publisher: AccountId,
		pub project_name: Vec<u8>,
		pub tags: Vec<TaskTypeTags>, // other approach possible
		pub publisher_name: Option<Vec<u8>>,
		pub milestones: Option<Vec<Milestone<AccountId, Balance>>>,
		pub overall_customer_rating: Option<u8>,
		pub status: ProjectStatus,
	}

	// Implementation for the Project Struct
	impl<AccountId, Balance> ProjectDetails<AccountId, Balance> {
		pub fn new(
			project_id: u128,
			project_name: Vec<u8>,
			tags: Vec<TaskTypeTags>,
			publisher: AccountId,
			publisher_name: Vec<u8>,
		) -> Self {
			ProjectDetails {
				project_id,
				project_name,
				tags,
				publisher,
				publisher_name: Some(publisher_name),
				milestones: None,
				overall_customer_rating: None,
				status: Default::default(),
			}
		}
	}

	// Milestone struct
	#[derive(Encode, Decode, Default, Debug, PartialEq, Clone, Eq, TypeInfo)]
	pub struct Milestone<AccountId, Balance> {
		pub milestone_id: Vec<u8>,
		pub milestone_name: Vec<u8>,
		pub tags: Vec<TaskTypeTags>,
		pub cost: Balance,
		pub status: Status,
		pub deadline: u8,
		pub worker_id: Option<AccountId>,
		pub worker_name: Option<Vec<u8>>,
		pub publisher_attachments: Option<Vec<Vec<u8>>>,
		pub worker_attachments: Option<Vec<Vec<u8>>>,
		pub final_worker_rating: Option<u8>,
		pub final_customer_rating: Option<u8>,
	}

	// Implementation of the milestone struct
	impl<AccountId, Balance> Milestone<AccountId, Balance> {
		fn new(
			milestone_id: Vec<u8>,
			milestone_name: Vec<u8>,
			tags: Vec<TaskTypeTags>,
			cost: Balance,
			deadline: u8,
			publisher_attachments: Vec<Vec<u8>>,
		) -> Self {
			Milestone {
				milestone_id,
				milestone_name,
				tags,
				cost,
				status: Default::default(),
				deadline,
				worker_id: None,
				worker_name: None,
				publisher_attachments: Some(publisher_attachments),
				worker_attachments: None,
				final_worker_rating: None,
				final_customer_rating: None,
			}
		}
	}

	// Bid struct
	#[derive(Encode, Decode, Default, Debug, PartialEq, Clone, Eq, TypeInfo)]
	pub struct Bid<Balance, AccountId> {
		pub bid_number: u32,
		pub bidder_id: AccountId,
		pub bidder_name: Vec<u8>,
		pub account: AccountDetails<Balance>,
	}

	impl<Balance, AccountId> Bid<Balance, AccountId> {
		pub fn new(
			bid_number: u32,
			bidder_id: AccountId,
			bidder_name: Vec<u8>,
			account: AccountDetails<Balance>,
		) -> Self {
			Bid { bid_number, bidder_id, bidder_name, account }
		}
	}

	#[derive(Encode, Decode, PartialEq, Eq, Debug, Clone, TypeInfo)]
	pub enum UserType {
		Customer,
		Worker,
	}

	impl Default for UserType {
		fn default() -> Self {
			UserType::Worker
		}
	}

	#[derive(Encode, Decode, Default, Debug, PartialEq, Clone, Eq, TypeInfo)]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub struct AccountDetails<Balance> {
		pub balance: Balance,
		pub ratings: Vec<u8>,
		pub avg_rating: Option<u8>,
		pub tags: Vec<TaskTypeTags>,
		pub sudo: bool,
	}

	impl<Balance> AccountDetails<Balance> {
		pub fn update_rating<T: Config>(account_id: T::AccountId, new_rating: u8) {
			let mut account_details = <AccountMap<T>>::get(account_id.clone());
			let mut all_ratings = account_details.ratings;
			all_ratings.push(new_rating);
			let avg_rating = Some(Self::get_list_average(all_ratings.clone()));
			account_details.avg_rating = avg_rating;
			account_details.ratings = all_ratings.clone();
			<AccountMap<T>>::insert(account_id, account_details);
		}

		pub fn get_list_average(list: Vec<u8>) -> u8 {
			let list_len: u8 = list.len() as u8;
			if list_len == 1 {
				return list[0]
			}
			let mut total_sum = 0;
			for item in list.iter() {
				total_sum += item;
			}

			roundoff(total_sum, list_len)
		}
	}

	#[derive(Encode, Decode, Default, Debug, PartialEq, Clone, Eq, TypeInfo)]
	pub struct TransferDetails<AccountId, Balance> {
		transfer_from: AccountId,
		from_before: Balance,
		from_after: Balance,
		transfer_to: AccountId,
		to_before: Balance,
		to_after: Balance,
	}

	/// Pallet configuration
	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: LockableCurrency<Self::AccountId>;
		type PalletId: Get<PalletId>;

		#[pallet::constant]
		type MaxMilestoneLimit: Get<u8>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn accounts)]
	pub type AccountMap<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, AccountDetails<BalanceOf<T>>, ValueQuery>;

	// * Genesis configuration for accounts
	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub account_map: Vec<(T::AccountId, AccountDetails<BalanceOf<T>>)>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> GenesisConfig<T> {
		/// Direct implementation of `GenesisBuild::build_storage`.
		///
		/// Kept in order not to break dependency.
		pub fn build_storage(&self) -> Result<sp_runtime::Storage, String> {
			<Self as GenesisBuild<T>>::build_storage(self)
		}

		/// Direct implementation of `GenesisBuild::assimilate_storage`.
		///
		/// Kept in order not to break dependency.
		pub fn assimilate_storage(&self, storage: &mut sp_runtime::Storage) -> Result<(), String> {
			<Self as GenesisBuild<T>>::assimilate_storage(self, storage)
		}
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { account_map: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			// Creating new accounts
			for (a, b) in &self.account_map {
				<AccountMap<T>>::insert(a, b);
			}
		}
	}

	#[pallet::storage]
	#[pallet::getter(fn get_project_count)]
	pub type ProjectCount<T> = StorageValue<_, u128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_project)]
	pub(super) type ProjectStorage<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		u128,
		ProjectDetails<T::AccountId, BalanceOf<T>>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn get_bidder_list)]
	pub type BidderList<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<Bid<BalanceOf<T>, T::AccountId>>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_count)]
	pub(super) type Count<T> = StorageValue<_, u128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_transfers)]
	pub(super) type Transfers<T: Config> =
		StorageValue<_, Vec<TransferDetails<T::AccountId, BalanceOf<T>>>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		AmountTransfered(T::AccountId, T::AccountId, BalanceOf<T>),
		AccBalance(T::AccountId, BalanceOf<T>),
		CountIncreased(u128),
		TransferMoney(
			T::AccountId,
			BalanceOf<T>,
			BalanceOf<T>,
			T::AccountId,
			BalanceOf<T>,
			BalanceOf<T>,
		),
		// CourtSummoned(u128, UserType, Reason, T::AccountId),
		// NewJurorAdded(u128, T::AccountId),
		// CustomerRatingProvided(u128, T::AccountId, u8, T::AccountId),
		VoteRecorded(u128, T::AccountId),
		/// A project has been created. [ProjectId, ProjectName, Publisher].
		ProjectCreated(u128, Vec<u8>, T::AccountId),
		/// A milestone has been created for some project. \[MilestoneId, \Cost].
		MileStoneCreated(Vec<u8>, BalanceOf<T>),
		/// A project has been added to marketplace. \[ProjectId]
		ProjectAddedToMarketplace(u128),
		/// A bid has been placed for some milestone. \[MilestoneId, AccountId]
		BidSuccessful(Vec<u8>, T::AccountId),
		/// A bid has been accepted for some milestone. \[MilestoneId, BidNumber]
		BidAccepted(Vec<u8>, u32),
		/// A milestone has been completed. \[MilestoneId]
		MilestoneCompleted(Vec<u8>),
		/// A milestone has been approved and a worker rating has been provided. \[MilestoneId,
		/// rating]
		MilestoneApproved(Vec<u8>, u8),
		/// Customer rating has been provided for a milestone. \[MilestoneId, rating]
		CustomerRatingProvided(Vec<u8>, u8),
		/// A milestone has been closed. \[MilestoneId]
		MilestoneClosed(Vec<u8>),
		/// A project has been closed. \[ProjectId]
		ProjectClosed(u128),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		/// To check the status and availibility of the task
		MilestoneNotOpen,
		/// To check balance of bidder for ability to stake amount
		NotEnoughBalanceToBid,
		/// To ensure publisher does not bid for the same task posted
		UnauthorisedToBid,
		/// To ensure that a task is bid for and is in progress
		TaskIsNotInProgress,
		/// To ensure a worker is chosen and is assigned to the task
		WorkerNotSet,
		/// To ensure only the assigned worker completes the task
		UnauthorisedToComplete,
		/// To ensure milestone status is completed and is waiting for approval from the publisher
		MilestoneNotPendingApproval,
		/// To ensure only the publisher approves the task
		UnauthorisedToApprove,
		/// To ensure milestone is approved by the publisher
		MilestoneNotPendingRating,
		/// To ensure the worker only provides the publisher rating
		UnauthorisedToProvideCustomerRating,
		/// To check if the sender has sufficient balance for a transfer
		NotEnoughBalance,
		/// To ensure approval is pending
		MilestoneInProgress,
		/// To ensure publisher is the one disapproving
		UnauthorisedToDisapprove,
		/// To ensure Customer Rating exists
		CustomerRatingNotProvided,

		// Phase 3 errors start here
		/// Maximum limit of u128 hit
		CannotCreateFurtherProjects,
		/// To ensure that project exists
		ProjectDoesNotExist,
		/// To ensure that project does not have more than 5 milestones
		MilestoneLimitReached,
		/// To ensure the publisher is making the transaction
		Unauthorised,
		/// The project should be ready before adding to the marketplace
		ProjectNotReady,
		/// Ensuring that the project is not closed
		ProjectClosed,
		/// Ensuring the milestone id is valid
		InvalidMilestoneId,
		/// Ensuring that project is open for bidding
		ProjectNotOpenForBidding,
		/// Ensuring that the milestone is open for bidding
		MilestoneNotOpenForBidding,
		/// Ensuring that the owner do not bid for the milestone
		PublisherCannotBid,
		/// Ensuring that the bid number is valid
		InvalidBidNumber,
		/// Something went wrong while transfering from escrow to account id
		FailedToTransferBack,
		/// Ensuring that the milestone is in progress
		MilestoneNotInProgress,
		/// Ensuring that the publisher does not attempt to complete the milestone
		PublisherCannotCompleteMilestone,
		/// Ensuring that the project is open before submission of milestone
		ProjectNotOpenForMilestoneCompletion,
		/// Project should be open while providing rating
		ProjectNotOpenToProvideRating,
		/// Publisher should not rate themself
		PublisherCannotRateSelf,
		/// The project cannot be closed for the following reasons : One or more milestones are not
		/// either open or completed
		CannotCloseProject,
		/// Project not open
		ProjectNotOpen,
		/// Raising a dispute for a milestone that has just been open
		MilestoneJustOpened,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		#[transactional]
		pub fn create_project(
			origin: OriginFor<T>,
			publisher_name: Vec<u8>,
			project_name: Vec<u8>,
			tags: Vec<TaskTypeTags>,
			milestone_one: MilestoneHelper<BalanceOf<T>>,
			add_milestones: Vec<MilestoneHelper<BalanceOf<T>>>,
		) -> DispatchResult {
			//function body starts here
			// ensuring that the transaction is signed and getting the account id of the transactor
			let publisher = ensure_signed(origin)?;
			ensure!(add_milestones.len() < 5, <Error<T>>::MilestoneLimitReached);
			let project_id = Self::get_project_count() + 1;
			<ProjectCount<T>>::set(project_id);
			let mut project = ProjectDetails::new(
				project_id,
				project_name.clone(),
				tags,
				publisher.clone(),
				publisher_name,
			);
			let mid = create_milestone_id(project_id, 0);
			let milestone1: Milestone<T::AccountId, BalanceOf<T>> = Milestone::new(
				mid,
				milestone_one.name,
				milestone_one.tags,
				milestone_one.cost,
				milestone_one.deadline,
				milestone_one.publisher_attachments,
			);
			let mut vector_of_milestones = Vec::new();
			vector_of_milestones.push(milestone1);
			for milestone_helper in add_milestones {
				let mid = create_milestone_id(project_id, vector_of_milestones.len() as u8);
				let milestone: Milestone<T::AccountId, BalanceOf<T>> = Milestone::new(
					mid.clone(),
					milestone_helper.name,
					milestone_helper.tags,
					milestone_helper.cost,
					milestone_helper.deadline,
					milestone_helper.publisher_attachments,
				);
				vector_of_milestones.push(milestone);
				Self::deposit_event(Event::MileStoneCreated(mid, milestone_helper.cost));
			}
			project.milestones = Some(vector_of_milestones.clone());
			<ProjectStorage<T>>::insert(&project_id, project);
			Self::collect_publisher_tokens(publisher.clone(), vector_of_milestones)?;
			Self::deposit_event(Event::ProjectCreated(project_id, project_name, publisher));
			Ok(())
			// function body ends here
		}

		#[pallet::weight(10_000)]
		#[transactional]
		pub fn add_milestones_to_project(
			origin: OriginFor<T>,
			project_id: u128,
			milestones: Vec<MilestoneHelper<BalanceOf<T>>>,
		) -> DispatchResult {
			// function body starts here

			//authentication
			let sender = ensure_signed(origin)?;
			<ProjectStorage<T>>::try_mutate(&project_id, |option_project| {
				let mut res = Ok(());
				match option_project {
					None => res = Err(<Error<T>>::ProjectDoesNotExist),
					Some(project) =>
						if project.publisher != sender {
							res = Err(<Error<T>>::Unauthorised);
						} else {
							match project.status {
								ProjectStatus::Closed => res = Err(<Error<T>>::ProjectClosed),
								_ => match &mut project.milestones {
									None => {
										let mut vector_of_milestones = Vec::new();
										for milestone_helper in milestones {
											let mid = create_milestone_id(
												project_id,
												vector_of_milestones.len() as u8,
											);
											let milestone: Milestone<T::AccountId, BalanceOf<T>> =
												Milestone::new(
													mid.clone(),
													milestone_helper.name,
													milestone_helper.tags,
													milestone_helper.cost,
													milestone_helper.deadline,
													milestone_helper.publisher_attachments,
												);
											vector_of_milestones.push(milestone);
											Self::deposit_event(Event::MileStoneCreated(
												mid,
												milestone_helper.cost,
											));
										}
										project.milestones = Some(vector_of_milestones);
									},
									Some(vector_of_milestones) => {
										if milestones.len() + vector_of_milestones.len() > 5 {
											res = Err(<Error<T>>::MilestoneLimitReached);
										} else {
											let mut vector_of_milestones_for_transfer = Vec::new();
											for milestone_helper in milestones {
												let mid = create_milestone_id(
													project_id,
													vector_of_milestones.len() as u8,
												);
												let milestone: Milestone<
													T::AccountId,
													BalanceOf<T>,
												> = Milestone::new(
													mid.clone(),
													milestone_helper.name,
													milestone_helper.tags,
													milestone_helper.cost,
													milestone_helper.deadline,
													milestone_helper.publisher_attachments,
												);
												vector_of_milestones.push(milestone.clone());
												vector_of_milestones_for_transfer.push(milestone);
												Self::deposit_event(Event::MileStoneCreated(
													mid,
													milestone_helper.cost,
												));
											}
											Self::collect_publisher_tokens(
												sender.clone(),
												vector_of_milestones_for_transfer,
											)?;
										}
									},
								},
							}
						},
				}
				res
			})?;
			Ok(())
		}

		#[pallet::weight(1000)]
		pub fn add_project_to_marketplace(
			origin: OriginFor<T>,
			project_id: u128,
		) -> DispatchResult {
			// function body starts here

			// authenticating
			let sender = ensure_signed(origin)?;

			<ProjectStorage<T>>::try_mutate(&project_id, |option| {
				let mut res = Ok(());
				match option {
					Some(project) =>
						if project.publisher != sender {
							res = Err(<Error<T>>::Unauthorised);
						} else if project.status != ProjectStatus::Ready {
							res = Err(<Error<T>>::ProjectNotReady);
						} else {
							project.status = ProjectStatus::Open;
							Self::deposit_event(Event::ProjectAddedToMarketplace(project_id));
						},
					None => res = Err(<Error<T>>::ProjectDoesNotExist),
				}
				res
			})?;

			Ok(())
			// function body ends here
		}

		#[pallet::weight(10_000)]
		#[transactional]
		pub fn bid_for_milestone(
			origin: OriginFor<T>,
			milestone_id: Vec<u8>,
			worker_name: Vec<u8>,
		) -> DispatchResult {
			// function body starts here

			// authentication
			let sender = ensure_signed(origin)?;
			let mut milestone_cost: BalanceOf<T> = 0u8.saturated_into();
			let mut milestone_id_clone = milestone_id.clone();
			let (milestone_number, project_id) =
				get_milestone_and_project_id(&mut milestone_id_clone)
					.map_err(|_| <Error<T>>::InvalidMilestoneId)?;
			// ensure that the project and milestone exists
			<ProjectStorage<T>>::try_mutate(&project_id, |option_project| {
				let mut res = Ok(());
				match option_project {
					Some(project) =>
						if project.status != ProjectStatus::Open {
							res = Err(<Error<T>>::ProjectNotOpenForBidding);
						} else if project.publisher == sender {
							res = Err(<Error<T>>::PublisherCannotBid);
						} else {
							match &mut project.milestones {
								Some(milestone_vector) => {
									if milestone_number >= milestone_vector.len() as u8 {
										res = Err(<Error<T>>::InvalidMilestoneId);
									} else if milestone_vector[milestone_number as usize].status !=
										Status::Open
									{
										res = Err(<Error<T>>::MilestoneNotOpenForBidding);
									} else {
										milestone_cost =
											milestone_vector[milestone_number as usize].cost;
									}
								},
								None => res = Err(<Error<T>>::InvalidMilestoneId),
							};
						},
					None => res = Err(<Error<T>>::ProjectDoesNotExist),
				}
				res
			})?;
			ensure!(
				T::Currency::free_balance(&sender) > milestone_cost,
				<Error<T>>::NotEnoughBalanceToBid
			);
			let account = Self::accounts(sender.clone());
			// let milestone_key = T::Hashing::hash_of(&milestone_id);
			<BidderList<T>>::mutate(&milestone_id, |bidder_vector| {
				// bid vector
				let bid_number = bidder_vector.len() as u32 + 1;
				let bid = Bid::new(bid_number, sender.clone(), worker_name, account);
				bidder_vector.push(bid);
			});
			let escrow_id = Self::get_escrow(milestone_id.clone());
			T::Currency::transfer(
				&sender,
				&escrow_id,
				milestone_cost,
				ExistenceRequirement::KeepAlive,
			)?;
			Self::deposit_event(Event::BidSuccessful(milestone_id, sender));
			Ok(())
			// function body ends here
		}

		#[pallet::weight(10_000)]
		#[transactional]
		pub fn accept_bid(
			origin: OriginFor<T>,
			milestone_id: Vec<u8>,
			bid_number: u32,
		) -> DispatchResult {
			// function body starts
			// authentication
			let sender = ensure_signed(origin)?;
			let mut milestone_cost: BalanceOf<T> = 0u8.saturated_into();
			let mut milestone_id_clone = milestone_id.clone();
			let (milestone_number, project_id) =
				get_milestone_and_project_id(&mut milestone_id_clone)
					.map_err(|_| <Error<T>>::InvalidMilestoneId)?;
			let bid_number = bid_number - 1_u32;
			<ProjectStorage<T>>::try_mutate(&project_id, |option_project| {
				let mut res = Ok(());
				// ensuring that the project exist
				match option_project {
					Some(project) => {
						// ensuring that the project publisher is making the request
						if project.publisher != sender {
							res = Err(<Error<T>>::Unauthorised);
						} else if project.status != ProjectStatus::Open {
							res = Err(<Error<T>>::ProjectNotOpenForBidding);
						} else {
							// ensuring that the project milestone exists and is open for bidding
							match &mut project.milestones {
								Some(milestone_vector) => {
									if milestone_number >= milestone_vector.len() as u8 {
										res = Err(<Error<T>>::InvalidMilestoneId);
									} else if milestone_vector[milestone_number as usize].status !=
										Status::Open
									{
										res = Err(<Error<T>>::MilestoneNotOpenForBidding);
									} else {
										// let milestone_key = T::Hashing::hash_of(&milestone_id);
										let bidder_list = Self::get_bidder_list(&milestone_id);
										if bidder_list.is_empty() ||
											bid_number >= bidder_list.len() as u32
										{
											res = Err(<Error<T>>::InvalidBidNumber);
										} else {
											// changing the status of the milestone to in progress
											milestone_vector[milestone_number as usize].status =
												Status::InProgress;
											// updating the worker id in the milestone
											milestone_vector[milestone_number as usize].worker_id =
												Some(
													bidder_list[bid_number as usize]
														.bidder_id
														.clone(),
												);
											// updating the worker name in th milestone
											milestone_vector[milestone_number as usize]
												.worker_name = Some(
												bidder_list[bid_number as usize]
													.bidder_name
													.clone(),
											);
											milestone_cost =
												milestone_vector[milestone_number as usize].cost;
										}
									}
								},
								None => res = Err(<Error<T>>::InvalidMilestoneId),
							}
						}
					},
					None => res = Err(<Error<T>>::ProjectDoesNotExist),
				};

				res
			})?;
			// locking the tokens of the publisher
			let escrow_id = Self::get_escrow(milestone_id.clone());
			// T::Currency::transfer(
			// 	&sender,
			// 	&escrow_id,
			// 	milestone_cost.clone(),
			// 	ExistenceRequirement::KeepAlive
			// )?;
			// let milestone_key = T::Hashing::hash_of(&milestone_id);
			// rejecting all the other bidders
			Self::reject_all(milestone_id.clone(), escrow_id, milestone_cost, bid_number)?;
			Self::deposit_event(Event::BidAccepted(milestone_id, bid_number));
			Ok(())
			// function body ends
		}

		#[pallet::weight(10_000)]
		pub fn complete_milestone(
			origin: OriginFor<T>,
			milestone_id: Vec<u8>,
			worker_attachments: Vec<Vec<u8>>,
		) -> DispatchResult {
			// function body starts here
			// authentication
			let sender = ensure_signed(origin)?;
			let mut milestone_id_clone = milestone_id.clone();
			let (milestone_number, project_id) =
				get_milestone_and_project_id(&mut milestone_id_clone)
					.map_err(|_| <Error<T>>::InvalidMilestoneId)?;
			<ProjectStorage<T>>::try_mutate(&project_id, |option_project| {
				let mut res = Ok(());
				match option_project {
					Some(project) => {
						if project.status != ProjectStatus::Open {
							res = Err(<Error<T>>::ProjectNotOpenForMilestoneCompletion);
						} else if sender == project.publisher {
							res = Err(<Error<T>>::PublisherCannotCompleteMilestone);
						} else {
							// checking the milestones
							match &mut project.milestones {
								Some(milestone_vector) => {
									if milestone_number >= milestone_vector.len() as u8 {
										res = Err(<Error<T>>::InvalidMilestoneId);
									} else if milestone_vector[milestone_number as usize].status !=
										Status::InProgress
									{
										res = Err(<Error<T>>::MilestoneNotInProgress);
									} else if milestone_vector[milestone_number as usize]
										.worker_id
										.clone()
										.unwrap() != sender
									{
										res = Err(<Error<T>>::UnauthorisedToComplete);
									} else {
										// updating the worker attachments
										milestone_vector[milestone_number as usize]
											.worker_attachments = Some(worker_attachments);
										// updating the status to pending approval
										milestone_vector[milestone_number as usize].status =
											Status::PendingApproval;
										Self::deposit_event(Event::MilestoneCompleted(
											milestone_id,
										));
									}
								},
								None => res = Err(<Error<T>>::InvalidMilestoneId),
							}
						}
					},
					None => res = Err(<Error<T>>::ProjectDoesNotExist),
				};
				res
			})?;
			Ok(())
			// function body ends here
		}

		#[pallet::weight(10_000)]
		pub fn approve_milestone(
			origin: OriginFor<T>,
			milestone_id: Vec<u8>,
			rating_for_the_worker: u8,
		) -> DispatchResult {
			// function body starts here
			// authentication
			let sender = ensure_signed(origin)?;
			let mut milestone_id_clone = milestone_id.clone();
			let (milestone_number, project_id) =
				get_milestone_and_project_id(&mut milestone_id_clone)
					.map_err(|_| <Error<T>>::InvalidMilestoneId)?;
			<ProjectStorage<T>>::try_mutate(&project_id, |option_project| {
				let mut res = Ok(());
				match option_project {
					Some(project) => {
						if project.publisher != sender {
							res = Err(<Error<T>>::UnauthorisedToApprove);
						} else if project.status != ProjectStatus::Open {
							res = Err(<Error<T>>::ProjectNotOpenToProvideRating);
						} else {
							// checking for milestones
							match &mut project.milestones {
								Some(milestone_vector) => {
									if milestone_number >= milestone_vector.len() as u8 {
										res = Err(<Error<T>>::InvalidMilestoneId);
									} else {
										match milestone_vector[milestone_number as usize].status {
											Status::PendingApproval => {
												milestone_vector[milestone_number as usize]
													.status = Status::CustomerRatingPending;
												milestone_vector[milestone_number as usize]
													.final_worker_rating = Some(rating_for_the_worker);
												Self::deposit_event(<Event<T>>::MilestoneApproved(
													milestone_id,
													rating_for_the_worker,
												));
											},
											_ => res = Err(<Error<T>>::MilestoneNotPendingApproval),
										}
									}
									// }else if milestone_vector[milestone_number as usize].status
									// != Status::PendingApproval { 	res = Err(<Error<T>>::
									// MilestoneNotPendingApproval); }else{
									// 	milestone_vector[milestone_number as usize].status =
									// Status::CustomerRatingPending;
									// 	milestone_vector[milestone_number as
									// usize].final_worker_rating = Some(rating_for_the_worker);
									// 	Self::deposit_event(Event::MilestoneApproved(milestone_id,
									// rating_for_the_worker)); }
								},
								None => res = Err(<Error<T>>::InvalidMilestoneId),
							};
						}
					},
					None => res = Err(<Error<T>>::ProjectDoesNotExist),
				};
				res
			})?;
			Ok(())
			// function body ends here
		}

		#[pallet::weight(10_000)]
		pub fn provide_customer_rating(
			origin: OriginFor<T>,
			milestone_id: Vec<u8>,
			rating_for_customer: u8,
		) -> DispatchResult {
			// function body starts here
			// authentication
			let sender = ensure_signed(origin)?;
			let mut milestone_id_clone = milestone_id.clone();
			let (milestone_number, project_id) =
				get_milestone_and_project_id(&mut milestone_id_clone)
					.map_err(|_| <Error<T>>::InvalidMilestoneId)?;
			<ProjectStorage<T>>::try_mutate(&project_id, |option_project| {
				let mut res = Ok(());
				match option_project {
					Some(project) => {
						if project.publisher == sender {
							res = Err(<Error<T>>::PublisherCannotRateSelf);
						} else if project.status != ProjectStatus::Open {
							res = Err(<Error<T>>::ProjectNotOpenToProvideRating);
						} else {
							// checking for milestone
							match &mut project.milestones {
								Some(vector_of_milestones) => {
									if milestone_number >= vector_of_milestones.len() as u8 {
										res = Err(<Error<T>>::InvalidMilestoneId);
									} else if vector_of_milestones[milestone_number as usize].status !=
										Status::CustomerRatingPending
									{
										res = Err(<Error<T>>::MilestoneNotPendingRating);
									} else if vector_of_milestones[milestone_number as usize]
										.worker_id
										.clone()
										.unwrap() != sender
									{
										res = Err(<Error<T>>::UnauthorisedToProvideCustomerRating);
									} else {
										vector_of_milestones[milestone_number as usize].status =
											Status::CustomerRatingProvided;
										vector_of_milestones[milestone_number as usize]
											.final_customer_rating = Some(rating_for_customer);
										Self::deposit_event(Event::CustomerRatingProvided(
											milestone_id,
											rating_for_customer,
										));
									}
								},
								None => res = Err(<Error<T>>::InvalidMilestoneId),
							}
						}
					},
					None => res = Err(<Error<T>>::ProjectDoesNotExist),
				};
				res
			})?;
			Ok(())
			// function body ends here
		}

		#[pallet::weight(10_000)]
		#[transactional]
		pub fn close_milestone(origin: OriginFor<T>, milestone_id: Vec<u8>) -> DispatchResult {
			// function body starts here
			// authentication
			let sender = ensure_signed(origin)?;
			let mut milestone_id_clone = milestone_id.clone();
			let (milestone_number, project_id) =
				get_milestone_and_project_id(&mut milestone_id_clone)
					.map_err(|_| <Error<T>>::InvalidMilestoneId)?;
			let worker_id = <ProjectStorage<T>>::try_mutate(&project_id, |option_project| {
				let res;
				match option_project {
					Some(project) => {
						if project.publisher != sender {
							res = Err(<Error<T>>::Unauthorised);
						} else if project.status != ProjectStatus::Open {
							res = Err(<Error<T>>::ProjectNotOpen);
						} else {
							match &mut project.milestones {
								Some(vector_of_milestones) => {
									if milestone_number >= vector_of_milestones.len() as u8 {
										res = Err(<Error<T>>::InvalidMilestoneId);
									} else if vector_of_milestones[milestone_number as usize].status !=
										Status::CustomerRatingProvided
									{
										res = Err(<Error<T>>::CustomerRatingNotProvided);
									} else {
										vector_of_milestones[milestone_number as usize].status =
											Status::Completed;
										let worker_id = vector_of_milestones
											[milestone_number as usize]
											.worker_id
											.clone()
											.unwrap();
										// 	// ----- Update overall customer rating
										<AccountDetails<BalanceOf<T>>>::update_rating::<T>(
											worker_id.clone(),
											vector_of_milestones[milestone_number as usize]
												.final_worker_rating
												.unwrap(),
										);
										// -----
										res = Ok(worker_id);
									}
								},
								None => res = Err(<Error<T>>::InvalidMilestoneId),
							}
						}
					},
					None => res = Err(<Error<T>>::ProjectDoesNotExist),
				}
				res
			})?;
			let escrow_id = Self::get_escrow(milestone_id.clone());
			let transfer_amount = T::Currency::free_balance(&escrow_id);
			T::Currency::transfer(
				&escrow_id,
				&worker_id,
				transfer_amount,
				ExistenceRequirement::AllowDeath,
			)?;

			Self::deposit_event(Event::MilestoneClosed(milestone_id));
			Ok(())
			// function body ends here
		}

		#[pallet::weight(10_000)]
		#[transactional]
		pub fn close_project(origin: OriginFor<T>, project_id: u128) -> DispatchResult {
			// function body starts here
			// authentication
			let sender = ensure_signed(origin)?;
			<ProjectStorage<T>>::try_mutate(&project_id, |option_project| {
				let mut res = Ok(());
				match option_project {
					Some(project) => {
						if sender != project.publisher {
							res = Err(<Error<T>>::Unauthorised);
						} else {
							match &mut project.milestones {
								Some(vector_of_milestones) => {
									let mut flag = false;
									for milestone in vector_of_milestones {
										match milestone.status {
											Status::Completed => flag = false,
											Status::Open => {
												// reject all the bidders
												// let milestone_key =
												// T::Hashing::hash_of(&milestone.milestone_id);
												let milestone_id = milestone.milestone_id.clone();
												let escrow_id = Self::get_escrow(
													milestone.milestone_id.clone(),
												);
												let cost = milestone.cost;
												let except = u32::MAX;
												// transferring publisher's token back to the
												// publisher
												T::Currency::transfer(
													&escrow_id,
													&sender,
													cost,
													ExistenceRequirement::AllowDeath,
												)
												.ok();
												// rejecting all the bidder
												res = Self::reject_all(
													milestone_id,
													escrow_id,
													cost,
													except,
												);
												flag = false;
											},
											_ => {
												flag = true;
												break
											},
										}
									}
									if flag {
										res = Err(<Error<T>>::CannotCloseProject);
									} else {
										let publisher_rating =
											Self::get_publisher_rating(project_id);
										// update publisher rating
										match publisher_rating {
											Some(rating) => {
												project.overall_customer_rating = Some(rating);
												<AccountDetails<BalanceOf<T>>>::update_rating::<T>(
													sender, rating,
												)
											},
											None => (),
										}
										project.status = ProjectStatus::Closed;
									}
								},
								None => {
									project.status = ProjectStatus::Closed;
									res = Ok(());
								},
							}
						}
					},
					None => res = Err(<Error<T>>::ProjectDoesNotExist),
				}
				res
			})?;
			Self::deposit_event(Event::ProjectClosed(project_id));
			Ok(())
			// function body ends here
		}

		#[pallet::weight(10_000)]
		pub fn transfer_money(
			origin: OriginFor<T>,
			to: T::AccountId,
			transfer_amount: BalanceOf<T>,
		) -> DispatchResult {
			// User authentication
			let sender = ensure_signed(origin)?;
			// Get total balance of sender
			let sender_account_balance = T::Currency::total_balance(&sender);
			// Verify if sender's balance is greater than transfer amount
			let is_valid_to_transfer = sender_account_balance < transfer_amount;
			// Is the transfer valid based on the sender's balance
			ensure!(!is_valid_to_transfer, <Error<T>>::NotEnoughBalance);
			// Get account balance of receiver
			let to_account_balance = T::Currency::total_balance(&to);
			// Making the transfer
			T::Currency::transfer(&sender, &to, transfer_amount, ExistenceRequirement::KeepAlive)?;
			// Get updated balance of sender
			let updated_sender_account_balance = T::Currency::total_balance(&sender);
			// Get updated balance of receiver
			let updated_to_account_balance = T::Currency::total_balance(&to);
			// Notify user about the increased transfer count
			Self::deposit_event(Event::CountIncreased(Self::get_count()));
			// Initializing a vec and storing the details is a Vec
			let mut details: Vec<TransferDetails<T::AccountId, BalanceOf<T>>> = Vec::new();
			// Preparing the transfer details structure
			let transfer_details = TransferDetails {
				transfer_from: sender.clone(),
				from_before: sender_account_balance,
				from_after: updated_sender_account_balance,
				transfer_to: to.clone(),
				to_before: to_account_balance,
				to_after: updated_to_account_balance,
			};
			// Updating the vector with transfer details
			details.push(transfer_details);
			// Updating storage with new transfer details
			<Transfers<T>>::put(details);
			// Notify event
			Self::deposit_event(Event::TransferMoney(
				sender,
				sender_account_balance,
				updated_sender_account_balance,
				to,
				to_account_balance,
				updated_to_account_balance,
			));

			Ok(())
		}
	}

	// Helper functions

	impl<T: Config> Pallet<T> {
		pub fn collect_publisher_tokens(
			publisher: T::AccountId,
			vector_of_milestone: Vec<Milestone<T::AccountId, BalanceOf<T>>>,
		) -> Result<(), Error<T>> {
			let mut total_cost: u32 = 0;
			let mut escrow_list = Vec::new();
			for milestone in vector_of_milestone {
				total_cost += milestone.cost.saturated_into::<u32>();
				let escrow_id = Self::get_escrow(milestone.milestone_id.clone());
				escrow_list.push((milestone.cost, escrow_id));
			}
			if T::Currency::free_balance(&publisher) < total_cost.into() {
				return Err(<Error<T>>::NotEnoughBalance)
			}
			for (cost, escrow_id) in escrow_list {
				T::Currency::transfer(
					&publisher,
					&escrow_id,
					cost,
					ExistenceRequirement::KeepAlive,
				)
				.map_err(|_| <Error<T>>::NotEnoughBalance)?;
			}
			Ok(())
		}

		pub fn get_escrow(id: Vec<u8>) -> T::AccountId {
			T::PalletId::get().into_sub_account(id)
		}

		// helper function to reject the other biddings and transfer the locked funds back to the
		// bidders
		pub fn reject_all(
			milestone_key: Vec<u8>,
			escrow_id: T::AccountId,
			cost: BalanceOf<T>,
			except: u32,
		) -> Result<(), Error<T>> {
			let bidder_list = Self::get_bidder_list(milestone_key.clone());
			for (index, bidder) in bidder_list.iter().enumerate() {
				if index as u32 == except {
					continue
				}
				let bidder_id = &bidder.bidder_id;
				T::Currency::transfer(
					&escrow_id,
					bidder_id,
					cost,
					ExistenceRequirement::AllowDeath,
				)
				.map_err(|_| <Error<T>>::FailedToTransferBack)?;
			}
			<BidderList<T>>::remove(&milestone_key);
			Ok(())
		}

		// helper function to get publisher rating
		pub fn get_publisher_rating(project_id: u128) -> Option<u8> {
			let project = Self::get_project(project_id).unwrap();
			let res;
			match project.milestones {
				Some(vector_of_milestones) => {
					let mut number_of_rating = 0;
					let mut total_rating = 0;
					for milestone in vector_of_milestones {
						match milestone.final_customer_rating {
							Some(rating) => {
								number_of_rating += 1;
								total_rating += rating;
							},
							None => (),
						}
					}
					if number_of_rating > 0 {
						res = Some(roundoff(total_rating, number_of_rating));
					} else {
						res = None;
					}
				},
				None => res = None,
			}
			res
		}
	}
}
