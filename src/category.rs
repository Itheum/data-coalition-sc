multiversx_sc::imports!();

use crate::{config, dao};

pub type CategoryName<M> = ManagedBuffer<M>;

#[multiversx_sc::module]
pub trait CategoryModule: config::ConfigModule + dao::DaoModule {
    #[endpoint(addCategory)]
    fn add_category_endpoint(&self, category: CategoryName<Self::Api>) {
        self.require_caller_is_dao();
        let dao = self.blockchain().get_caller();
        require!(!self.categories(&dao).contains(&category), "category already exists");

        self.categories(&dao).insert(category);
    }

    fn require_category_exists(&self, dao: &ManagedAddress, category: &CategoryName<Self::Api>) {
        require!(self.categories(&dao).contains(category), "category does not exist");
    }

    #[storage_mapper("category:categories")]
    fn categories(&self, dao: &ManagedAddress) -> UnorderedSetMapper<ManagedBuffer>;
}
