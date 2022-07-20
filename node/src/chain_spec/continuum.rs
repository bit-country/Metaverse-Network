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

use continuum_runtime::{
	constants::currency::*, AccountId, AuraConfig, BalancesConfig, EstateConfig, GenesisConfig, OracleMembershipConfig,
	SessionKeys, Signature, SudoConfig, SystemConfig, EXISTENTIAL_DEPOSIT, WASM_BINARY,
};
use metaverse_runtime::MintingRateInfo;
use primitives::Balance;

use crate::chain_spec::Extensions;

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<continuum_runtime::GenesisConfig, Extensions>;

/// The default XCM version to set in genesis config.
const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;

pub const PARA_ID: u32 = 2050;

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
pub fn parachain_session_keys(keys: AuraId) -> continuum_runtime::SessionKeys {
	continuum_runtime::SessionKeys { aura: keys }
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
		"continuum-dev",
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
				PARA_ID.into(),
			)
		},
		vec![],
		None,
		None,
		None,
		Some(continuum_properties()),
		Extensions {
			relay_chain: "rococo-local".into(), // You MUST set this to the correct network!
			para_id: PARA_ID,
		},
	)
}

pub fn local_testnet_config() -> ChainSpec {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "NUUM".into());
	properties.insert("tokenDecimals".into(), 18.into());

	ChainSpec::from_genesis(
		// Name
		"Local Testnet",
		// ID
		"continuum_local_testnet",
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
				PARA_ID.into(),
			)
		},
		Vec::new(),
		None,
		None,
		None,
		Some(continuum_properties()),
		Extensions {
			relay_chain: "rococo-local".into(), // You MUST set this to the correct network!
			para_id: PARA_ID,
		},
	)
}

pub fn pioneer_network_config_json() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../../../node/res/pioneer-live-raw-spec.json")[..])
}

pub fn continuum_genesis_config() -> ChainSpec {
	ChainSpec::from_genesis(
		// Name
		"Pioneer Network",
		// ID
		"pioneer-live",
		ChainType::Live,
		move || {
			continuum_genesis(
				hex!["886286c58d67217bdd854832d5e9f1b218dec6a0ff7e0b7573147ca94a233a0a"].into(),
				vec![
					(
						// 5Fh7g4pSqUYWKENqhjgWvMv44KFYbk2cdktfaCuSFuACvUFz
						hex!["a079b5f55ba3f64990f8af5edf1fc57712f3ee97f51a74a8143c360a2739ff02"].into(),
						hex!["a079b5f55ba3f64990f8af5edf1fc57712f3ee97f51a74a8143c360a2739ff02"].unchecked_into(),
					),
					(
						// 5CPYv4r2kMwC8cac7psqex6Ajh2xw22MrCJPm7kdjfvEbbt1
						hex!["0e5f902f5273b54271f9e57b35d7872e42b60fa7d770870c313473db5903597b"].into(),
						hex!["0e5f902f5273b54271f9e57b35d7872e42b60fa7d770870c313473db5903597b"].unchecked_into(),
					),
				],
				vec![
					(
						hex!["886286c58d67217bdd854832d5e9f1b218dec6a0ff7e0b7573147ca94a233a0a"].into(),
						500 * 1000 * KILODOLLARS,
					),
					(
						hex!["641625840fd1b0e315b178e360fa6a6b200f514bd51d348a0564525c27ec7b25"].into(),
						500 * 1000 * KILODOLLARS,
					),
				],
				PARA_ID.into(),
			)
		},
		vec![],
		None,
		None,
		None,
		Some(continuum_properties()),
		Extensions {
			relay_chain: "polkadot".into(),
			para_id: PARA_ID.into(),
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

fn continuum_genesis(
	root_key: AccountId,
	initial_authorities: Vec<(AccountId, AuraId)>,
	initial_allocation: Vec<(AccountId, Balance)>,
	id: ParaId,
) -> continuum_runtime::GenesisConfig {
	continuum_runtime::GenesisConfig {
		system: continuum_runtime::SystemConfig {
			code: continuum_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
		},
		balances: continuum_runtime::BalancesConfig {
			balances: initial_allocation,
		},
		sudo: continuum_runtime::SudoConfig {
			key: Some(root_key.clone()),
		},
		parachain_info: continuum_runtime::ParachainInfoConfig {
			parachain_id: PARA_ID.into(),
		},
		collator_selection: continuum_runtime::CollatorSelectionConfig {
			invulnerables: initial_authorities.iter().cloned().map(|(acc, _)| acc).collect(),
			candidacy_bond: EXISTENTIAL_DEPOSIT * 16,
			..Default::default()
		},
		session: continuum_runtime::SessionConfig {
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

fn testnet_genesis(
	root_key: AccountId,
	initial_authorities: Vec<(AccountId, AuraId)>,
	endowed_accounts: Vec<AccountId>,
	id: ParaId,
) -> continuum_runtime::GenesisConfig {
	continuum_runtime::GenesisConfig {
		system: continuum_runtime::SystemConfig {
			code: continuum_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
		},
		balances: continuum_runtime::BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, 250 * KILODOLLARS))
				.collect(),
		},
		sudo: continuum_runtime::SudoConfig {
			key: Some(root_key.clone()),
		},
		parachain_info: continuum_runtime::ParachainInfoConfig { parachain_id: id },
		collator_selection: continuum_runtime::CollatorSelectionConfig {
			invulnerables: initial_authorities.iter().cloned().map(|(acc, _)| acc).collect(),
			candidacy_bond: EXISTENTIAL_DEPOSIT * 16,
			..Default::default()
		},
		session: continuum_runtime::SessionConfig {
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

pub fn continuum_properties() -> Properties {
	let mut properties = Properties::new();

	properties.insert("ss58Format".into(), 268.into());
	properties.insert("tokenDecimals".into(), 18.into());
	properties.insert("tokenSymbol".into(), "NUUM".into());

	properties
}
