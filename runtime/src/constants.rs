pub mod currency {
	use node_primitives::Balance;

	pub const BCG: Balance = 1_000_000_000_000;
	pub const DOLLARS: Balance = BCG;             // 1_000_000_000_000
	pub const CENTS: Balance = DOLLARS / 100;      // 10_000_000_000
	pub const MILLICENTS: Balance = CENTS / 1_000; // 10_000_000

	pub const fn deposit(items: u32, bytes: u32) -> Balance {
		items as Balance * 15 * CENTS + (bytes as Balance) * 6 * CENTS
	}
}

pub mod time {
	use node_primitives::{Moment, BlockNumber};

	pub const MILLISECS_PER_BLOCK: Moment = 6000;
	pub const SLOT_DURATION: Moment = MILLISECS_PER_BLOCK;

	// These time units are defined in number of blocks.
	pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
	pub const HOURS: BlockNumber = MINUTES * 60;
	pub const DAYS: BlockNumber = HOURS * 24;
}