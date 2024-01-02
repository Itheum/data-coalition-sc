use crate::config;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait AggregateModule: config::ConfigModule {
    #[endpoint(initAggregateModule)]
    fn init_aggregate_module_endpoint(&self, data_aggregator: ManagedAddress) {
        self.require_caller_is_admin();
        self.data_aggregator().set(&data_aggregator);
    }

    fn register_aggregator_app(&self, name: ManagedBuffer) -> data_aggregator_proxy::AppId {
        require!(!self.data_aggregator().is_empty(), "data aggregator not set");
        let data_aggregator = self.data_aggregator().get();

        let app_id: data_aggregator_proxy::AppId = self
            .data_aggregator_contract(data_aggregator)
            .register_app_endpoint(name)
            .execute_on_dest_context();

        app_id
    }

    fn delegate_aggregator(
        &self,
        dao: ManagedAddress,
        delegator: ManagedAddress,
        category: ManagedBuffer,
        transfers: ManagedVec<EsdtTokenPayment>,
    ) -> AsyncCall {
        let data_aggregator = self.data_aggregator().get();
        let app_id = self.coalitions().get(&dao).unwrap();

        self.data_aggregator_contract(data_aggregator)
            .delegate_endpoint(app_id, category, OptionalValue::Some(delegator))
            .with_multi_token_transfer(transfers)
            .async_call()
    }

    fn undelegate_aggregator(&self, dao: ManagedAddress, nfts: MultiValueEncoded<MultiValue2<TokenIdentifier, u64>>) -> AsyncCall {
        let data_aggregator = self.data_aggregator().get();
        let app_id = self.coalitions().get(&dao).unwrap();

        self.data_aggregator_contract(data_aggregator).undelegate_endpoint(app_id, nfts).async_call()
    }

    fn handle_aggregator_undelegate_endpoint(&self, delegator: ManagedAddress, collection: TokenIdentifier, nonce: u64) {
        // TODO: implement
    }

    #[storage_mapper("aggregate:contract")]
    fn data_aggregator(&self) -> SingleValueMapper<ManagedAddress>;

    #[proxy]
    fn data_aggregator_contract(&self, to: ManagedAddress) -> data_aggregator_proxy::Proxy<Self::Api>;
}

pub mod data_aggregator_proxy {
    multiversx_sc::imports!();

    pub type AppId = u64;

    #[multiversx_sc::proxy]
    pub trait DataAggregatorContractProxy {
        #[endpoint(registerApp)]
        fn register_app_endpoint(&self, name: ManagedBuffer) -> AppId;

        #[payable("*")]
        #[endpoint(delegate)]
        fn delegate_endpoint(&self, app_id: AppId, segment: ManagedBuffer, user: OptionalValue<ManagedAddress>);

        #[endpoint(undelegate)]
        fn undelegate_endpoint(&self, app_id: AppId, nfts: MultiValueEncoded<MultiValue2<TokenIdentifier, u64>>);
    }
}
