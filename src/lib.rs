#![no_std]

multiversx_sc::imports!();

pub mod dao;

#[multiversx_sc::contract]
pub trait DataCoalition: dao::DaoModule {
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {
        self.init();
    }
}
