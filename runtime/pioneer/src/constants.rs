/// Money matters.
pub mod currency {
	use primitives::Balance;

	pub const KILODOLLARS: Balance = 1_000_000_000_000_000_000_000;
	pub const DOLLARS: Balance = 1_000_000_000_000_000_000;
	pub const CENTS: Balance = DOLLARS / 100;
	/// 10_000_000_000_000_000
	pub const MILLICENTS: Balance = CENTS / 1000;
	/// 10_000_000_000_000
	pub const MICROCENTS: Balance = MILLICENTS / 1000;
	/// 10_000_000_000

	pub const fn deposit(items: u32, bytes: u32) -> Balance {
		items as Balance * 15 * CENTS + (bytes as Balance) * 6 * CENTS
	}
}

/// Time.
pub mod time {
	use frame_support::dispatch::Weight;
	use frame_support::weights::constants::{ExtrinsicBaseWeight, WEIGHT_PER_SECOND};

	use primitives::{BlockNumber, Moment};

	use crate::{Balance, Perbill, CENTS};

	/// This determines the average expected block time that we are targeting.
	/// Blocks will be produced at a minimum duration defined by `SLOT_DURATION`.
	/// `SLOT_DURATION` is picked up by `pallet_timestamp` which is in turn picked
	/// up by `pallet_aura` to implement `fn slot_duration()`.
	///
	/// Change this to adjust the block time.
	pub const MILLISECS_PER_BLOCK: u64 = 12000;

	// NOTE: Currently it is not possible to change the slot duration after the chain has started.
	//       Attempting to do so will brick block production.
	pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

	// Time is measured by number of blocks.
	pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
	pub const HOURS: BlockNumber = MINUTES * 60;
	pub const DAYS: BlockNumber = HOURS * 24;

	// Unit = the base number of indivisible units for balances
	pub const UNIT: Balance = 1_000_000_000_000;
	pub const MILLIUNIT: Balance = 1_000_000_000;
	pub const MICROUNIT: Balance = 1_000_000;

	/// The existential deposit. Set to 1/10 of the Rococo Relay Chain.
	pub const EXISTENTIAL_DEPOSIT: Balance = MILLIUNIT;

	// 1 in 4 blocks (on average, not counting collisions) will be primary babe blocks.
	pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);

	/// We assume that ~5% of the block weight is consumed by `on_initialize` handlers. This is
	/// used to limit the maximal weight of a single extrinsic.
	pub const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(5);

	/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used by
	/// `Operational` extrinsics.
	pub const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

	/// We allow for 0.5 of a second of compute with a 12 second average block time.
	pub const MAXIMUM_BLOCK_WEIGHT: Weight = WEIGHT_PER_SECOND / 2;

	fn base_tx_fee() -> Balance {
		CENTS / 10
	}

	pub fn ksm_per_second() -> u128 {
		let base_weight = Balance::from(ExtrinsicBaseWeight::get());
		let base_tx_per_second = (WEIGHT_PER_SECOND as u128) / base_weight;
		let fee_per_second = base_tx_per_second * base_tx_fee();
		fee_per_second / 100
	}
}

#[allow(non_snake_case)]
pub mod parachains {
	pub mod karura {
		pub const ID: u32 = 2000;
		pub const KAR_KEY: &[u8] = &[0, 128];
		pub const KUSD_KEY: &[u8] = &[0, 129];
	}
}
