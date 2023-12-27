multiversx_sc::imports!();

use crate::config;
use crate::dao;
use crate::stake;

#[multiversx_sc::module]
pub trait CategoryModule: config::ConfigModule + dao::DaoModule + stake::StakeModule {
    #[endpoint(addCategory)]
    fn add_category_endpoint(&self, name: ManagedBuffer) {
        self.require_caller_is_dao();
        let dao = self.blockchain().get_caller();
        require!(!self.categories(&dao).contains(&name), "category already exists");

        self.categories(&dao).insert(name);
    }

    #[endpoint(removeCategory)]
    fn remove_category_endpoint(&self, name: ManagedBuffer) {
        self.require_caller_is_dao();
        let dao = self.blockchain().get_caller();
        self.require_category_exists(&dao, &name);

        self.categories(&dao).swap_remove(&name);
    }

    fn require_category_exists(&self, dao: &ManagedAddress, name: &ManagedBuffer) {
        require!(self.categories(&dao).contains(name), "category does not exist");
    }

    #[storage_mapper("category:categories")]
    fn categories(&self, dao: &ManagedAddress) -> UnorderedSetMapper<ManagedBuffer>;
}
