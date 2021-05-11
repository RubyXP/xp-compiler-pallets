#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_std::{convert::*, marker::PhantomData, str, vec::Vec};
    use move_compiler::generators::Generator;
    use xp_compiler::XpCompiler;

    // Main pallet config
    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    // Main pallet
    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(PhantomData<T>);

    // Calling outside of pallet(Extrensics)
    // TODO: Tweak weights
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a new account with an initial currency
        #[pallet::weight(50_000_000)]
        pub(super) fn create_account(
            origin: OriginFor<T>,
            address: u128,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            // Compile the script with our common interface
            // create_account(address)
            let res = Generator.create_account(&address.to_string()).unwrap();

            // Save the script to store so that it can be retrieved
            <AccountsStore<T>>::insert(&sender, res.as_bytes());

            // Trigger event
            Self::deposit_event(Event::AccountCreation(sender));

            Ok(().into())
        }


        /// Transfer funds from your account
        #[pallet::weight(70_000_000)]
        pub(super) fn transfer_funds(
            origin: OriginFor<T>,
            receiver: u128,
            amount: u64,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            // transfer_amount(receiver, amount)
            let res = Generator.transfer_amount(&receiver.to_string(), &amount.to_string()).unwrap();

            <AccountsStore<T>>::insert(&sender, res.as_bytes());
            Self::deposit_event(Event::TransferFund(sender, receiver, amount));
            Ok(().into())
        }
    }

    type RawScriptData = Vec<u8>;
    // Pallet storage
    #[pallet::storage]
    #[pallet::getter(fn accounts)]
    pub(super) type AccountsStore<T: Config> =
        StorageMap<_, Twox64Concat, T::AccountId, RawScriptData>;

    // Pallet events
    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        AccountCreation(T::AccountId),
        TransferFund(T::AccountId, u128, u64), // AccountId, Receiver, Ammount
    }

    // Pallet hooks
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub initial_accounts: Vec<(T::AccountId, RawScriptData)>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                initial_accounts: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            for acc in &self.initial_accounts {
                AccountsStore::<T>::insert(&acc.0, &acc.1)
            }
        }
    }
}

#[cfg(test)]
mod tests;
