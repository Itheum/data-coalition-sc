multiversx_sc::imports!();

use crate::config;
use crate::config::UserId;
use crate::dao;

#[multiversx_sc::module]
pub trait BoardModule: config::ConfigModule + dao::DaoModule {
    #[endpoint(setBoardMinStake)]
    fn set_board_min_stake_endpoint(&self, amount: BigUint) {
        self.require_caller_is_dao();
        let dao = self.blockchain().get_caller();

        self.stake_min_amount(&dao).set(&amount);
    }

    #[endpoint(setBoardMinStakeDuration)]
    fn set_board_min_stake_duration_endpoint(&self, duration: u64) {
        self.require_caller_is_dao();
        let dao = self.blockchain().get_caller();

        self.stake_min_duration_seconds(&dao).set(&duration);
    }

    #[endpoint(acceptBoardMember)]
    fn accept_board_member_endpoint(&self, address: ManagedAddress) {
        self.require_caller_is_dao();
        let dao = self.blockchain().get_caller();
        let member = self.users().get_user_id(&address);
        require!(member != 0, "member does not exist");

        // TODO: assert minimum stake

        self.board_members(&dao).insert(member);
    }

    #[storage_mapper("board:members")]
    fn board_members(&self, dao: &ManagedAddress) -> UnorderedSetMapper<UserId>;

    #[storage_mapper("board:stake_min_amount")]
    fn stake_min_amount(&self, dao: &ManagedAddress) -> SingleValueMapper<BigUint>;

    #[storage_mapper("board:stake_min_duration_seconds")]
    fn stake_min_duration_seconds(&self, dao: &ManagedAddress) -> SingleValueMapper<u64>;
}
