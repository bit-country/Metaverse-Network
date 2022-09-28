use std::collections::BTreeMap;
use std::str::FromStr;

use cumulus_primitives_core::ParaId;
use hex_literal::hex;
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::{ChainType, Properties};
use serde::{Deserialize, Serialize};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public, H160, U256};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::{
	traits::{IdentifyAccount, Verify},
	Perbill,
};

use metaverse_runtime::MintingRateInfo;
use pioneer_runtime::{
	constants::currency::*, AccountId, AuraConfig, BalancesConfig, EstateConfig, GenesisConfig, OracleMembershipConfig,
	SessionKeys, Signature, SudoConfig, SystemConfig, EXISTENTIAL_DEPOSIT, WASM_BINARY,
};
use primitives::Balance;

use crate::chain_spec::Extensions;

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<pioneer_runtime::GenesisConfig, Extensions>;

/// The default XCM version to set in genesis config.
const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;

pub const PARA_ID: u32 = 2096;
pub const ROC_PARA_ID: u32 = 2096;

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
pub fn parachain_session_keys(keys: AuraId) -> pioneer_runtime::SessionKeys {
	pioneer_runtime::SessionKeys { aura: keys }
}

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

pub fn development_config() -> ChainSpec {
	ChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"pioneer-dev",
		ChainType::Local,
		move || {
			testnet_genesis(
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				vec![authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")],
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
				],
			)
		},
		vec![],
		None,
		None,
		None,
		Some(pioneer_properties()),
		Extensions {
			relay_chain: "rococo-local".into(), // You MUST set this to the correct network!
			para_id: PARA_ID,
		},
	)
}

pub fn local_testnet_config() -> ChainSpec {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "NEER".into());
	properties.insert("tokenDecimals".into(), 18.into());

	ChainSpec::from_genesis(
		// Name
		"Local Testnet",
		// ID
		"pioneer_local_testnet",
		ChainType::Local,
		move || {
			testnet_genesis(
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				vec![authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")],
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Charlie"),
					get_account_id_from_seed::<sr25519::Public>("Dave"),
					get_account_id_from_seed::<sr25519::Public>("Eve"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
					get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
					get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
					get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
				],
			)
		},
		Vec::new(),
		None,
		None,
		None,
		Some(pioneer_properties()),
		Extensions {
			relay_chain: "rococo-local".into(), // You MUST set this to the correct network!
			para_id: PARA_ID,
		},
	)
}

pub fn pioneer_network_config_json() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../../../node/res/pioneer-live-raw-spec.json")[..])
}

pub fn roc_pioneer_testnet_config() -> ChainSpec {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "NEER".into());
	properties.insert("tokenDecimals".into(), 18.into());

	ChainSpec::from_genesis(
		// Name
		"Pioneer Testnet",
		// ID
		"pioneer_roc_testnet",
		ChainType::Live,
		move || {
			testnet_genesis(
				hex![
					// 5Dqy8KtwmGJd6Tkar8Va3Uw7xvX4RQAhrygUk3T8vUxDXf2a
					"4ec1ae0facb941380f72f314a5ef6c3ee012a3e105e34806537e3f3c4a3ff167"
				]
				.into(),
				vec![
					(
						// 5FpqLqqbFyYWgYtgQS11HvTkaripk1nPFFti6CwDaMj8cSvu
						hex!["a65cb28d2524996ee0e02aa1ebfa5c1b4ff3db7edad9b11f7033960cc5aa3c3e"].into(),
						hex!["a65cb28d2524996ee0e02aa1ebfa5c1b4ff3db7edad9b11f7033960cc5aa3c3e"].unchecked_into(),
					),
					(
						// 5EUXjqNx3Rsh3wtDJAPBzEvJdGVD3QmxmMUjrfARNr4uh7pq
						hex!["6aa44c06b0a479f95757137a1b08fd00971823430147094dc66e7aa2b381f146"].into(),
						hex!["6aa44c06b0a479f95757137a1b08fd00971823430147094dc66e7aa2b381f146"].unchecked_into(),
					),
				],
				vec![hex![
					// 5Dqy8KtwmGJd6Tkar8Va3Uw7xvX4RQAhrygUk3T8vUxDXf2a
					"4ec1ae0facb941380f72f314a5ef6c3ee012a3e105e34806537e3f3c4a3ff167"
				]
				.into()],
			)
		},
		Vec::new(),
		None,
		None,
		None,
		Some(pioneer_properties()),
		Extensions {
			relay_chain: "rococo".into(),
			para_id: ROC_PARA_ID,
		},
	)
}

pub fn metaverse_land_minting_config() -> MintingRateInfo {
	MintingRateInfo {
		expect: Default::default(),
		// 10% minting rate per annual
		annual: 10,
		// Max 100 millions land unit
		max: 100_000_000,
	}
}

fn pioneer_genesis(
	root_key: AccountId,
	initial_authorities: Vec<(AccountId, AuraId)>,
	initial_allocation: Vec<(AccountId, Balance)>,
	id: ParaId,
) -> pioneer_runtime::GenesisConfig {
	pioneer_runtime::GenesisConfig {
		system: pioneer_runtime::SystemConfig {
			code: pioneer_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
		},
		balances: pioneer_runtime::BalancesConfig {
			balances: initial_allocation,
		},
		sudo: pioneer_runtime::SudoConfig {
			key: Some(root_key.clone()),
		},
		parachain_info: pioneer_runtime::ParachainInfoConfig {
			parachain_id: PARA_ID.into(),
		},
		collator_selection: pioneer_runtime::CollatorSelectionConfig {
			invulnerables: initial_authorities.iter().cloned().map(|(acc, _)| acc).collect(),
			candidacy_bond: EXISTENTIAL_DEPOSIT * 16,
			..Default::default()
		},
		session: pioneer_runtime::SessionConfig {
			keys: initial_authorities
				.iter()
				.cloned()
				.map(|(acc, aura)| {
					(
						acc.clone(),                  // account id
						acc,                          // validator id
						parachain_session_keys(aura), // session keys
					)
				})
				.collect(),
		},
		aura: Default::default(),
		aura_ext: Default::default(),
		parachain_system: Default::default(),
		estate: EstateConfig {
			minting_rate_config: metaverse_land_minting_config(),
		},
		oracle_membership: OracleMembershipConfig {
			members: vec![],
			phantom: Default::default(),
		},
		treasury: Default::default(),
	}
}

fn testnet_genesis(
	root_key: AccountId,
	initial_authorities: Vec<(AccountId, AuraId)>,
	endowed_accounts: Vec<AccountId>,
) -> pioneer_runtime::GenesisConfig {
	pioneer_runtime::GenesisConfig {
		system: pioneer_runtime::SystemConfig {
			code: pioneer_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
		},
		balances: pioneer_runtime::BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, 250 * KILODOLLARS))
				.collect(),
		},
		sudo: pioneer_runtime::SudoConfig {
			key: Some(root_key.clone()),
		},
		parachain_info: pioneer_runtime::ParachainInfoConfig {
			parachain_id: ROC_PARA_ID.into(),
		},
		collator_selection: pioneer_runtime::CollatorSelectionConfig {
			invulnerables: initial_authorities.iter().cloned().map(|(acc, _)| acc).collect(),
			candidacy_bond: EXISTENTIAL_DEPOSIT * 16,
			..Default::default()
		},
		session: pioneer_runtime::SessionConfig {
			keys: initial_authorities
				.iter()
				.cloned()
				.map(|(acc, aura)| {
					(
						acc.clone(),                  // account id
						acc,                          // validator id
						parachain_session_keys(aura), // session keys
					)
				})
				.collect(),
		},
		aura: Default::default(),
		aura_ext: Default::default(),
		parachain_system: Default::default(),
		estate: EstateConfig {
			minting_rate_config: metaverse_land_minting_config(),
		},
		oracle_membership: OracleMembershipConfig {
			members: vec![],
			phantom: Default::default(),
		},
	}
}

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AccountId, AuraId) {
	(
		get_account_id_from_seed::<sr25519::Public>(s),
		get_from_seed::<AuraId>(s),
	)
}

pub fn pioneer_properties() -> Properties {
	let mut properties = Properties::new();

	properties.insert("ss58Format".into(), 268.into());
	properties.insert("tokenDecimals".into(), 18.into());
	properties.insert("tokenSymbol".into(), "NEER".into());

	properties
}
