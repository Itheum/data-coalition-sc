multiversx_sc::imports!();

use crate::config;
use crate::config::UserId;
use crate::dao;

const BOARD_ROLE_NAME: &[u8] = b"board";

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

        // TODO: assert minimum stake

        self.add_board_member(dao, address);
    }

    fn add_board_member(&self, dao: ManagedAddress, address: ManagedAddress) {
        let member = self.users().get_or_create_user(&address);
        let endpoint = ManagedBuffer::from(b"assignRole");
        let mut args = ManagedVec::new();
        args.push(ManagedBuffer::from(BOARD_ROLE_NAME));
        args.push(address.as_managed_buffer().clone());

        self.execute_unilateral_action(dao.clone(), endpoint, args, 10_000_000);

        self.board_members(&dao).insert(member);
    }

    fn configure_board_permissions(&self, dao: ManagedAddress) {
        let permission = ManagedBuffer::from(b"interactCoalition");
        let contract = self.blockchain().get_sc_address();
        let endpoint = ManagedBuffer::new();
        self.create_permission(dao.clone(), permission.clone(), BigUint::zero(), contract, endpoint, ManagedVec::new());

        let role = ManagedBuffer::from(BOARD_ROLE_NAME);
        self.create_policy(dao, dao::PolicyMethod::Majority, role, permission, BigUint::zero(), 0);
    }

    #[storage_mapper("board:members")]
    fn board_members(&self, dao: &ManagedAddress) -> UnorderedSetMapper<UserId>;

    #[storage_mapper("board:stake_min_amount")]
    fn stake_min_amount(&self, dao: &ManagedAddress) -> SingleValueMapper<BigUint>;

    #[storage_mapper("board:stake_min_duration_seconds")]
    fn stake_min_duration_seconds(&self, dao: &ManagedAddress) -> SingleValueMapper<u64>;
}
