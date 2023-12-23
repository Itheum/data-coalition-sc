use crate::config;

multiversx_sc::imports!();

pub type AggregatorAppId = u64;

#[multiversx_sc::module]
pub trait AggregateModule: config::ConfigModule {
    #[endpoint(initAggregateModule)]
    fn init_aggregate_module_endpoint(&self, data_aggregator: ManagedAddress) {
        self.require_caller_is_admin();
        self.data_aggregator().set(&data_aggregator);
    }

    fn register_aggregator_app(&self, dao: &ManagedAddress) -> AggregatorAppId {
        require!(!self.data_aggregator().is_empty(), "data aggregator not set");
        let data_aggregator = self.data_aggregator().get();

        let app_id: AggregatorAppId = self
            .data_aggregator_contract(data_aggregator)
            .register_app_endpoint(dao)
            .execute_on_dest_context();

        app_id
    }

    #[storage_mapper("aggregate:contract")]
    fn data_aggregator(&self) -> SingleValueMapper<ManagedAddress>;

    #[proxy]
    fn data_aggregator_contract(&self, to: ManagedAddress) -> data_aggregator_proxy::Proxy<Self::Api>;
}

mod data_aggregator_proxy {
    multiversx_sc::imports!();

    #[multiversx_sc::proxy]
    pub trait DataAggregatorContractProxy {
        #[endpoint(registerApp)]
        fn register_app_endpoint(&self, address: ManagedAddress) -> u64;
    }
}
