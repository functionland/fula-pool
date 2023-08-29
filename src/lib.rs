// SBP-M1 review: missing license header

#![cfg_attr(not(feature = "std"), no_std)]

// SBP-M1 review: nest use statements
use codec::EncodeLike;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::traits::Defensive;
use frame_support::traits::Len;
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_std::fmt::Debug;

pub use pallet::*;

// SBP-M1 review: add doc comments
pub trait PoolInterface {
    type AccountId;
    type PoolId: Copy + TypeInfo + Debug + Eq + EncodeLike + Encode + Decode;
    // SBP-M1 review: take account by reference as only borrowed to check membership
    fn is_member(account: Self::AccountId, pool: Self::PoolId) -> bool;
}

// SBP-M1 review: group using statements together
use frame_support::{dispatch::DispatchResult, ensure, traits::Get, BoundedVec};
use sp_std::prelude::*;

// SBP-M1 review: duplicate import
pub use pallet::*;

// SBP-M1 review: consider moving structs and their impls to another module

/// Type used for a unique identifier of each pool.
// SBP-M1 review: move to config trait so runtime can decide on implementation
pub type PoolId = u32;

// SBP-M1 review: doesnt need to be pub
pub type BoundedStringOf<T> = BoundedVec<u8, <T as Config>::StringLimit>;

/// Pool
// SBP-M1 review: typos in comment
/// TODO: we we need an actual list of users in each pool? If so - we'll need to rething the storage
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
// SBP-M1 review: add generic parameters for only those types needed - e.g. AccountId, StringLimit, MaxPoolParticipants
pub struct Pool<T: Config> {
    /// Pool name, bounded by Config::StringLimit
    pub name: BoundedVec<u8, T::StringLimit>,
    /// Optional owner, there is no pool owner when a pool is created by the system.
    // SBP-M1 review: consider renaming to creator
    pub owner: Option<T::AccountId>,
    /// Optional parent, only set when a pool has been created by the system. Unset when the pool
    /// reaches at least 3 members.
    pub parent: Option<PoolId>,
    /// The current pool participants.
    // SBP-M1 review: consider whether a PoolParticipants storage double map (with () type as value) may be more efficient as state accumulates over time
    // SBP-M1 review: may eliminate binary_search when leaving pool and more easily check if account is member of a pool without reading whole pool state
    // SBP-M1 review: will require additional read however so important to benchmark when assessing viability
    pub participants: BoundedVec<T::AccountId, T::MaxPoolParticipants>,
    /// Number of outstanding join requests.
    pub request_number: u8,
    // SBP-M1 review: missing / to form doc comment
    // Region of the pool
    pub region: BoundedVec<u8, T::StringLimit>,
}

// SBP-M1 review: missing doc comments
impl<T: Config> Pool<T> {
    pub fn is_full(&self) -> bool {
        // SBP-M1 review: use safe math, depends on configuration of MaxPoolParticipants
        self.participants.len() + self.request_number as usize
            == T::MaxPoolParticipants::get() as usize
    }
}

/// User data for pool users. Created if a user has been previously unknown by the pool system, in
/// case of a new user trying to create or join a pool.
#[derive(Clone, Encode, Decode, Default, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
// SBP-M1 review: no type parameter T
#[scale_info(skip_type_params(T))]
pub struct User<BoundedString> {
    /// Optional PoolId, signifies membership in a pool.
    pub pool_id: Option<PoolId>,
    /// Signifies whether or not a user has a pending join request to a given pool. If this is set -
    /// the `pool_id` should be `None`.
    pub request_pool_id: Option<PoolId>,
    /// libp2p peerID validated on the client-side.
    // SBP-M1 review: consider changing to BoundedVec<u8, StringLimit> where StringLimit : Get<u32> so type less opaque
    pub peer_id: BoundedString,
}

impl<BoundedString> User<BoundedString> {
    /// Signifies whether or not a user can create or join a pool.
    pub(crate) fn is_free(&self) -> bool {
        self.pool_id.is_none() && self.request_pool_id.is_none()
    }
}

/// An enum that represents a vote result.
pub(crate) enum VoteResult {
    /// Majority voted for.
    Accepted,
    /// Majority voted against.
    Denied,
    /// Not conclusive yet.
    Inconclusive,
}

// SBP-M1 review: comment refers to invalid type
/// The current implementation of `PoolJoinRequest` only cares about positive votes and keeps track
/// of everyone that voted.
/// TODO: we might have to cover corner-cases, such as:
/// 1. When a user voted for somebody and left (possibly not the worst case)
/// 2. When a user left from the pool without voting, we can only recalculate this when another user
/// votes
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
// SBP-M1 review: add generic parameters for only those types needed - e.g. AccountId, StringLimit, MaxPoolParticipants
pub struct PoolRequest<T: Config> {
    // SBP-M1 review: comment refers to invalid type
    /// Prevents a user to vote twice on the same `PoolJoinRequest`.
    // SBP-M1 review: consider whether a PoolRequestVotes storage double map (with () type as value) may be more efficient depending on future pool sizes
    // SBP-M1 review: decoding a large vector just to check if an account has voted may become problematic
    // SBP-M1 review: additional reads have additional charges, so best to benchmark and assess carefully
    pub voted: BoundedVec<T::AccountId, T::MaxPoolParticipants>,
    /// Currently we only calculate positive votes to avoid having to iterate through voters map.
    /// We can easily calculate negative votes by taking the `voted` length and subtracting
    // SBP-M1 review: typo
    /// `positivte_votes` from it.
    pub positive_votes: u16,
    /// libp2p peerID validated on the client-side. A pre-requisite for voting
    pub peer_id: BoundedVec<u8, T::StringLimit>,
}

// SBP-M1 review: could potentially be replaced by adding Default to #[derive(...)] on struct
impl<T: Config> Default for PoolRequest<T> {
    fn default() -> Self {
        // SBP-M1 review: use Self
        PoolRequest {
            positive_votes: Default::default(),
            // SBP-M1 review: using BoundedVec::default() more clear
            voted: Default::default(),
            peer_id: Default::default(),
        }
    }
}

impl<T: Config> PoolRequest<T> {
    /// A method that checks whether or not a user has been accepted to a pool.
    // SBP-M1 review: use safe math
    pub(crate) fn check_votes(&self, num_participants: u16) -> VoteResult {
        // More than half of the participants voted for this user.
        if self.positive_votes > num_participants / 2 {
            return VoteResult::Accepted;
        }

        // More than half of the participants voted against this user.
        // SBP-M1 review: truncation via cast
        if self.voted.len() as u16 - self.positive_votes > num_participants / 2 {
            return VoteResult::Denied;
        }

        VoteResult::Inconclusive
    }
}

// TODO: Implement benchmarks for proper weight calculation
#[frame_support::pallet]
pub mod pallet {
    use crate::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_runtime::BoundedVec;

    #[pallet::pallet]
    // SBP-M1 review: generate_store is deprecated and should be removed
    #[pallet::generate_store(pub (super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    /// The module configuration trait.
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>>
            + IsType<<Self as frame_system::Config>::RuntimeEvent>
            + TryInto<Event<Self>>;

        /// The maximum length of a name or symbol stored on-chain. See if this can be limited to
        /// `u8::MAX`.
        #[pallet::constant]
        type StringLimit: Get<u32>;

        /// The maximum number of pool participants. We are aiming at `u8::MAX`.
        #[pallet::constant]
        type MaxPoolParticipants: Get<u32>;
    }

    /// An incremental value reflecting all pools created so far.
    #[pallet::storage]
    // SBP-M1 review: restrict visibility > pub(super)
    pub type LastPoolId<T: Config> = StorageValue<_, PoolId, ValueQuery>;

    /// Maximum number of pools that can exist. If `None`, then an unbounded number of
    /// pools can exist.
    #[pallet::storage]
    // SBP-M1 review: storage item is never set
    // SBP-M1 review: could read initial value from config via OnEmpty handler, then add permissioned dispatchable function which allows root to modify as required
    // SBP-M1 review: OptionQuery is the default and can therefore be omitted
    // SBP-M1 review: restrict visibility > pub(super)
    pub type MaxPools<T: Config> = StorageValue<_, PoolId, OptionQuery>;

    /// Pools storage
    #[pallet::storage]
    #[pallet::getter(fn pool)]
    // SBP-M1 review: OptionQuery is the default and can therefore be omitted
    // SBP-M1 review: empty pools are never removed
    // SBP-M1 review: restrict visibility > pub(super)
    pub type Pools<T: Config> = StorageMap<_, Blake2_128Concat, PoolId, Pool<T>, OptionQuery>;

    /// PoolRequests storage
    #[pallet::storage]
    #[pallet::getter(fn request)]
    // SBP-M1 review: restrict visibility > pub(super)
    pub type PoolRequests<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        PoolId,
        Blake2_128Concat,
        T::AccountId,
        PoolRequest<T>,
        // SBP-M1 review: OptionQuery is the default and can therefore be omitted
        OptionQuery,
    >;

    /// Users storage, useful in case a user wants to leave or join a pool.
    #[pallet::storage]
    #[pallet::getter(fn user)]
    // SBP-M1 review: users which leave are never removed
    // SBP-M1 review: restrict visibility > pub(super)
    pub type Users<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, User<BoundedStringOf<T>>>;

    /// The events of this pallet.
    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A pool has been created.
        PoolCreated {
            owner: Option<T::AccountId>,
            pool_id: PoolId,
        },

        /// A user requested to join a pool.
        JoinRequested {
            account: T::AccountId,
            pool_id: PoolId,
        },

        /// A user has withdrawn their request to join a pool.
        RequestWithdrawn {
            account: T::AccountId,
            pool_id: PoolId,
        },

        // SBP-M1 review: typo in comment
        // Results of the voting proccess
        VotingResult {
            account: T::AccountId,
            pool_id: PoolId,
            // SBP-M1 review: why not just use the VoteResult here? Events are technically still retained internally and should therefore be suitably optimised.
            result: Vec<u8>,
        },
        /// Pool's capacity has been reached,
        CapacityReached { pool_id: PoolId },

        /// Pool participant left.
        ParticipantLeft {
            account: T::AccountId,
            pool_id: PoolId,
        },
    }

    #[pallet::error]
    #[cfg_attr(test, derive(PartialEq))]
    pub enum Error<T> {
        /// User is already attached to a pool or has a pending join request.
        UserBusy,
        /// Maximum pool number has been reached.
        MaxPools,
        /// The pool name supplied was too long.
        NameTooLong,
        /// The pool does not exist.
        PoolDoesNotExist,
        /// The pool join request does not exist.
        RequestDoesNotExist,
        /// The pool is at max capacity.
        CapacityReached,
        /// The user does not exist.
        UserDoesNotExist,
        /// Access denied due to invalid data, e.g. user is trying to leave the pool that it does
        /// not belong to or vote without rights.
        AccessDenied,
        /// Internal error.
        InternalError,
        /// The user has already voted.
        // SBP-M1 review: slashing may not be required provided dispatchable functions are properly benchmarked and caller is therefore properly charged tx fees for a call despite no effect
        /// TODO: might be considered slashable behaviour as it wastes resources.
        AlreadyVoted,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Creates a new pool. `peer_id` is a libp2p peerID validated on the client-side.
        ///
        // SBP-M1 review: comment seems outdated - LastPoolId implies number of pools created (database id) rather than active pools
        /// TODO: Deposit; check the current pool number. Currently we check the PoolId to retrieve
        /// the pool number, but if we want to delete empty pools - then we need to retrieve the
        /// actual pool number from storage, for which a CountedMap should be used.
        // SBP-M1 review: missing #[pallet::call_index(..)] attribute
        // SBP-M1 review: implement benchmark and use resulting weight function
        #[pallet::weight(10_000)]
        pub fn create(
            origin: OriginFor<T>,
            // SBP-M1 review: use BoundedVec as parameter
            name: Vec<u8>,
            region: Vec<u8>,
            peer_id: BoundedVec<u8, T::StringLimit>,
        ) -> DispatchResult {
            let owner = ensure_signed(origin)?;
            let mut user = Self::get_or_create_user(&owner);

            ensure!(user.is_free(), Error::<T>::UserBusy);

            // SBP-M1 review: MaxPools is never set, making this redundant
            if let Some(max_pools) = MaxPools::<T>::get() {
                ensure!(max_pools > LastPoolId::<T>::get(), Error::<T>::MaxPools);
            }

            let pool_id = LastPoolId::<T>::mutate(|id| {
                // SBP-M1 review: use safe math > e.g. .saturating_inc()
                *id += 1;
                *id
            });

            // SBP-M1 review: can be omitted if using BoundedVec as parameter type
            let bounded_name: BoundedVec<u8, T::StringLimit> = name
                .clone()
                .try_into()
                .map_err(|_| Error::<T>::NameTooLong)?;

            // SBP-M1 review: can be omitted if using BoundedVec as parameter type
            let bounded_region: BoundedVec<u8, T::StringLimit> = region
                .clone()
                .try_into()
                .map_err(|_| Error::<T>::NameTooLong)?;

            let mut bounded_participants =
                // SBP-M1 review: BoundedVec::default() is sufficient
                BoundedVec::<T::AccountId, T::MaxPoolParticipants>::default();

            ensure!(
                bounded_participants.try_push(owner.clone()).is_ok(),
                Error::<T>::CapacityReached
            );

            let pool = Pool {
                // SBP-M1 review: name variables the same as struct field names and simplify the below
                name: bounded_name,
                region: bounded_region,
                owner: Some(owner.clone()),
                parent: None,
                participants: bounded_participants,
                request_number: 0,
            };
            // SBP-M1 review: use &pool_id to avoid clone
            Pools::<T>::insert(pool_id.clone(), pool);
            // SBP-M1 review: clone unnecessary due to type
            user.pool_id = Some(pool_id.clone());
            // SBP-M1 review: no peer_id validation (format/length or already used)
            user.peer_id = peer_id;
            // SBP-M1 review: use .insert to avoid Some(..)
            Users::<T>::set(&owner, Some(user));

            Self::deposit_event(Event::<T>::PoolCreated {
                pool_id,
                owner: Some(owner),
            });

            Ok(())
        }

        /// Allows for the user to leave a pool.
        // SBP-M1 review: missing #[pallet::call_index(..)] attribute
        // SBP-M1 review: implement benchmark and use resulting weight function
        #[pallet::weight(10_000)]
        pub fn leave_pool(origin: OriginFor<T>, pool_id: PoolId) -> DispatchResult {
            let account = ensure_signed(origin)?;

            let mut user = Self::get_user(&account)?;
            ensure!(
                // SBP-M1 review: 'user.pool_id == Some(pool_id)'
                // SBP-M1 review: unwrap may panic and should be avoided
                user.pool_id.is_some() && pool_id == user.pool_id.unwrap(),
                Error::<T>::AccessDenied
            );

            // SBP-M1 review: unnecessary borrow (copy)
            let mut pool: Pool<T> = Self::pool(&pool_id).ok_or(Error::<T>::PoolDoesNotExist)?;
            // SBP-M1 review: clone seems wasteful
            let mut participants = pool.participants.clone();

            // SBP-M1 review: consider PoolParticipants storage map
            match participants.binary_search(&account) {
                Ok(index) => {
                    // SBP-M1 review: pool should be removed when no participants - seems a caller can currently create and leave pools continuously, bloating chain state
                    participants.remove(index);
                    // SBP-M1 review: pool owner not reset if leaver is owner
                    pool.participants = participants;
                    // SBP-M1 review: use .insert to avoid Some(..)
                    // SBP-M1 review: unnecessary borrow (copy)
                    Pools::<T>::set(&pool_id, Some(pool));

                    user.pool_id = None;
                    // SBP-M1 review: use .insert to avoid Some(..)
                    // SBP-M1 review: user may never rejoin, leaving state forever, so should be removed
                    Users::<T>::set(&account, Some(user));

                    Self::deposit_event(Event::<T>::ParticipantLeft { pool_id, account });
                    Ok(())
                }
                // This should never happen, but if it does - what do we do? One option is to
                // deposit an error event. The problem here is that a user will be permanently stuck
                // in an inconsistent state due to the fact that they have a pool_id in their
                // profile, but they are not actually a member of a pool. This is a defensive check.
                Err(_) => {
                    frame_support::defensive!(
                        "a user is not a participant of the pool they are assigned to"
                    );
                    Err(Error::<T>::InternalError.into())
                }
            }
        }

        /// Open a `PoolRequest` to join the pool.
        // SBP-M1 review: missing #[pallet::call_index(..)] attribute
        // SBP-M1 review: implement benchmark and use resulting weight function
        // SBP-M1 review: should join requests have an expiry to allow for state cleanup? User could simply generate a new account to join another pool
        #[pallet::weight(10_000)]
        pub fn join(
            origin: OriginFor<T>,
            pool_id: PoolId,
            peer_id: BoundedVec<u8, T::StringLimit>,
        ) -> DispatchResult {
            let account = ensure_signed(origin)?;
            // SBP-M1 review: unnecessary borrow (copy)
            let mut pool = Self::pool(&pool_id).ok_or(Error::<T>::PoolDoesNotExist)?;

            ensure!(!pool.is_full(), Error::<T>::CapacityReached);

            let mut user = Self::get_or_create_user(&account);

            ensure!(user.is_free(), Error::<T>::UserBusy);

            user.request_pool_id = Some(pool_id);
            // SBP-M1 review: no peer_id validation (format/length or already used)
            // SBP-M1 review: can a participant take control of pool by generating accounts and re-using same peer_id
            user.peer_id = peer_id.clone();
            // SBP-M1 review: use .insert to avoid Some(..)
            Users::<T>::set(&account, Some(user));

            // SBP-M1 review: assign field via initializer
            let mut request = PoolRequest::<T>::default();
            request.peer_id = peer_id;
            // SBP-M1 review: unnecessary borrow (copy)
            PoolRequests::<T>::insert(&pool_id, &account, request);
            // SBP-M1 review: use safe math - e.g. .checked_add(1).expect("pool capacity checked above; qed")
            pool.request_number += 1;
            // SBP-M1 review: use .insert to avoid Some(..)
            // SBP-M1 review: unnecessary borrow (copy)
            Pools::<T>::set(&pool_id, Some(pool));

            Self::deposit_event(Event::<T>::JoinRequested { pool_id, account });
            Ok(())
        }

        /// Cancel a `PoolRequest`, useful if a user decides to join another pool or they are stuck in
        /// the voting queue for too long.
        // SBP-M1 review: missing #[pallet::call_index(..)] attribute
        // SBP-M1 review: implement benchmark and use resulting weight function
        // SBP-M1 review: consider taking a deposit to join a pool, which is returned when accepted/denied or when user cancels join to clean up state
        #[pallet::weight(10_000)]
        pub fn cancel_join(origin: OriginFor<T>, pool_id: PoolId) -> DispatchResult {
            let account = ensure_signed(origin)?;
            // SBP-M1 review: unnecessary borrow (copy)
            // SBP-M1 review: use .contains_key() as value is not used
            Self::request(&pool_id, &account).ok_or(Error::<T>::RequestDoesNotExist)?;
            let mut user = Self::user(&account).ok_or(Error::<T>::UserDoesNotExist)?;
            // SBP-M1 review: unnecessary borrow (copy)
            let pool = Self::pool(&pool_id).ok_or(Error::<T>::PoolDoesNotExist)?;

            user.request_pool_id = None;
            // SBP-M1 review: use .insert to avoid Some(..)
            Users::<T>::set(&account, Some(user));

            // SBP-M1 review: unnecessary borrow (copy)
            PoolRequests::<T>::remove(&pool_id, &account);

            // SBP-M1 review: already removed in preceding statement?
            Self::remove_pool_request(&account, pool_id, pool);

            Self::deposit_event(Event::<T>::RequestWithdrawn { pool_id, account });
            Ok(())
        }

        /// Vote for a `PoolRequest`. If `positive` is set to `false` - that's voting against.
        /// This method also calculates votes each time it's called and takes action once the result
        /// is conclusive.
        /// TODO: Currently does not cover pool overflow scenario and simply fails then.
        // SBP-M1 review: missing #[pallet::call_index(..)] attribute
        // SBP-M1 review: implement benchmark and use resulting weight function
        // SBP-M1 review: how are participants incentivized to actually vote? Perhaps percentage of join fee?
        #[pallet::weight(10_000)]
        pub fn vote(
            origin: OriginFor<T>,
            pool_id: PoolId,
            account: T::AccountId,
            // SBP-M1 review: consider less confusing name - e.g. 'enum Vote { Approve, Reject }' based on readme terminology
            positive: bool,
            peer_id: BoundedVec<u8, T::StringLimit>,
        ) -> DispatchResult {
            let voter = ensure_signed(origin)?;
            let voter_user = Self::get_user(&voter)?;

            let mut request =
                // SBP-M1 review: unnecessary borrow (copy)
                Self::request(&pool_id, &account).ok_or(Error::<T>::RequestDoesNotExist)?;

            ensure!(
                // SBP-M1 review: 'voter_user.pool_id == Some(pool_id)'
                // SBP-M1 review: unwrap may panic, handle gracefully
                voter_user.pool_id.is_some() && voter_user.pool_id.unwrap() == pool_id,
                Error::<T>::AccessDenied
            );

            ensure!(
                // SBP-M1 review: unnecessary .to_vec()
                request.peer_id.to_vec() == peer_id.to_vec(),
                Error::<T>::AccessDenied
            );

            let mut voted = request.voted.clone();

            match voted.binary_search(&voter) {
                Ok(_) => Err(Error::<T>::AlreadyVoted.into()),

                Err(index) => {
                    // This should never fail.
                    voted
                        .try_insert(index, voter.clone())
                        .map_err(|_| Error::<T>::InternalError)
                        .defensive()?;

                    // Increment votes if positive, we do all that here to be able to calculate the
                    // votes as we need to do it on every vote.
                    if positive {
                        // SBP-M1 review: use safe math for good measure
                        // SBP-M1 review: consider renaming to 'approvals' to align with readme terminology
                        request.positive_votes += 1;
                    }
                    request.voted = voted;

                    // This should never fail.
                    // SBP-M1 review: unnecessary borrow (copy)
                    let pool = Self::pool(&pool_id)
                        .ok_or(Error::<T>::PoolDoesNotExist)
                        .defensive()?;

                    // TODO: to be removed when we implement copy for pools.
                    ensure!(!pool.is_full(), Error::<T>::CapacityReached);

                    // SBP-M1 review: move request.check_votes() into process_vote_result() impl
                    // SBP-M1 review: possible truncation with cast
                    let result = request.check_votes(pool.participants.len() as u16);
                    Self::process_vote_result(&account, pool_id, pool, request, result)
                }
            }
        }
    }

    impl<T: Config> Pallet<T> {
        /// Get or create a user
        fn get_or_create_user(who: &T::AccountId) -> User<BoundedStringOf<T>> {
            if let Some(user) = Self::user(who) {
                return user;
            }
            let user = User::default();

            // SBP-M1 review: all usages of this function later store the user, so this seems redundant
            Users::<T>::insert(who, user.clone());

            user
        }

        fn get_user(who: &T::AccountId) -> Result<User<BoundedStringOf<T>>, DispatchError> {
            Self::user(who).ok_or(Error::<T>::UserDoesNotExist.into())
        }

        fn remove_pool_request(who: &T::AccountId, pool_id: PoolId, mut pool: Pool<T>) {
            PoolRequests::<T>::remove(pool_id, who);
            // SBP-M1 review: use safe math - e.g. saturating_dec()
            pool.request_number -= 1;
            // SBP-M1 review: use .insert to avoid Some(..)
            // SBP-M1 review: unnecessary borrow (copy)
            Pools::<T>::set(&pool_id, Some(pool));
        }

        // SBP-M1 review: function too long, needs refactoring
        fn process_vote_result(
            who: &T::AccountId,
            pool_id: PoolId,
            mut pool: Pool<T>,
            request: PoolRequest<T>,
            // SBP-M1 review: parameter not consumed, consider borrow or implement Copy
            result: VoteResult,
        ) -> DispatchResult {
            // SBP-M1 review: repeated code, have match statement return result so event emission defined once
            // SBP-M1 review: e.g. 'let vote_result = match result { .. };'
            match result {
                // If the user has been accepted - remove the PoolRequest, update the user to
                // link them to the pool and remove PoolRequest reference. Also add them to
                // pool participants.
                VoteResult::Accepted => {
                    // This should never fail.
                    let mut user = Self::get_user(who).defensive()?;

                    // SBP-M1 review: should this not use remove_pool_request as below?
                    PoolRequests::<T>::remove(pool_id, who);
                    let mut participants = pool.participants.clone();
                    match participants.binary_search(who) {
                        // should never happen
                        Ok(_) => Err(Error::<T>::InternalError.into()),
                        Err(index) => {
                            participants
                                .try_insert(index, who.clone())
                                .map_err(|_| Error::<T>::InternalError)
                                .defensive()?;
                            // SBP-M1 review: unnecessary clone (copy)
                            user.pool_id = Some(pool_id.clone());
                            user.request_pool_id = None;
                            // SBP-M1 review: unnecessary .into()
                            user.peer_id = request.peer_id.into();
                            // SBP-M1 review: use .insert() to avoid Some()
                            Users::<T>::set(who, Some(user));

                            pool.participants = participants;
                            Self::remove_pool_request(who, pool_id, pool);
                            // SBP-M1 review: use VoteResult enum rather than string
                            let result = "Accepted";

                            Self::deposit_event(Event::<T>::VotingResult {
                                pool_id,
                                account: who.clone(),
                                result: result.as_bytes().to_vec(),
                            });
                            Ok(())
                        }
                    }
                    .defensive()
                }
                // If the user has been denied access to the pool - remove the PoolRequest
                // and it's reference from the user profile.
                VoteResult::Denied => {
                    let mut user = Self::get_user(who).defensive()?;
                    user.request_pool_id = None;
                    Users::<T>::set(who, Some(user));

                    Self::remove_pool_request(who, pool_id, pool);
                    // SBP-M1 review: use VoteResult enum rather than string
                    let result = "Denied";

                    Self::deposit_event(Event::<T>::VotingResult {
                        pool_id,
                        account: who.clone(),
                        result: result.as_bytes().to_vec(),
                    });

                    Ok(())
                }
                // If the vote result is inconclusive - just set the incremented vote count
                // and add the voter to the PoolRequest.
                VoteResult::Inconclusive => {
                    // SBP-M1 review: unnecessary borrow (copy)
                    PoolRequests::<T>::set(&pool_id, who, Some(request));
                    // SBP-M1 review: use VoteResult enum rather than string
                    let result = "Inconclusive";
                    // SBP-M1 review: consider whether VotingResult event emission required on every vote (i.e. whilst inconclusive)
                    Self::deposit_event(Event::<T>::VotingResult {
                        pool_id,
                        account: who.clone(),
                        result: result.as_bytes().to_vec(),
                    });
                    Ok(())
                }
            }
        }
    }

    impl<T: Config> PoolInterface for Pallet<T> {
        type AccountId = T::AccountId;
        type PoolId = PoolId;

        fn is_member(account: Self::AccountId, pool: Self::PoolId) -> bool {
            // SBP-M1 review: alternative approach 'Self::user(&account).map_or(false, |u| u.pool_id == Some(pool))'
            Self::user(&account)
                .map(|u| u.pool_id.iter().any(|&v| v == pool))
                .unwrap_or(false)
        }
    }
}
