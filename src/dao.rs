multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait DaoModule {
    #[endpoint(initDaoModule)]
    fn init_dao_module_endpoint(&self, dao_manager: ManagedAddress) {
        self.dao_manager().set(&dao_manager);
    }

    #[view(getDaoVoteWeight)]
    fn get_dao_vote_weight_view(&self, address: ManagedAddress, _token: OptionalValue<TokenIdentifier>) -> BigUint {
        // TODO: implement
        BigUint::zero()
    }

    #[view(getDaoMembers)]
    fn get_dao_members_view(&self, _token: OptionalValue<TokenIdentifier>) -> MultiValueEncoded<MultiValue2<ManagedAddress, BigUint>> {
        let members_multi = MultiValueEncoded::new();

        // TODO: implement

        members_multi.into()
    }

    fn create_dao(&self) -> ManagedAddress {
        require!(!self.dao_manager().is_empty(), "dao manager not set");
        let dao_manager = self.dao_manager().get();
        let features = MultiValueManagedVec::new();

        let dao: ManagedAddress = self
            .dao_manager_contract(dao_manager)
            .create_entity_endpoint(features)
            .execute_on_dest_context();

        self.daos().insert(dao.clone());

        dao
    }

    #[storage_mapper("dao:daos")]
    fn daos(&self) -> UnorderedSetMapper<ManagedAddress>;

    #[storage_mapper("dao:manager_contract")]
    fn dao_manager(&self) -> SingleValueMapper<ManagedAddress>;

    #[proxy]
    fn dao_manager_contract(&self, to: ManagedAddress) -> dao_manager_proxy::Proxy<Self::Api>;
}

mod dao_manager_proxy {
    multiversx_sc::imports!();

    #[multiversx_sc::proxy]
    pub trait DaoManagerContractProxy {
        #[payable("*")]
        #[endpoint(createEntity)]
        fn create_entity_endpoint(&self, features: MultiValueManagedVec<ManagedBuffer>) -> ManagedAddress;
    }
}
