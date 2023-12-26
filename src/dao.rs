use crate::config;
use crate::stake;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

const DEFAULT_QUORUM: u64 = 1;
const DEFAULT_MIN_TO_PROPOSE: u64 = 1;
const DEFAULT_PLUG_WEIGHT_DECIMALS: u8 = 0;

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, PartialEq)]
pub enum PolicyMethod {
    Weight,
    One,
    All,
    Quorum,
    Majority,
}

#[multiversx_sc::module]
pub trait DaoModule: config::ConfigModule + stake::StakeModule {
    #[endpoint(initDaoModule)]
    fn init_dao_module_endpoint(&self, dao_manager: ManagedAddress) {
        self.require_caller_is_admin();
        self.dao_manager().set(&dao_manager);
    }

    #[view(getDaoVoteWeight)]
    fn get_dao_vote_weight_view(&self, address: ManagedAddress, _token: OptionalValue<TokenIdentifier>) -> BigUint {
        let dao = self.blockchain().get_caller();
        let user = self.users().get_user_id(&address);

        if user == 0 {
            return BigUint::zero();
        }

        self.delegations_amount(&dao, user).get()
    }

    #[view(getDaoMembers)]
    fn get_dao_members_view(&self, _token: OptionalValue<TokenIdentifier>) -> MultiValueEncoded<MultiValue2<ManagedAddress, BigUint>> {
        let members_multi = MultiValueEncoded::new();

        // TODO: implement

        members_multi.into()
    }

    fn create_dao(&self, payment: EsdtTokenPayment) -> ManagedAddress {
        require!(!self.dao_manager().is_empty(), "dao manager not set");
        let dao_manager = self.dao_manager().get();
        let features = MultiValueManagedVec::new();

        let dao: ManagedAddress = self
            .dao_manager_contract(dao_manager)
            .create_entity_endpoint(features)
            .with_esdt_transfer(payment)
            .execute_on_dest_context();

        self.daos().insert(dao.clone());

        dao
    }

    fn configure_plug(&self, dao: ManagedAddress) {
        let contract = self.blockchain().get_sc_address();
        let mut args = ManagedVec::new();
        args.push(contract.as_managed_buffer().clone());
        args.push(BigUint::from(DEFAULT_QUORUM).to_bytes_be_buffer());
        args.push(BigUint::from(DEFAULT_MIN_TO_PROPOSE).to_bytes_be_buffer());
        args.push(ManagedBuffer::from(&[DEFAULT_PLUG_WEIGHT_DECIMALS]));

        self.execute_unilateral_action(dao.clone(), ManagedBuffer::from(b"setPlug"), args, 10_000_000);
    }

    fn create_permission(
        &self,
        dao: ManagedAddress,
        permission_name: ManagedBuffer,
        value: BigUint,
        destination: ManagedAddress,
        endpoint: ManagedBuffer,
        payments: ManagedVec<EsdtTokenPayment>,
    ) {
        let mut args = ManagedVec::new();
        args.push(permission_name);
        args.push(value.to_bytes_be_buffer());
        args.push(destination.as_managed_buffer().clone());
        args.push(endpoint);

        for payment in payments.iter() {
            args.push(payment.token_identifier.as_managed_buffer().clone());
            args.push(payment.amount.to_bytes_be_buffer());
            args.push(ManagedBuffer::from(&[payment.token_nonce as u8]));
        }

        self.execute_unilateral_action(dao.clone(), ManagedBuffer::from(b"createPermission"), args, 10_000_000);
    }

    fn create_policy(
        &self,
        dao: ManagedAddress,
        method: PolicyMethod,
        role: ManagedBuffer,
        permission: ManagedBuffer,
        quorum: BigUint,
        voting_period_minutes: usize,
    ) {
        let endpoint = match method {
            PolicyMethod::Weight => ManagedBuffer::from(b"createPolicyWeighted"),
            PolicyMethod::One => ManagedBuffer::from(b"createPolicyOne"),
            PolicyMethod::All => ManagedBuffer::from(b"createPolicyAll"),
            PolicyMethod::Quorum => ManagedBuffer::from(b"createPolicyQuorum"),
            PolicyMethod::Majority => ManagedBuffer::from(b"createPolicyMajority"),
        };

        let mut args = ManagedVec::new();
        args.push(role);
        args.push(permission);

        if method == PolicyMethod::Weight {
            args.push(quorum.to_bytes_be_buffer());
            args.push(ManagedBuffer::from(&[voting_period_minutes as u8]));
        }

        if method == PolicyMethod::Quorum {
            args.push(ManagedBuffer::from(&[quorum.to_u64().unwrap() as u8]));
        }

        self.execute_unilateral_action(dao.clone(), endpoint, args, 10_000_000);
    }

    fn execute_unilateral_action(&self, dao: ManagedAddress, endpoint: ManagedBuffer, args: ManagedVec<ManagedBuffer>, gas_limit: u64) {
        let mut actions = MultiValueManagedVec::new();
        actions.push(dao_proxy::Action {
            destination: dao.clone(),
            endpoint,
            value: BigUint::zero(),
            payments: ManagedVec::new(),
            arguments: args,
            gas_limit,
        });

        self.dao_contract(dao).direct_execute_endpoint(actions).execute_on_dest_context::<()>();
    }

    fn require_caller_is_dao(&self) {
        let caller = self.blockchain().get_caller();
        require!(self.daos().contains(&caller), "caller must be dao");
    }

    #[storage_mapper("dao:daos")]
    fn daos(&self) -> UnorderedSetMapper<ManagedAddress>;

    #[storage_mapper("dao:manager_contract")]
    fn dao_manager(&self) -> SingleValueMapper<ManagedAddress>;

    #[proxy]
    fn dao_manager_contract(&self, to: ManagedAddress) -> dao_manager_proxy::Proxy<Self::Api>;

    #[proxy]
    fn dao_contract(&self, to: ManagedAddress) -> dao_proxy::Proxy<Self::Api>;
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

mod dao_proxy {
    multiversx_sc::imports!();
    multiversx_sc::derive_imports!();

    #[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, ManagedVecItem, Clone)]
    pub struct Action<M: ManagedTypeApi> {
        pub destination: ManagedAddress<M>,
        pub endpoint: ManagedBuffer<M>,
        pub value: BigUint<M>,
        pub payments: ManagedVec<M, EsdtTokenPayment<M>>,
        pub arguments: ManagedVec<M, ManagedBuffer<M>>,
        pub gas_limit: u64,
    }

    #[multiversx_sc::proxy]
    pub trait DaoContractProxy {
        #[endpoint(directExecute)]
        fn direct_execute_endpoint(&self, actions: MultiValueManagedVec<Action<Self::Api>>);
    }
}
