use cumulus_primitives::ParaId;
use hex_literal::hex;
use primitives::{AccountId, Signature};
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::{ChainType, Properties};
use serde::{Deserialize, Serialize};
use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};
use bitcountry_runtime::{WASM_BINARY,BalancesConfig,SystemConfig, SudoConfig};
// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<bitcountry_runtime::GenesisConfig, Extensions>;

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
	/// The relay chain of the Parachain.
	pub relay_chain: String,
	/// The id of the Parachain.
	pub para_id: u32,
}

impl Extensions {
	/// Try to get the extension from the given `ChainSpec`.
	pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
		sc_chain_spec::get_extension(chain_spec.extensions())
	}
}

type AccountPublic = <Signature as Verify>::Signer;

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

// /// Generate an Aura authority key.
// pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
// 	(get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
// }

pub fn get_chain_spec(id: ParaId) -> ChainSpec {
	ChainSpec::from_genesis(
		"Local Testnet",
		"local_testnet",
		ChainType::Local,
		move || {
			testnet_genesis(
				get_account_id_from_seed::<sr25519::Public>("Alice"),
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
		vec![],
		None,
		None,
		None,
		Extensions {
			relay_chain: "westend-dev".into(),
			para_id: id.into(),
		},
	)
}

pub fn development_config(id: ParaId) -> ChainSpec {
	ChainSpec::from_genesis(
		// Name
		"Bit.Country Dev Chain",
		// ID
		"bit-country-dev",
		ChainType::Development,
		move || {
			testnet_genesis(
				get_account_id_from_seed::<sr25519::Public>("Alice"),
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
		// Properties
		Some(bitcountry_properties()),
		Extensions {
			relay_chain: "westend-dev".into(),
			para_id: id.into(),
		},
	)
}

// pub fn development_config(id: ParaId) -> Result<ChainSpec, String> {
// 	let wasm_binary = WASM_BINARY.ok_or("Development wasm binary not available".to_string())?;

// 	Ok(ChainSpec::from_genesis(
// 		// Name
// 		"Bit.Country Dev Chain",
// 		// ID
// 		"dev",
// 		ChainType::Development,
// 		move || {
// 			testnet_genesis(
// 				get_account_id_from_seed::<sr25519::Public>("Alice"),
// 				vec![
// 					get_account_id_from_seed::<sr25519::Public>("Alice"),
// 					get_account_id_from_seed::<sr25519::Public>("Bob"),
// 					get_account_id_from_seed::<sr25519::Public>("Charlie"),
// 					get_account_id_from_seed::<sr25519::Public>("Dave"),
// 					get_account_id_from_seed::<sr25519::Public>("Eve"),
// 					get_account_id_from_seed::<sr25519::Public>("Ferdie"),
// 					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
// 					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
// 					get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
// 					get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
// 					get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
// 					get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
// 				],
// 				id,
// 			)
// 		},
// 		// Bootnodes
// 		vec![],
// 		// Telemetry
// 		None,
// 		// Protocol ID
// 		None,
// 		// Properties
// 		Some(bitcountry_properties()),
// 		// Extensions
// 		Extensions {
// 			relay_chain: "westend-dev".into(),
// 			para_id: id.into(),
// 		},
// 	))
// }

pub fn local_testnet_config(id: ParaId) -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or("Development wasm binary not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Bit.Country Test Chain",
		// ID
		"local_testnet",
		ChainType::Local,
		move || {
			testnet_genesis(
				get_account_id_from_seed::<sr25519::Public>("Alice"),
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
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Properties
		Some(bitcountry_properties()),
		// Extensions
		Extensions {
			relay_chain: "westend-dev".into(),
			para_id: id.into(),
		},
	))
}

pub fn tewai_testnet_config(id: ParaId) -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or("Development wasm binary not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Bit.Country Tewai Chain",
		// ID
		"tewai_testnet",
		ChainType::Live,
		move || {
			testnet_genesis(
				// Sudo account
				hex!["788dddd00a4c5ce1575fe9c11dee9a781455b516ee424885d4b50c2f49862a35"].into(),
				// Pre-funded accounts
				vec![
						/* Sudo Account */
						hex!["788dddd00a4c5ce1575fe9c11dee9a781455b516ee424885d4b50c2f49862a35"].into(),
				],
				id,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Properties
		Some(bitcountry_properties()),
		// Extensions
		Extensions {
			relay_chain: "westend-dev".into(),
			para_id: id.into(),
		},
	))
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	id: ParaId,
) -> bitcountry_runtime::GenesisConfig {
	bitcountry_runtime::GenesisConfig {
		frame_system: Some(bitcountry_runtime::SystemConfig {
			code: bitcountry_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
			changes_trie_config: Default::default(),
		}),
		pallet_balances: Some(BalancesConfig {
			// Configure endowed accounts with initial balance of 1 billion token to test.
			// balances: endowed_accounts.iter().cloned().map(|k|(k)).collect(),
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, 10u128.pow(18 + 9)))
				.collect(),
		}),
		// pallet_aura: Some(AuraConfig {
		// 	authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
		// }),
		// pallet_grandpa: Some(GrandpaConfig {
		// 	authorities: initial_authorities
		// 		.iter()
		// 		.map(|x| (x.1.clone(), 1))
		// 		.collect(),
		// }),
		// pallet_collective_Instance1: Some(Default::default()),
		// pallet_treasury: Some(Default::default()),
		pallet_sudo: Some(SudoConfig {
			// Assign network admin rights.
			key: root_key,
		}),
		// pallet_contracts: Some(ContractsConfig {
		// 	current_schedule: pallet_contracts::Schedule {
		// 		enable_println,
		// 		..Default::default()
		// 	},
		// }),
		parachain_info: Some(bitcountry_runtime::ParachainInfoConfig { parachain_id: id }),
		// tokenization: Some(TokenConfig {
		// 	init_token_id: 0
		// })
	}
}

pub fn bitcountry_properties() -> Properties {
	let mut properties = Properties::new();

	properties.insert("ss58Format".into(), 28.into());
	properties.insert("tokenDecimals".into(), 18.into());
	properties.insert("tokenSymbol".into(), "BCG".into());

	properties
}
