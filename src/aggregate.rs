multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait AggregateModule {
    #[endpoint(initAggregateModule)]
    fn init_aggregate_module_endpoint(&self, data_aggregator: ManagedAddress) {
        self.data_aggregator().set(&data_aggregator);
    }

    fn register_aggregator_app(&self, dao: ManagedAddress) {
        require!(!self.data_aggregator().is_empty(), "data aggregator not set");
        let data_aggregator = self.data_aggregator().get();

        let _: () = self
            .data_aggregator_contract(data_aggregator)
            .register_app_endpoint(dao)
            .execute_on_dest_context();
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
        fn register_app_endpoint(&self, address: ManagedAddress);
    }
}
