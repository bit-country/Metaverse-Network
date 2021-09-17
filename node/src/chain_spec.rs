use node_template_runtime::{
	AccountId, AuraConfig, BalancesConfig, GenesisConfig, GrandpaConfig, Signature, SudoConfig, SystemConfig,
	WASM_BINARY,
};
use sc_service::ChainType;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{sr25519, Pair, Public};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{IdentifyAccount, Verify};

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

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
pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
	(get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
}

pub fn development_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
		ChainType::Development,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![authority_keys_from_seed("Alice")],
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
		// Properties
		None,
		// Extensions
		None,
	))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Local Testnet",
		// ID
		"local_testnet",
		ChainType::Local,
		move || {
			testnet_genesis(
				wasm_binary,
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
		// Properties
		None,
		// Extensions
		None,
	))
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AuraId, GrandpaId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	_enable_println: bool,
) -> GenesisConfig {
	GenesisConfig {
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
			changes_trie_config: Default::default(),
		},
		balances: BalancesConfig {
			// Configure endowed accounts with initial balance of 1 << 60.
			balances: endowed_accounts.iter().cloned().map(|k| (k, 1 << 60)).collect(),
		},
		aura: AuraConfig {
			authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
		},
		grandpa: GrandpaConfig {
			authorities: initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect(),
		},
		sudo: SudoConfig {
			// Assign network admin rights.
			key: root_key,
		},
	}
}

//use bitcountry_runtime::{
//    constants::currency::*, opaque::Block, opaque::SessionKeys, wasm_binary_unwrap, AuraConfig,
//    AuthorityDiscoveryConfig, BabeConfig, BalancesConfig, ContinuumConfig, CouncilConfig,
//    DemocracyConfig, ElectionsConfig, GenesisConfig, GrandpaConfig, ImOnlineConfig, IndicesConfig,
//    SessionConfig, StakerStatus, StakingConfig, SudoConfig, SystemConfig,
// TechnicalCommitteeConfig,    BABE_GENESIS_EPOCH_CONFIG,
//};
//use hex_literal::hex;
//use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
//pub use primitives::{AccountId, Balance, Signature};
//use sc_chain_spec::ChainSpecExtension;
//use sc_chain_spec::Properties;
//use sc_service::ChainType;
//use sc_telemetry::TelemetryEndpoints;
//use serde::{Deserialize, Serialize};
//use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
//use sp_consensus_babe::AuthorityId as BabeId;
//use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public};
//use sp_finality_grandpa::AuthorityId as GrandpaId;
//use sp_runtime::{
//    traits::{IdentifyAccount, Verify},
//    Perbill,
//};
//
//// The URL for the telemetry server.
//// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";
//
////type AccountPublic = <Signature as Verify>::Signer;
//
///// Node `ChainSpec` extensions.
/////
///// Additional parameters for some Substrate core modules,
///// customizable from the chain spec.
//#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
//#[serde(rename_all = "camelCase")]
//pub struct Extensions {
//    /// Block numbers with known hashes.
//    pub fork_blocks: sc_client_api::ForkBlocks<Block>,
//    /// Known bad block hashes.
//    pub bad_blocks: sc_client_api::BadBlocks<Block>,
//    /// The light sync state extension used by the sync-state rpc.
//    pub light_sync_state: sc_sync_state_rpc::LightSyncStateExtension,
//}
//
///// Specialized `ChainSpec`.
//pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;
//
//fn session_keys(
//    grandpa: GrandpaId,
//    babe: BabeId,
//    im_online: ImOnlineId,
//    authority_discovery: AuthorityDiscoveryId,
//) -> SessionKeys {
//    SessionKeys {
//        grandpa,
//        babe,
//        im_online,
//        authority_discovery,
//    }
//}
/////// Helper function to generate a crypto pair from seed
////pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
////    TPublic::Pair::from_string(&format!("//{}", seed), None)
////        .expect("static values are valid; qed")
////        .public()
////}
////
/////// Generate an account ID from seed.
////pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
////where
////    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
////{
////    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
////}
////
/////// Helper function to generate stash, controller and session key from seed
////pub fn authority_keys_from_seed(
////    seed: &str,
////) -> (
////    AccountId,
////    AccountId,
////    GrandpaId,
////    BabeId,
////    ImOnlineId,
////    AuthorityDiscoveryId,
////) {
////    (
////        get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
////        get_account_id_from_seed::<sr25519::Public>(seed),
////        get_from_seed::<GrandpaId>(seed),
////        get_from_seed::<BabeId>(seed),
////        get_from_seed::<ImOnlineId>(seed),
////        get_from_seed::<AuthorityDiscoveryId>(seed),
////    )
////}
//
///// Generate a crypto pair from seed.
//pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
//    TPublic::Pair::from_string(&format!("//{}", seed), None)
//        .expect("static values are valid; qed")
//        .public()
//}
//
//type AccountPublic = <Signature as Verify>::Signer;
//
///// Generate an account ID from seed.
//pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
//where
//    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
//{
//    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
//}
//
///// Generate an Aura authority key.
//pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
//    (get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
//}
//
//fn development_config_genesis() -> bitcountry_runtime::GenesisConfig {
//    testnet_genesis(
//        vec![authority_keys_from_seed("Alice")],
//        vec![],
//        get_account_id_from_seed::<sr25519::Public>("Alice"),
//        None,
//        true,
//    )
//}
//
//pub fn development_config() -> ChainSpec {
//    ChainSpec::from_genesis(
//        "Bit.Country Dev Chain",
//        "dev",
//        ChainType::Development,
//        development_config_genesis,
//        vec![],
//        None,
//        None,
//        Some(bitcountry_properties()),
//        Default::default(),
//    )
//}
//
//fn local_testnet_genesis() -> bitcountry_runtime::GenesisConfig {
//    testnet_genesis(
//        vec![
//            authority_keys_from_seed("Alice"),
//            authority_keys_from_seed("Bob"),
//        ],
//        vec![],
//        get_account_id_from_seed::<sr25519::Public>("Alice"),
//        None,
//        false,
//    )
//}
//
///// Local testnet config (multivalidator Alice + Bob)
//pub fn local_testnet_config() -> ChainSpec {
//    ChainSpec::from_genesis(
//        "Bit.Country Test Chain",
//        "local_testnet",
//        ChainType::Local,
//        local_testnet_genesis,
//        vec![],
//        None,
//        None,
//        Some(bitcountry_properties()),
//        Default::default(),
//    )
//}
//
//fn tewai_testnet_genesis() -> bitcountry_runtime::GenesisConfig {
//    let initial_authorities: Vec<(
//        AccountId,
//        AccountId,
//        GrandpaId,
//        BabeId,
//        ImOnlineId,
//        AuthorityDiscoveryId,
//    )> = vec![
//        (
//            // 5FpqLqqbFyYWgYtgQS11HvTkaripk1nPFFti6CwDaMj8cSvu
//            hex!["a65cb28d2524996ee0e02aa1ebfa5c1b4ff3db7edad9b11f7033960cc5aa3c3e"].into(),
//            hex!["a65cb28d2524996ee0e02aa1ebfa5c1b4ff3db7edad9b11f7033960cc5aa3c3e"].into(),
//            hex!["2098c0df8dd97903bf2203bda7ba5aa6afaf3b5e353f60bc42000dca351c6a20"]
//                .unchecked_into(),
//            hex!["a65cb28d2524996ee0e02aa1ebfa5c1b4ff3db7edad9b11f7033960cc5aa3c3e"]
//                .unchecked_into(),
//            hex!["a65cb28d2524996ee0e02aa1ebfa5c1b4ff3db7edad9b11f7033960cc5aa3c3e"]
//                .unchecked_into(),
//            hex!["a65cb28d2524996ee0e02aa1ebfa5c1b4ff3db7edad9b11f7033960cc5aa3c3e"]
//                .unchecked_into(),
//        ),
//        (
//            // 5EUXjqNx3Rsh3wtDJAPBzEvJdGVD3QmxmMUjrfARNr4uh7pq
//            hex!["6aa44c06b0a479f95757137a1b08fd00971823430147094dc66e7aa2b381f146"].into(),
//            hex!["6aa44c06b0a479f95757137a1b08fd00971823430147094dc66e7aa2b381f146"].into(),
//            hex!["ed0524b8e41b652c36381c0d77ab80129c39070a808ab53586177804291acc79"]
//                .unchecked_into(),
//            hex!["6aa44c06b0a479f95757137a1b08fd00971823430147094dc66e7aa2b381f146"]
//                .unchecked_into(),
//            hex!["6aa44c06b0a479f95757137a1b08fd00971823430147094dc66e7aa2b381f146"]
//                .unchecked_into(),
//            hex!["6aa44c06b0a479f95757137a1b08fd00971823430147094dc66e7aa2b381f146"]
//                .unchecked_into(),
//        ),
//    ];
//
//    let aura_authorities: Vec<(AuraId, GrandpaId)> = vec![
//        (
//            // 5FpqLqqbFyYWgYtgQS11HvTkaripk1nPFFti6CwDaMj8cSvu
//            hex!["a65cb28d2524996ee0e02aa1ebfa5c1b4ff3db7edad9b11f7033960cc5aa3c3e"].into(),
//            hex!["2098c0df8dd97903bf2203bda7ba5aa6afaf3b5e353f60bc42000dca351c6a20"]
//                .unchecked_into(),
//        ),
//        (
//            // 5EUXjqNx3Rsh3wtDJAPBzEvJdGVD3QmxmMUjrfARNr4uh7pq
//            hex!["6aa44c06b0a479f95757137a1b08fd00971823430147094dc66e7aa2b381f146"].into(),
//            hex!["ed0524b8e41b652c36381c0d77ab80129c39070a808ab53586177804291acc79"]
//                .unchecked_into(),
//        ),
//    ];
//
//    // generated with secret: subkey inspect "$secret"/fir
//    let root_key: AccountId = hex![
//        // 5Dqy8KtwmGJd6Tkar8Va3Uw7xvX4RQAhrygUk3T8vUxDXf2a
//        "4ec1ae0facb941380f72f314a5ef6c3ee012a3e105e34806537e3f3c4a3ff167"
//    ]
//    .into();
//
//    let endowed_accounts: Vec<AccountId> = vec![root_key.clone()];
//
//    testnet_genesis(
//        aura_authorities,
//        vec![],
//        root_key,
//        Some(endowed_accounts),
//        false,
//    )
//}
//
//pub fn tewai_testnet_config() -> Result<ChainSpec, String> {
//    ChainSpec::from_json_bytes(&include_bytes!("../../node/res/tewaiChainNodeSpecRaw.json")[..])
//}
//
///// Configure initial storage state for FRAME modules.
//fn testnet_genesis(
//    initial_authorities: Vec<(AuraId, GrandpaId)>,
//    //    initial_authorities: Vec<(
//    //        AccountId,
//    //        AccountId,
//    //        GrandpaId,
//    //        BabeId,
//    //        ImOnlineId,
//    //        AuthorityDiscoveryId,
//    //    )>,
//    initial_nominators: Vec<AccountId>,
//    root_key: AccountId,
//    endowed_accounts: Option<Vec<AccountId>>,
//    enable_println: bool,
//) -> GenesisConfig {
//    // Initial endowned if no endowned accounts
//    let mut endowed_accounts: Vec<AccountId> = endowed_accounts.unwrap_or_else(|| {
//        vec![
//            get_account_id_from_seed::<sr25519::Public>("Alice"),
//            get_account_id_from_seed::<sr25519::Public>("Bob"),
//            get_account_id_from_seed::<sr25519::Public>("Charlie"),
//            get_account_id_from_seed::<sr25519::Public>("Dave"),
//            get_account_id_from_seed::<sr25519::Public>("Eve"),
//            get_account_id_from_seed::<sr25519::Public>("Ferdie"),
//            get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
//            get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
//            get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
//            get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
//            get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
//            get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
//        ]
//    });
//
//    // endow all authorities and nominators.
//    initial_authorities
//        .iter()
//        .map(|x| &x.0)
//        .chain(initial_nominators.iter())
//        .for_each(|x| {
//            if !endowed_accounts.contains(&x) {
//                endowed_accounts.push(x.clone())
//            }
//        });
//
//    // stakers: all validators and nominators.
//    let mut rng = rand::thread_rng();
//    let stakers = initial_authorities
//        .iter()
//        .map(|x| (x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator))
//        .chain(initial_nominators.iter().map(|x| {
//            use rand::{seq::SliceRandom, Rng};
//            let limit = (32 as usize).min(initial_authorities.len());
//            let count = rng.gen::<usize>() % limit;
//            let nominations = initial_authorities
//                .as_slice()
//                .choose_multiple(&mut rng, count)
//                .into_iter()
//                .map(|choice| choice.0.clone())
//                .collect::<Vec<_>>();
//            (
//                x.clone(),
//                x.clone(),
//                STASH,
//                StakerStatus::Nominator(nominations),
//            )
//        }))
//        .collect::<Vec<_>>();
//
//    let num_endowed_accounts = endowed_accounts.len();
//
//    const ENDOWMENT: Balance = 300_000_000 * DOLLARS;
//    const STASH: Balance = ENDOWMENT / 100;
//
//    GenesisConfig {
//        system: SystemConfig {
//            code: wasm_binary_unwrap().to_vec(),
//            changes_trie_config: Default::default(),
//        },
//        balances: BalancesConfig {
//            balances: endowed_accounts
//                .iter()
//                .cloned()
//                .map(|x| (x, ENDOWMENT))
//                .collect(),
//        },
//        indices: IndicesConfig { indices: vec![] },
//        session: SessionConfig {
//            keys: initial_authorities
//                .iter()
//                .map(|x| {
//                    (
//                        x.0.clone(),
//                        x.0.clone(),
//                        session_keys(x.2.clone(), x.3.clone(), x.4.clone(), x.5.clone()),
//                    )
//                })
//                .collect::<Vec<_>>(),
//        },
//        staking: StakingConfig {
//            validator_count: initial_authorities.len() as u32 * 2,
//            minimum_validator_count: initial_authorities.len() as u32,
//            invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
//            slash_reward_fraction: Perbill::from_percent(10),
//            stakers,
//            ..Default::default()
//        },
//        democracy: DemocracyConfig::default(),
//        elections: ElectionsConfig {
//            members: endowed_accounts
//                .iter()
//                .take((num_endowed_accounts + 1) / 2)
//                .cloned()
//                .map(|member| (member, STASH))
//                .collect(),
//        },
//        council: CouncilConfig::default(),
//        technical_committee: TechnicalCommitteeConfig {
//            members: endowed_accounts
//                .iter()
//                .take((num_endowed_accounts + 1) / 2)
//                .cloned()
//                .collect(),
//            phantom: Default::default(),
//        },
//        sudo: SudoConfig { key: root_key },
//        //        babe: BabeConfig {
//        //            authorities: vec![],
//        //            epoch_config: Some(bitcountry_runtime::BABE_GENESIS_EPOCH_CONFIG),
//        //        },
//        aura: AuraConfig {
//            authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
//        },
//        im_online: ImOnlineConfig { keys: vec![] },
//        authority_discovery: AuthorityDiscoveryConfig { keys: vec![] },
//        grandpa: GrandpaConfig {
//            authorities: vec![],
//        },
//        treasury: Default::default(),
//        vesting: Default::default(),
//        continuum: ContinuumConfig {
//            initial_active_session: Default::default(),
//            initial_auction_rate: 5,
//            initial_max_bound: (-100, 100),
//            spot_price: 5 * DOLLARS,
//        },
//    }
//}
//
//pub fn bitcountry_properties() -> Properties {
//    let mut properties = Properties::new();
//
//    properties.insert("ss58Format".into(), 28.into());
//    properties.insert("tokenDecimals".into(), 18.into());
//    properties.insert("tokenSymbol".into(), "NUUM".into());
//
//    properties
//}
