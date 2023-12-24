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
        self.lock_stake_for(user);
    }

    fn lock_stake_for(&self, user: UserId) {
        let lock_until = self.blockchain().get_block_timestamp() + self.stake_lock_time_seconds().get();
        self.stake_unlock_time(user).set(lock_until);
    }

    fn require_stake_unlocked_for(&self, user: UserId) {
        let current_time = self.blockchain().get_block_timestamp();
        let unlock_time = self.stake_unlock_time(user).get();
        require!(current_time > unlock_time, "stake is locked");
    }

    #[storage_mapper("stake:stakes")]
    fn stakes(&self, user: UserId) -> SingleValueMapper<BigUint>;

    #[view(getStakeUnlockTime)]
    #[storage_mapper("stake:unlock_time")]
    fn stake_unlock_time(&self, user: UserId) -> SingleValueMapper<u64>;

    #[view(getStakeLockTimeSeconds)]
    #[storage_mapper("stake:lock_time_seconds")]
    fn stake_lock_time_seconds(&self) -> SingleValueMapper<u64>;
}
