#![cfg(test)]

use frame_support::{assert_err, assert_noop, assert_ok};
use sp_runtime::traits::BadOrigin;

use mock::{RuntimeCall, RuntimeEvent, RuntimeOrigin, *};

use super::*;

#[test]
fn start_migration_should_not_work_from_non_admin_account() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(NftMigration::start_migration(RuntimeOrigin::signed(BOB)), BadOrigin);
	});
}

#[test]
fn start_migration_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(NftMigration::is_migration_active(), false);
		assert_ok!(NftMigration::start_migration(RuntimeOrigin::signed(ALICE)));
		assert_eq!(last_event(), RuntimeEvent::NftMigration(crate::Event::MigrationStarted));
		assert_eq!(NftMigration::is_migration_active(), true);
	});
}
