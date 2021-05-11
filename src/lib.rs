#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_std::{convert::*, marker::PhantomData, str, vec::Vec};
    use solidity_compiler::generators::Generator;
    use sp_core::{H160, U256};

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
        /// Transfer funds from your account
        #[pallet::weight(70_000_000)]
        pub(super) fn transfer_funds(
            origin: OriginFor<T>,
            receiver: H160,
            amount: U256,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            // transfer_amount(receiver, amount)
            let mut amb: [u8; 32] = [0; 32];
            amount.to_big_endian(&mut amb);
            let call = Generator::payment_p2p_bytes(receiver.as_bytes(), &amb);

            <AccountsStore<T>>::insert(&sender, &call);
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
        TransferFund(T::AccountId, H160, U256), // AccountId, Currency Type, Receiver, Ammount
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
