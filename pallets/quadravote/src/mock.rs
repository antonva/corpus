use crate as pallet_quadravote;
use frame_support::{dispatch::Vec, parameter_types, traits::Everything};
use frame_system as system;
use pallet_identity;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub(crate) type AccountId = u64;
// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Quadravote: pallet_quadravote::{Pallet, Call, Storage, Event<T>},
		// TODO: mock this and remove the dependency
		VotingRegistry: pallet_votingregistry::{Pallet, Call, Storage, Event<T>}
	}
);

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

parameter_types! {
	pub const MaxRegistrars: u32 = 32;
	pub const MaxSubAccounts: u32 = 0;
	pub const SubAccountDeposit: u32 = 0;
	pub const MaxAdditionalFields: u32 = 2;
	pub const FieldDeposit: u32 = 0;
	pub const BasicDeposit: u32 = 0;
}

impl pallet_identity::Config for Test {
	type Event = Event;
	type Currency = ();
	type BasicDeposit = BasicDeposit;
	type FieldDeposit = FieldDeposit;
	type SubAccountDeposit = SubAccountDeposit;
	type MaxSubAccounts = MaxSubAccounts;
	type MaxAdditionalFields = MaxAdditionalFields;
	type MaxRegistrars = MaxRegistrars;
	type Slashed = (); // Don't do anything if account holders are slashed
	type ForceOrigin = system::EnsureRoot<AccountId>;
	type RegistrarOrigin = system::EnsureRoot<AccountId>;
	type WeightInfo = pallet_identity::weights::SubstrateWeight<Test>;
}

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
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}
