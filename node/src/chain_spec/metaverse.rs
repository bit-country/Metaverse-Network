use std::collections::BTreeMap;
use std::str::FromStr;

use hex_literal::hex;
use log::info;
use sc_service::{ChainType, Properties};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::crypto::UncheckedInto;
use sp_core::{sr25519, Pair, Public, H160, U256};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::{
	traits::{IdentifyAccount, Verify},
	Perbill,
};

use metaverse_runtime::{
	constants::currency::*, opaque::SessionKeys, wasm_binary_unwrap, AccountId, AuraConfig, BalancesConfig,
	BaseFeeConfig, CollatorSelectionConfig, DemocracyConfig, EVMConfig, EstateConfig, EthereumConfig, GenesisAccount,
	GenesisConfig, GrandpaConfig, MintingRange, MintingRateInfo, OracleMembershipConfig, SessionConfig, Signature,
	SudoConfig, SystemConfig,
};
use primitives::Balance;

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

fn session_keys(aura: AuraId, grandpa: GrandpaId) -> SessionKeys {
	SessionKeys { aura, grandpa }
}

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AccountId, AuraId, GrandpaId) {
	(
		get_account_id_from_seed::<sr25519::Public>(s),
		get_from_seed::<AuraId>(s),
		get_from_seed::<GrandpaId>(s),
	)
}

pub fn development_config() -> Result<ChainSpec, String> {
	Ok(ChainSpec::from_genesis(
		// Name
		"Metaverse Dev",
		// ID
		"metaverse-dev",
		ChainType::Development,
		move || {
			testnet_genesis(
				// Initial PoA authorities
				vec![authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
				],
				true,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		None,
		// Properties
		Some(metaverse_properties()),
		// Extensions
		None,
	))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
	Ok(ChainSpec::from_genesis(
		// Name
		"Metaverse Local",
		// ID
		"metaverse-local",
		ChainType::Local,
		move || {
			testnet_genesis(
				// Initial PoA authorities
				vec![authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
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
				true,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		None,
		// Properties
		Some(metaverse_properties()),
		// Extensions
		None,
	))
}

pub fn metaverse_genesis() -> GenesisConfig {
	let aura_authorities: Vec<(AccountId, AuraId, GrandpaId)> = vec![
		(
			// 5FpqLqqbFyYWgYtgQS11HvTkaripk1nPFFti6CwDaMj8cSvu
			hex!["a65cb28d2524996ee0e02aa1ebfa5c1b4ff3db7edad9b11f7033960cc5aa3c3e"].into(),
			hex!["a65cb28d2524996ee0e02aa1ebfa5c1b4ff3db7edad9b11f7033960cc5aa3c3e"].unchecked_into(),
			hex!["2098c0df8dd97903bf2203bda7ba5aa6afaf3b5e353f60bc42000dca351c6a20"].unchecked_into(),
		),
		(
			// 5EUXjqNx3Rsh3wtDJAPBzEvJdGVD3QmxmMUjrfARNr4uh7pq
			hex!["6aa44c06b0a479f95757137a1b08fd00971823430147094dc66e7aa2b381f146"].into(),
			hex!["6aa44c06b0a479f95757137a1b08fd00971823430147094dc66e7aa2b381f146"].unchecked_into(),
			hex!["ed0524b8e41b652c36381c0d77ab80129c39070a808ab53586177804291acc79"].unchecked_into(),
		),
	];

	// generated with secret: subkey inspect "$secret"/fir
	let root_key: AccountId = hex![
		// 5Dqy8KtwmGJd6Tkar8Va3Uw7xvX4RQAhrygUk3T8vUxDXf2a
		"4ec1ae0facb941380f72f314a5ef6c3ee012a3e105e34806537e3f3c4a3ff167"
	]
	.into();

	let endowed_accounts: Vec<AccountId> = vec![root_key.clone()];

	testnet_genesis(
		// Initial PoA authorities
		aura_authorities,
		// Sudo account
		root_key,
		// Pre-funded accounts
		endowed_accounts,
		true,
	)
}

pub fn metaverse_testnet_config() -> Result<ChainSpec, String> {
	Ok(ChainSpec::from_genesis(
		// Name
		"Metaverse Testnet",
		// ID
		"metaverse-testnet",
		ChainType::Live,
		metaverse_genesis,
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		None,
		// Properties
		Some(metaverse_properties()),
		// Extensions
		None,
	))
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

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
	initial_authorities: Vec<(AccountId, AuraId, GrandpaId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	_enable_println: bool,
) -> GenesisConfig {
	let staking_candidate: Vec<(AccountId, Balance)> = initial_authorities
		.iter()
		.clone()
		.map(|x| (x.0.clone(), 100 * DOLLARS))
		.collect();

	let session_keys_test: Vec<(AccountId, AccountId, SessionKeys)> = initial_authorities
		.iter()
		.map(|x| (x.0.clone(), x.0.clone(), session_keys(x.1.clone(), x.2.clone())))
		.collect::<Vec<_>>();
	info!("{}", session_keys_test[0].0);
	info!("{}", staking_candidate[0].0);
	info!("{}", staking_candidate[1].0);

	GenesisConfig {
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary_unwrap().to_vec(),
		},
		balances: BalancesConfig {
			// Configure endowed accounts with initial balance of 1 << 60.
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, 600 * KILODOLLARS))
				.collect(),
		},
		aura: AuraConfig {
			//            authorities: initial_authorities.iter().map(|x| (x.1.clone())).collect(),
			authorities: vec![],
		},
		grandpa: GrandpaConfig {
			//            authorities: initial_authorities.iter().map(|x| (x.2.clone(), 1)).collect(),
			authorities: vec![],
		},
		sudo: SudoConfig {
			// Assign network admin rights.
			key: Some(root_key.clone()),
		},
		collator_selection: CollatorSelectionConfig {
			invulnerables: initial_authorities.iter().cloned().map(|(acc, _, _)| acc).collect(),
			candidacy_bond: 16 * CENTS,
			..Default::default()
		},
		council: Default::default(),
		democracy: DemocracyConfig::default(),
		tokens: Default::default(),
		treasury: Default::default(),
		vesting: Default::default(),
		session: SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| (x.0.clone(), x.0.clone(), session_keys(x.1.clone(), x.2.clone())))
				.collect::<Vec<_>>(),
		},
		estate: EstateConfig {
			minting_rate_config: metaverse_land_minting_config(),
		},
		oracle_membership: OracleMembershipConfig {
			members: BoundedVec<AccountId, OracleMaxMembers>::new(),
			phantom: Default::default(),
		},
		evm: EVMConfig {
			accounts: {
				// Prefund the "ALICE" account
				let mut accounts = std::collections::BTreeMap::new();
				accounts.insert(
					/*SS58: 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
					 * hex: 0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d
					 * Using the full hex key, truncating to the first 20 bytes (the first 40 hex chars)
					 *H160::from_str("d43593c715fdd31c61141abd04a99fd6822c8558").expect("internal H160 is valid;
					 * qed"), */
					H160::from_slice(&hex_literal::hex!("d43593c715fdd31c61141abd04a99fd6822c8558")),
					GenesisAccount {
						nonce: U256::zero(),
						// Using a larger number, so I can tell the accounts apart by balance.
						balance: U256::from(1u64 << 61),
						code: vec![],
						storage: std::collections::BTreeMap::new(),
					},
				);
				accounts.insert(
					// H160 address of CI test runner account
					H160::from_str("6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b").expect("internal H160 is valid; qed"),
					GenesisAccount {
						balance: U256::from(1u64 << 61),
						code: Default::default(),
						nonce: Default::default(),
						storage: Default::default(),
					},
				);
				accounts
			},
		},
		ethereum: EthereumConfig {},
		base_fee: BaseFeeConfig::new(
			sp_core::U256::from(1_000_000_000u64),
			false,
			sp_runtime::Permill::from_parts(125_000),
		),
	}
}

pub fn metaverse_properties() -> Properties {
	let mut properties = Properties::new();

	properties.insert("ss58Format".into(), 42.into());
	properties.insert("tokenDecimals".into(), 18.into());
	properties.insert("tokenSymbol".into(), "NUUM".into());

	properties
}
