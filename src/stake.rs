multiversx_sc::imports!();

use crate::config;
use crate::config::UserId;

#[multiversx_sc::module]
pub trait StakeModule: config::ConfigModule {
    #[endpoint(stake)]
    fn stake_endpoint(&self) {
        let payment = self.call_value().single_esdt();

        require!(!self.native_token().is_empty(), "stake token not set");
        require!(payment.token_identifier == self.native_token().get(), "invalid stake token");
        require!(payment.token_nonce == 0, "stake token must be fungible");

        let caller = self.blockchain().get_caller();
        let user = self.users().get_or_create_user(&caller);

        self.stakes(user).update(|stake| *stake += payment.amount);
    }

    #[storage_mapper("stake:stakes")]
    fn stakes(&self, user: UserId) -> SingleValueMapper<BigUint>;
}
