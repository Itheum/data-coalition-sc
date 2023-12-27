multiversx_sc::imports!();

use crate::config;
use crate::dao;
use crate::stake;

const BOARD_ROLE_NAME: &[u8] = b"board";

#[multiversx_sc::module]
pub trait BoardModule: config::ConfigModule + dao::DaoModule + stake::StakeModule {
    #[endpoint(setBoardMinStake)]
    fn set_board_min_stake_endpoint(&self, amount: BigUint) {
        self.require_caller_is_dao();
        let dao = self.blockchain().get_caller();

        self.board_stake_amount(&dao).set(&amount);
    }

    #[endpoint(setBoardStakeDuration)]
    fn set_board_min_stake_duration_endpoint(&self, duration: u64) {
        self.require_caller_is_dao();
        let dao = self.blockchain().get_caller();

        self.board_stake_duration(&dao).set(&duration);
    }

    #[endpoint(acceptBoardMember)]
    fn accept_board_member_endpoint(&self, address: ManagedAddress) {
        self.require_caller_is_dao();
        let dao = self.blockchain().get_caller();
        let user = self.users().get_or_create_user(&address);

        let user_stake = self.stakes(user).get();
        let board_stake_amount = self.board_stake_amount(&dao).get();
        require!(user_stake >= board_stake_amount, "insufficient stake");

        let current_time = self.blockchain().get_block_timestamp();
        let staked_locked_until = self.stake_unlock_time(user).get();
        let board_stake_duration = self.board_stake_duration(&dao).get();
        require!(staked_locked_until > current_time + board_stake_duration, "stake unlocks too early");

        self.add_board_member(dao, address);
    }

    #[endpoint(unlockBoardMember)]
    fn unlock_board_member_endpoint(&self, address: ManagedAddress) {
        self.require_caller_is_dao();
        let dao = self.blockchain().get_caller();
        let user = self.users().get_or_create_user(&address);
        require!(self.board_members(&dao).contains(&user), "not a board member");

        self.stake_unlock_time(user).set(0);
    }

    fn add_board_member(&self, dao: ManagedAddress, address: ManagedAddress) {
        let user = self.users().get_or_create_user(&address);
        let endpoint = ManagedBuffer::from(b"assignRole");
        let mut args = ManagedVec::new();
        args.push(ManagedBuffer::from(BOARD_ROLE_NAME));
        args.push(address.as_managed_buffer().clone());

        self.execute_unilateral_action(dao.clone(), endpoint, args, 10_000_000);

        self.board_members(&dao).insert(user);
    }

    fn configure_board_permissions(&self, dao: ManagedAddress) {
        let permission = ManagedBuffer::from(b"interactCoalition");
        let contract = self.blockchain().get_sc_address();
        let endpoint = ManagedBuffer::new();
        self.create_permission(dao.clone(), permission.clone(), BigUint::zero(), contract, endpoint, ManagedVec::new());

        let role = ManagedBuffer::from(BOARD_ROLE_NAME);
        self.create_policy(dao, dao::PolicyMethod::Majority, role, permission, BigUint::zero(), 0);
    }

    #[storage_mapper("board:stake_amount")]
    fn board_stake_amount(&self, dao: &ManagedAddress) -> SingleValueMapper<BigUint>;

    #[storage_mapper("board:stake_min_duration_seconds")]
    fn board_stake_duration(&self, dao: &ManagedAddress) -> SingleValueMapper<u64>;
}
