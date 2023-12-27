multiversx_sc::imports!();

use crate::config;
use crate::config::UserId;

#[multiversx_sc::module]
pub trait StakeModule: config::ConfigModule {
    #[endpoint(stake)]
    fn stake_endpoint(&self, extra_lock_seconds: u64) {
        let payment = self.call_value().single_esdt();

        require!(!self.native_token().is_empty(), "stake token not set");
        require!(payment.token_identifier == self.native_token().get(), "invalid stake token");
        require!(payment.token_nonce == 0, "stake token must be fungible");

        let caller = self.blockchain().get_caller();
        let user = self.users().get_or_create_user(&caller);

        self.stakes(user).update(|stake| *stake += payment.amount);
        self.lock_stake_for(user, extra_lock_seconds);
    }

    #[endpoint(unstake)]
    fn unstake_endpoint(&self) {
        let caller = self.blockchain().get_caller();
        let user = self.users().get_or_create_user(&caller);
        let native_token = self.native_token().get();
        let stake = self.stakes(user).get();

        require!(stake > 0, "no stake to unstake");
        self.require_stake_unlocked_for(user);

        self.stakes(user).clear();
        self.send().direct_esdt(&caller, &native_token, 0, &stake);
    }

    fn lock_stake_for(&self, user: UserId, extra_lock_seconds: u64) {
        let lock_until = self.blockchain().get_block_timestamp() + self.stake_lock_time_seconds().get() + extra_lock_seconds;
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
