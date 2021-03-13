
// #![cfg(test)]

// use super::*;
// use crate::{Module, Trait};
// use sp_core::H256;
// use frame_support::{impl_outer_event, impl_outer_origin, parameter_types};
// use sp_runtime::{
// 	traits::{BlakeTwo256, IdentityLookup}, testing::Header, Perbill,
// };

// pub type AccountId = u128;
// pub type BlockNumber = u64;

// pub const ALICE: AccountId = 1;
// pub const BOB: AccountId = 2;

// use super::*;
// use frame_system as system;


// #[derive(Clone, Eq, PartialEq)]
// pub struct Runtime;

// mod block {
// 	pub use super::super::*;
// }

// impl_outer_origin! {
// 	pub enum Origin for Runtime {}
// }

// impl_outer_event! {
// 	pub enum TestEvent for Runtime {
// 		frame_system<T>,
// 		block,
// 	}
// }


// // Configure a mock runtime to test the pallet.

// parameter_types! {
// 	pub const BlockHashCount: u64 = 250;
// 	pub const MaximumBlockWeight: u32 = 1024;
// 	pub const MaximumBlockLength: u32 = 2 * 1024;
// 	pub const AvailableBlockRatio: Perbill = Perbill::one();
// }


// impl frame_system::Trait for Runtime {
// 	type Origin = Origin;
// 	type Index = u64;
// 	type BlockNumber = BlockNumber;
// 	type Call = ();
// 	type Hash = H256;
// 	type Hashing = ::sp_runtime::traits::BlakeTwo256;
// 	type AccountId = AccountId;
// 	type Lookup = IdentityLookup<Self::AccountId>;
// 	type Header = Header;
// 	type Event = TestEvent;
// 	type BlockHashCount = BlockHashCount;
// 	type MaximumBlockWeight = MaximumBlockWeight;
// 	type MaximumBlockLength = MaximumBlockLength;
// 	type AvailableBlockRatio = AvailableBlockRatio;
// 	type Version = ();
// 	type PalletInfo = ();
// 	type AccountData = ();
// 	type OnNewAccount = ();
// 	type OnKilledAccount = ();
// 	type DbWeight = ();
// 	type BlockExecutionWeight = ();
// 	type ExtrinsicBaseWeight = ();
// 	type MaximumExtrinsicWeight = ();
// 	type BaseCallFilter = ();
// 	type SystemWeightInfo = ();
// }

// pub type System = frame_system::Module<Runtime>;

// impl Trait for Runtime {
// 	type Event = TestEvent;
// }

// pub type BlockModule = Module<Runtime>;


// pub struct ExtBuilder();

// impl Default for ExtBuilder {
// 	fn default() -> Self {
// 		ExtBuilder
// 	}
// }

// impl ExtBuilder {
// 	pub fn build(self) -> sp_io::TestExternalities {
// 		let t = frame_system::GenesisConfig::default()
// 			.build_storage::<Runtime>()
// 			.unwrap();

// 		let mut ext = sp_io::TestExternalities::new(t);
// 		ext.execute_with(|| System::set_block_number(1));
// 		ext
// 	}
// }