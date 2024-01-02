#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub mod aggregate;
pub mod board;
pub mod category;
pub mod config;
pub mod dao;
pub mod stake;

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi)]
pub struct Info<M: ManagedTypeApi> {
    pub native_token: TokenIdentifier<M>,
    pub aggregator: ManagedAddress<M>,
    pub aggregator_app: u64,
    pub categories: ManagedVec<M, ManagedBuffer<M>>,
    pub delegators: usize,
    pub board_stake_amount: BigUint<M>,
    pub board_stake_duration: u64,
    pub stake_lock_time: u64,
    pub user_stake: BigUint<M>,
    pub user_stake_unlocks_at: u64,
}

#[multiversx_sc::contract]
pub trait DataCoalition:
    config::ConfigModule + dao::DaoModule + aggregate::AggregateModule + category::CategoryModule + board::BoardModule + stake::StakeModule
{
    #[init]
    fn init(&self) {
        let caller = self.blockchain().get_caller();
        self.admins().insert(caller);
    }

    #[upgrade]
    fn upgrade(&self) {}

    #[payable("*")]
    #[endpoint(create)]
    fn create_endpoint(&self, name: ManagedBuffer, native_token: TokenIdentifier, stake_lock_time: u64) {
        let caller = self.blockchain().get_caller();
        let payment = self.call_value().single_esdt();
        let dao = self.create_dao(payment);
        let app_id = self.register_aggregator_app(name, dao.clone());

        self.coalitions().insert(dao.clone(), app_id);
        self.configure_dao_categories(&dao);
        self.configure_staking(&dao, native_token, stake_lock_time);
        self.configure_plug(dao.clone());
        self.add_board_member(dao.clone(), caller);
        self.configure_board_permissions(dao);
    }

    #[endpoint(createExternal)]
    fn create_external_endpoint(&self, dao: ManagedAddress, name: ManagedBuffer, native_token: TokenIdentifier, stake_lock_time: u64) {
        let app_id = self.register_aggregator_app(name, dao.clone());

        self.coalitions().insert(dao.clone(), app_id);
        self.configure_dao_categories(&dao);
        self.configure_staking(&dao, native_token, stake_lock_time);
    }

    // #[payable("*")]
    // #[endpoint(execute)]
    // fn execute_endpoint(&self, destination: ManagedAddress, endpoint: ManagedBuffer) {
    //     self.require_caller_is_dao();
    //     let egld_value = self.call_value().egld_value().clone_value();
    //     let transfers = self.call_value().all_esdt_transfers();

    //     let mut call = self.send().contract_call::<()>(destination, endpoint);

    //     call.push_raw_argument()

    //     if egld_value > 0 {
    //         call.with_egld_transfer(egld_value).transfer_execute();
    //     }

    //     call.with_multi_token_transfer(transfers.clone_value()).transfer_execute();
    // }

    #[payable("*")]
    #[endpoint(grantAccess)]
    fn grant_access_endpoint(&self, dao: ManagedAddress, category: ManagedBuffer) {
        self.require_category_exists(&dao, &category);
        let caller = self.blockchain().get_caller();
        let transfers = self.call_value().all_esdt_transfers().clone_value();

        self.delegate_aggregator(dao.clone(), category, transfers.clone())
            .with_callback(self.callbacks().grant_access_callback(caller, transfers, dao))
            .call_and_exit();
    }

    #[callback]
    fn grant_access_callback(
        &self,
        original_caller: ManagedAddress,
        original_payments: ManagedVec<EsdtTokenPayment<Self::Api>>,
        dao: ManagedAddress,
        #[call_result] result: ManagedAsyncCallResult<()>,
    ) {
        let back_transfers = self.blockchain().get_back_transfers();

        if !back_transfers.esdt_payments.is_empty() {
            self.send().direct_multi(&original_caller, &back_transfers.esdt_payments);
        }

        let user = self.users().get_user_id(&original_caller);

        match result {
            ManagedAsyncCallResult::Ok(()) => {
                self.delegators(&dao).insert(user);
                self.delegations_amount(&dao, user).update(|val| *val += BigUint::from(original_payments.len()));
            }
            ManagedAsyncCallResult::Err(_) => {}
        };
    }

    #[endpoint(revokeAccess)]
    fn revoke_access_endpoint(&self, dao: ManagedAddress, collection: TokenIdentifier, nonce: u64) {
        let caller = self.blockchain().get_caller();

        self.undelegate_aggregator(dao.clone(), collection, nonce)
            .with_callback(self.callbacks().revoke_access_callback(caller, dao))
            .call_and_exit();
    }

    #[callback]
    fn revoke_access_callback(&self, original_caller: ManagedAddress, dao: ManagedAddress, #[call_result] result: ManagedAsyncCallResult<()>) {
        let user = self.users().get_user_id(&original_caller);

        match result {
            ManagedAsyncCallResult::Ok(()) => {
                let amount = BigUint::from(1u8);
                self.delegations_amount(&dao, user).update(|val| *val -= amount);
                if self.delegations_amount(&dao, user).get() == 0 {
                    self.delegators(&dao).swap_remove(&user);
                }
            }
            ManagedAsyncCallResult::Err(_) => {}
        };
    }

    #[view(getInfo)]
    fn get_info_view(&self, dao: ManagedAddress, address: OptionalValue<ManagedAddress>) -> Info<Self::Api> {
        let address = address.into_option().unwrap_or_default();
        let user = self.users().get_user_id(&address);

        Info {
            native_token: self.native_token().get(),
            aggregator: self.data_aggregator().get(),
            aggregator_app: self.coalitions().get(&dao).unwrap_or_default(),
            categories: self.categories(&dao).iter().collect(),
            delegators: self.delegators(&dao).len(),
            board_stake_amount: self.board_stake_amount(&dao).get(),
            board_stake_duration: self.board_stake_duration(&dao).get(),
            stake_lock_time: self.stake_lock_time_seconds(&dao).get(),
            user_stake: self.stakes(&dao, user).get(),
            user_stake_unlocks_at: self.stake_unlock_time(&dao, user).get(),
        }
    }
}
