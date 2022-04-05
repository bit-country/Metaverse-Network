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
	constants::currency::*, AccountId, AuraConfig, BalancesConfig, GenesisConfig, SessionKeys, Signature, SudoConfig,
	SystemConfig, EXISTENTIAL_DEPOSIT, WASM_BINARY,
};
use primitives::Balance;

use crate::chain_spec::Extensions;

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<pioneer_runtime::GenesisConfig, Extensions>;

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

pub fn development_config(id: ParaId) -> ChainSpec {
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
				id,
			)
		},
		vec![],
		None,
		None,
		None,
		Some(pioneer_properties()),
		Extensions {
			relay_chain: "rococo-local".into(), // You MUST set this to the correct network!
			para_id: id.into(),
		},
	)
}

pub fn local_testnet_config(id: ParaId) -> ChainSpec {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "UNIT".into());
	properties.insert("tokenDecimals".into(), 12.into());

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
				id,
			)
		},
		Vec::new(),
		None,
		None,
		None,
		Some(pioneer_properties()),
		Extensions {
			relay_chain: "rococo-local".into(), // You MUST set this to the correct network!
			para_id: id.into(),
		},
	)
}

pub fn pioneer_network_config_json() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../../../node/res/pioneer-live-raw-spec.json")[..])
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
		parachain_info: pioneer_runtime::ParachainInfoConfig { parachain_id: id },
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
		//		continuum: ContinuumConfig {
		//			initial_active_session: Default::default(),
		//			initial_auction_rate: 5,
		//			initial_max_bound: (-100, 100),
		//			spot_price: 5 * DOLLARS,
		//		},
		//		estate: EstateConfig {
		//			minting_rate_config: metaverse_land_minting_config(),
		//		},
	}
}

fn testnet_genesis(
	root_key: AccountId,
	initial_authorities: Vec<(AccountId, AuraId)>,
	endowed_accounts: Vec<AccountId>,
	id: ParaId,
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
		parachain_info: pioneer_runtime::ParachainInfoConfig { parachain_id: id },
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
		//		continuum: ContinuumConfig {
		//			initial_active_session: Default::default(),
		//			initial_auction_rate: 5,
		//			initial_max_bound: (-100, 100),
		//			spot_price: 5 * DOLLARS,
		//		},
		//		estate: EstateConfig {
		//			minting_rate_config: metaverse_land_minting_config(),
		//		},
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
