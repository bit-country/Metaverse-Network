use crate::chain_spec::Extensions;
use cumulus_primitives_core::ParaId;
use hex_literal::hex;
use pioneer_runtime::{
	constants::currency::*, AccountId, AuraConfig, BalancesConfig, GenesisConfig, SessionKeys, Signature, SudoConfig,
	SystemConfig, EXISTENTIAL_DEPOSIT, WASM_BINARY,
};
use primitives::Balance;
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
use std::collections::BTreeMap;
use std::str::FromStr;

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<pioneer_runtime::GenesisConfig, Extensions>;

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
pub fn parachain_session_keys(keys: AuraId) -> pioneer_runtime::SessionKeys {
	pioneer_runtime::SessionKeys { aura: keys }
}

//pub fn metaverse_network_inflation_config() -> InflationInfo<Balance> {
//	InflationInfo {
//		expect: Range {
//			min: 100_000 * DOLLARS,
//			ideal: 200_000 * DOLLARS,
//			max: 500_000 * DOLLARS,
//		},
//		annual: Range {
//			min: Perbill::from_percent(4),
//			ideal: Perbill::from_percent(5),
//			max: Perbill::from_percent(5),
//		},
//		// 8766 rounds (hours) in a year
//		round: Range {
//			min: Perbill::from_parts(Perbill::from_percent(4).deconstruct() / 8766),
//			ideal: Perbill::from_parts(Perbill::from_percent(5).deconstruct() / 8766),
//			max: Perbill::from_parts(Perbill::from_percent(5).deconstruct() / 8766),
//		},
//	}
//}

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
		Some(pioneer_properties()),
		Extensions {
			relay_chain: "rococo-local".into(), // You MUST set this to the correct network!
			para_id: id.into(),
		},
	)
}

pub fn pioneer_network_config(id: ParaId) -> ChainSpec {
	// Give your base currency a unit name and decimal places

	ChainSpec::from_genesis(
		// Name
		"Pioneer Network",
		// ID
		"pioneer-live",
		ChainType::Live,
		move || {
			pioneer_genesis(
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
						50 * 1000 * KILODOLLARS,
					),
					(
						hex!["641625840fd1b0e315b178e360fa6a6b200f514bd51d348a0564525c27ec7b25"].into(),
						50 * 1000 * KILODOLLARS,
					),
				],
				id,
			)
		},
		vec![],
		None,
		None,
		Some(pioneer_properties()),
		Extensions {
			relay_chain: "rococo-local".into(), // You MUST set this to the correct network!
			para_id: id.into(),
		},
	)
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
			changes_trie_config: Default::default(),
		},
		balances: pioneer_runtime::BalancesConfig {
			balances: initial_allocation,
		},
		sudo: pioneer_runtime::SudoConfig { key: root_key },
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
						acc.clone(),                 // account id
						acc,                         // validator id
						parachain_session_keys(aura), // session keys
					)
				})
				.collect(),
		},
		aura: Default::default(),
		aura_ext: Default::default(),
		parachain_system: Default::default(),
//		council: Default::default(),
//		tokens: Default::default(),
//		vesting: Default::default(),
//		continuum: ContinuumConfig {
//			initial_active_session: Default::default(),
//			initial_auction_rate: 5,
//			initial_max_bound: (-100, 100),
//			spot_price: 5 * DOLLARS,
//		},
		/* staking: StakingConfig {
		 * 	candidates: staking_candidate,
		 * 	nominations: vec![],
		 * 	inflation_config: metaverse_network_inflation_config(),
		 * },
		 * session: SessionConfig {
		 * 	keys: initial_authorities
		 * 		.iter()
		 * 		.map(|x| (x.0.clone(), x.0.clone(), session_keys(x.1.clone(), x.2.clone())))
		 * 		.collect::<Vec<_>>(),
		 * },
		 *		evm: EVMConfig {
		 *			accounts: {
		 *				let mut map = BTreeMap::new();
		 *				map.insert(
		 *					// H160 address of Alice dev account
		 *					// Derived from SS58 (42 prefix) address
		 *					// SS58: 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
		 *					// hex: 0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d
		 *					// Using the full hex key, truncating to the first 20 bytes (the first 40 hex chars)
		 *					H160::from_str("d43593c715fdd31c61141abd04a99fd6822c8558").expect("internal H160 is valid; qed"),
		 *					pallet_evm::GenesisAccount {
		 *						balance: U256::from_str("0xffffffffffffffffffffffffffffffff")
		 *							.expect("internal U256 is valid; qed"),
		 *						code: Default::default(),
		 *						nonce: Default::default(),
		 *						storage: Default::default(),
		 *					},
		 *				);
		 *				map.insert(
		 *					// H160 address of CI test runner account
		 *					H160::from_str("6be02d1d3665660d22ff9624b7be0551ee1ac91b").expect("internal H160 is valid; qed"),
		 *					pallet_evm::GenesisAccount {
		 *						balance: U256::from_str("0xffffffffffffffffffffffffffffffff")
		 *							.expect("internal U256 is valid; qed"),
		 *						code: Default::default(),
		 *						nonce: Default::default(),
		 *						storage: Default::default(),
		 *					},
		 *				);
		 *				map
		 *			},
		 *		}, */
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
			changes_trie_config: Default::default(),
		},
		balances: pioneer_runtime::BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|k| (k, 1 << 60)).collect(),
		},
		sudo: pioneer_runtime::SudoConfig { key: root_key },
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
						acc.clone(),                 // account id
						acc,                         // validator id
						parachain_session_keys(aura), // session keys
					)
				})
				.collect(),
		},
		aura: Default::default(),
		aura_ext: Default::default(),
		parachain_system: Default::default(),
//		council: Default::default(),
//		tokens: Default::default(),
//		vesting: Default::default(),
//		continuum: ContinuumConfig {
//			initial_active_session: Default::default(),
//			initial_auction_rate: 5,
//			initial_max_bound: (-100, 100),
//			spot_price: 5 * DOLLARS,
//		},
		/* staking: StakingConfig {
		 * 	candidates: staking_candidate,
		 * 	nominations: vec![],
		 * 	inflation_config: metaverse_network_inflation_config(),
		 * },
		 * session: SessionConfig {
		 * 	keys: initial_authorities
		 * 		.iter()
		 * 		.map(|x| (x.0.clone(), x.0.clone(), session_keys(x.1.clone(), x.2.clone())))
		 * 		.collect::<Vec<_>>(),
		 * },
		 *		evm: EVMConfig {
		 *			accounts: {
		 *				let mut map = BTreeMap::new();
		 *				map.insert(
		 *					// H160 address of Alice dev account
		 *					// Derived from SS58 (42 prefix) address
		 *					// SS58: 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
		 *					// hex: 0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d
		 *					// Using the full hex key, truncating to the first 20 bytes (the first 40 hex chars)
		 *					H160::from_str("d43593c715fdd31c61141abd04a99fd6822c8558").expect("internal H160 is valid; qed"),
		 *					pallet_evm::GenesisAccount {
		 *						balance: U256::from_str("0xffffffffffffffffffffffffffffffff")
		 *							.expect("internal U256 is valid; qed"),
		 *						code: Default::default(),
		 *						nonce: Default::default(),
		 *						storage: Default::default(),
		 *					},
		 *				);
		 *				map.insert(
		 *					// H160 address of CI test runner account
		 *					H160::from_str("6be02d1d3665660d22ff9624b7be0551ee1ac91b").expect("internal H160 is valid; qed"),
		 *					pallet_evm::GenesisAccount {
		 *						balance: U256::from_str("0xffffffffffffffffffffffffffffffff")
		 *							.expect("internal U256 is valid; qed"),
		 *						code: Default::default(),
		 *						nonce: Default::default(),
		 *						storage: Default::default(),
		 *					},
		 *				);
		 *				map
		 *			},
		 *		}, */
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

	properties.insert("ss58Format".into(), 42.into());
	properties.insert("tokenDecimals".into(), 18.into());
	properties.insert("tokenSymbol".into(), "NEER".into());

	properties
}
