#![no_std]

use aggregate::AggregatorAppId;

multiversx_sc::imports!();

pub mod aggregate;
pub mod category;
pub mod config;
pub mod dao;

#[multiversx_sc::contract]
pub trait DataCoalition: config::ConfigModule + dao::DaoModule + aggregate::AggregateModule + category::CategoryModule {
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}

    #[payable("*")]
    #[endpoint(create)]
    fn create_endpoint(&self) {
        let payment = self.call_value().single_esdt();
        let dao = self.create_dao(payment);
        let app_id = self.register_aggregator_app(&dao);

        self.coalitions().insert(dao, app_id);
    }

    #[storage_mapper("coalitions")]
    fn coalitions(&self) -> MapMapper<ManagedAddress, AggregatorAppId>;
}
