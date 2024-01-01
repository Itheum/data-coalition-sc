use crate::aggregate::data_aggregator_proxy;

multiversx_sc::imports!();

pub type UserId = usize;

#[multiversx_sc::module]
pub trait ConfigModule {
    #[endpoint(initConfigModule)]
    fn init_config_module_endpoint(&self, native_token: TokenIdentifier) {
        self.require_caller_is_admin();
        self.native_token().set(&native_token);
    }

    #[endpoint(addAdmin)]
    fn add_admin_endpoint(&self, address: ManagedAddress) {
        self.require_caller_is_admin();
        self.admins().insert(address);
    }

    #[endpoint(removeAdmin)]
    fn remove_admin_endpoint(&self, address: ManagedAddress) {
        self.require_caller_is_admin();
        self.admins().swap_remove(&address);
    }

    fn require_caller_is_admin(&self) {
        let caller = self.blockchain().get_caller();
        let is_admin = self.admins().contains(&caller);
        let is_owner = self.blockchain().get_owner_address() == caller;
        require!(is_admin || is_owner, "caller must be admin");
    }

    #[storage_mapper("coalitions")]
    fn coalitions(&self) -> MapMapper<ManagedAddress, data_aggregator_proxy::AppId>;

    #[view(getAdmins)]
    #[storage_mapper("admins")]
    fn admins(&self) -> UnorderedSetMapper<ManagedAddress>;

    #[storage_mapper("users")]
    fn users(&self) -> UserMapper;

    #[storage_mapper("board_members")]
    fn board_members(&self, dao: &ManagedAddress) -> UnorderedSetMapper<UserId>;

    #[storage_mapper("delegators")]
    fn delegators(&self, dao: &ManagedAddress) -> UnorderedSetMapper<UserId>;

    #[view(getDaoWeight)]
    #[storage_mapper("delegations_amount")]
    fn delegations_amount(&self, dao: &ManagedAddress, user: UserId) -> SingleValueMapper<BigUint>;

    #[storage_mapper("native_token")]
    fn native_token(&self) -> SingleValueMapper<TokenIdentifier>;
}
