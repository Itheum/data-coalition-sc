#![no_std]

multiversx_sc::imports!();

pub mod aggregate;
pub mod dao;

#[multiversx_sc::contract]
pub trait DataCoalition: dao::DaoModule + aggregate::AggregateModule {
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {
        self.init();
    }
}
