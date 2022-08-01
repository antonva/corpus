use crate as pallet_quadravote;
use frame_support::{
	dispatch::Vec,
	parameter_types,
	traits::{ConstU64, Everything, OnFinalize, OnInitialize},
};
use frame_system as system;
use pallet_balances;
use pallet_votingregistry;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub(crate) type AccountId = u64;
type Balance = u64;
// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system,
		Balances: pallet_balances,
		Quadravote: pallet_quadravote,
		VotingRegistry: pallet_votingregistry,
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
	type BaseCallFilter = Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

// Quadravote parameter types
parameter_types! {
	pub const PeriodLength: u32 = 5;
	pub const MaxProposals: u8 = 10;
	pub const MaxVotesPerAccount: u32 = 10;
}

impl pallet_quadravote::Config for Test {
	type Currency = ();
	type Event = Event;
	type IdentityProvider = VotingRegistry;
	type MaxProposals = MaxProposals;
	type PeriodLength = PeriodLength;
	type MaxVotesPerAccount = MaxVotesPerAccount;
}

impl pallet_votingregistry::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type ReserveAmount = ConstU64<50>;
}

impl pallet_balances::Config for Test {
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type Balance = Balance;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ConstU64<1>;
	type AccountStore = System;
	type WeightInfo = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}

// Helper function to fast forward to a specific block number.
pub fn run_to_block(n: u64) {
	while System::block_number() < n {
		if System::block_number() > 1 {
			Quadravote::on_finalize(System::block_number());
			System::on_finalize(System::block_number());
		}
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		Quadravote::on_initialize(System::block_number());
	}
}
