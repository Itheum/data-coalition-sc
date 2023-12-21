#![no_std]

multiversx_sc::imports!();

#[multiversx_sc::contract]
pub trait DataCoalitionContract {
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {
        self.init();
    }
}
