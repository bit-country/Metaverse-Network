use cumulus_primitives_core::ParaId;
use pioneer_runtime::{
	constants::currency::*,
	// SessionConfig,  StakingConfig,
	opaque::SessionKeys,
	AccountId,
	AuraConfig,
	BalancesConfig,
	ContinuumConfig,
	EVMConfig,
	GenesisConfig,
	GrandpaConfig,
	InflationInfo,
	Range,
	Signature,
	SudoConfig,
	SystemConfig,
	WASM_BINARY,
};
use primitives::Balance;
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{sr25519, Pair, Public, H160, U256};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::{
	traits::{IdentifyAccount, Verify},
	Perbill,
};
use std::collections::BTreeMap;
use std::str::FromStr;

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<pioneer_runtime::GenesisConfig, Extensions>;

fn session_keys(aura: AuraId, grandpa: GrandpaId) -> SessionKeys {
	SessionKeys { aura, grandpa }
}

pub fn metaverse_network_inflation_config() -> InflationInfo<Balance> {
	InflationInfo {
		expect: Range {
			min: 100_000 * DOLLARS,
			ideal: 200_000 * DOLLARS,
			max: 500_000 * DOLLARS,
		},
		annual: Range {
			min: Perbill::from_percent(4),
			ideal: Perbill::from_percent(5),
			max: Perbill::from_percent(5),
		},
		// 8766 rounds (hours) in a year
		round: Range {
			min: Perbill::from_parts(Perbill::from_percent(4).deconstruct() / 8766),
			ideal: Perbill::from_parts(Perbill::from_percent(5).deconstruct() / 8766),
			max: Perbill::from_parts(Perbill::from_percent(5).deconstruct() / 8766),
		},
	}
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
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "UNIT".into());
	properties.insert("tokenDecimals".into(), 12.into());

	ChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
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
		Extensions {
			relay_chain: "rococo-local".into(), // You MUST set this to the correct network!
			para_id: id.into(),
		},
	)
}

fn testnet_genesis(
	root_key: AccountId,
	initial_authorities: Vec<(AccountId, AuraId, GrandpaId)>,
	endowed_accounts: Vec<AccountId>,
	id: ParaId,
) -> pioneer_runtime::GenesisConfig {
	// let staking_candidate: Vec<(AccountId, Balance)> = initial_authorities
	// 	.iter()
	// 	.clone()
	// 	.map(|x| (x.0.clone(), 100 * DOLLARS))
	// 	.collect();

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
		aura: pioneer_runtime::AuraConfig {
			authorities: initial_authorities.iter().clone().map(|x| (x.1.clone())).collect(),
		},
		aura_ext: Default::default(),
		parachain_system: Default::default(),
		grandpa: GrandpaConfig {
			//			authorities: initial_authorities.iter().map(|x| (x.2.clone(), 1)).collect(),
			authorities: vec![],
		},
		council: Default::default(),
		tokens: Default::default(),
		vesting: Default::default(),
		continuum: ContinuumConfig {
			initial_active_session: Default::default(),
			initial_auction_rate: 5,
			initial_max_bound: (-100, 100),
			spot_price: 5 * DOLLARS,
		},
		// staking: StakingConfig {
		// 	candidates: staking_candidate,
		// 	nominations: vec![],
		// 	inflation_config: metaverse_network_inflation_config(),
		// },
		// session: SessionConfig {
		// 	keys: initial_authorities
		// 		.iter()
		// 		.map(|x| (x.0.clone(), x.0.clone(), session_keys(x.1.clone(), x.2.clone())))
		// 		.collect::<Vec<_>>(),
		// },
		evm: EVMConfig {
			accounts: {
				let mut map = BTreeMap::new();
				map.insert(
					// H160 address of Alice dev account
					// Derived from SS58 (42 prefix) address
					// SS58: 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
					// hex: 0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d
					// Using the full hex key, truncating to the first 20 bytes (the first 40 hex chars)
					H160::from_str("d43593c715fdd31c61141abd04a99fd6822c8558").expect("internal H160 is valid; qed"),
					pallet_evm::GenesisAccount {
						balance: U256::from_str("0xffffffffffffffffffffffffffffffff")
							.expect("internal U256 is valid; qed"),
						code: Default::default(),
						nonce: Default::default(),
						storage: Default::default(),
					},
				);
				map.insert(
					// H160 address of CI test runner account
					H160::from_str("6be02d1d3665660d22ff9624b7be0551ee1ac91b").expect("internal H160 is valid; qed"),
					pallet_evm::GenesisAccount {
						balance: U256::from_str("0xffffffffffffffffffffffffffffffff")
							.expect("internal U256 is valid; qed"),
						code: Default::default(),
						nonce: Default::default(),
						storage: Default::default(),
					},
				);
				map
			},
		},
	}
}

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AccountId, AuraId, GrandpaId) {
	(
		get_account_id_from_seed::<sr25519::Public>(s),
		get_from_seed::<AuraId>(s),
		get_from_seed::<GrandpaId>(s),
	)
}
