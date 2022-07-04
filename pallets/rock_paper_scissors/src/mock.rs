use crate as pallet_rock_paper_scissors;


use frame_support::traits::{ConstU16, ConstU64};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		RPS: pallet_rock_paper_scissors::{Pallet, Call, Storage, Event<T>},
	}
);

impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
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
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

frame_support::parameter_types! {
	pub const MinBetAmount: u64 = 100;
	pub static ExistentialDeposit: u64 = 1;
}

impl pallet_rock_paper_scissors::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type MinBetAmount = MinBetAmount;
}

impl pallet_balances::Config for Test {
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type Balance = u64;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext(
	endowed_accounts: &[u64],
	endowment_amount: u64,
) -> sp_io::TestExternalities {
	let mut ext: sp_io::TestExternalities = GenesisConfig {
		system: Default::default(),
		balances: BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|k| (k, endowment_amount)).collect(),
		},
	}
	.build_storage()
	.unwrap()
	.into();

	ext.execute_with(|| {
		System::set_block_number(1);
	});

	ext
}

pub fn last_event() -> Event {
	frame_system::Pallet::<Test>::events().pop().expect("Event expected").event
}
