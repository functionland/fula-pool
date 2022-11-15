use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

/// Type used for a unique identifier of each pool.
pub type PoolId = u32;

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct Pool<T: frame_system::Config, BoundedString> {
    pub name: BoundedString,
    pub owner: T::AccountId,
    pub parent: Option<PoolId>,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo, MaxEncodedLen)]
pub struct User {
    pub pool_id: Option<PoolId>,
    pub join_requested: bool,
}

#[frame_support::pallet]
pub mod pallet {
    use crate::{Pool, PoolId, User};
    use frame_support::pallet_prelude::*;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    /// The module configuration trait.
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>>
            + IsType<<Self as frame_system::Config>::RuntimeEvent>
            + TryInto<Event<Self>>;

        /// The maximum length of a name or symbol stored on-chain.
        #[pallet::constant]
        type StringLimit: Get<u32>;
    }

    /// Maximum number of pools that can exist. If `None`, then an unbounded number of
    /// pools can exist.
    #[pallet::storage]
    pub type MaxPools<T: Config> = StorageValue<_, u16, OptionQuery>;

    /// Maximum number of members that may belong to a pool. If `None`, then the count of
    /// members is not bound on a per pool basis.
    #[pallet::storage]
    pub type MaxPoolMembersPerPool<T: Config> = StorageValue<_, u16, OptionQuery>;

    #[pallet::storage]
    pub type Pools<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        PoolId,
        Blake2_128Concat,
        T::AccountId,
        Pool<T, BoundedVec<u8, T::StringLimit>>,
        OptionQuery,
    >;

    #[pallet::storage]
    pub type Users<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, User>;

    /// The events of this pallet.
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A pool has been created.
        PoolCreated {
            owner: T::AccountId,
            pool_id: PoolId,
        },

        /// A user requested to join a pool.
        JoinRequested { user: T::AccountId, pool_id: PoolId },

        /// A user has been accepted to the pool
        Accepted { user: T::AccountId, pool_id: PoolId },

        /// A user has been denied access to the pool.
        Denied {
            stash: T::AccountId,
            pool_id: PoolId,
        },
        /// Pool's capacity has been reached,
        CapacityReached { pool_id: PoolId },
    }
}
