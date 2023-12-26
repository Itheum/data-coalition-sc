#![no_std]

multiversx_sc::imports!();

pub mod aggregate;
pub mod board;
pub mod category;
pub mod config;
pub mod dao;
pub mod stake;

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
    fn create_endpoint(&self, name: ManagedBuffer) {
        let caller = self.blockchain().get_caller();
        let payment = self.call_value().single_esdt();
        let dao = self.create_dao(payment);
        let app_id = self.register_aggregator_app(name, dao.clone());

        self.coalitions().insert(dao.clone(), app_id);
        self.configure_plug(dao.clone());
        self.add_board_member(dao.clone(), caller);
        self.configure_board_permissions(dao);
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
        let transfers = self.call_value().all_esdt_transfers();

        self.delegate_aggregator(dao, category, transfers.clone_value());
    }

    #[endpoint(revokeAccess)]
    fn revoke_access_endpoint(&self, dao: ManagedAddress, collection: TokenIdentifier, nonce: u64) {
        self.undelegate_aggregator(dao, collection, nonce);
    }
}
