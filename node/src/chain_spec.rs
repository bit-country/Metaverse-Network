use bitcountry_runtime::{BalancesConfig, OrmlNFTConfig, SudoConfig, SystemConfig, WASM_BINARY};
use cumulus_primitives_core::ParaId;
use hex_literal::hex;
use primitives::{AccountId, Signature};
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::{ChainType, Properties};
use serde::{Deserialize, Serialize};
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::crypto::UncheckedInto;
use sp_core::{sr25519, Pair, Public};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{IdentifyAccount, Verify};
use sp_runtime::{FixedPointNumber, FixedU128, Perbill};

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

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(seed: &str) -> (AccountId, AccountId, GrandpaId, BabeId) {
    (
        get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
        get_account_id_from_seed::<sr25519::Public>(seed),
        get_from_seed::<GrandpaId>(seed),
        get_from_seed::<BabeId>(seed),
    )
}

// pub fn development_config(id: ParaId) -> ChainSpec {

// 	let wasm_binary = bitcountry_runtime::WASM_BINARY.ok_or("Bit Country Tewai runtime wasm binary not available")?;

// 	ChainSpec::from_genesis(
// 		// Name
// 		"Bit.Country Dev Chain",
// 		// ID
// 		"bit-country-dev",
// 		ChainType::Development,
// 		move || {
// 			testnet_genesis(
// 				wasm_binary,
// 				get_account_id_from_seed::<sr25519::Public>("Alice"),
// 				vec![
// 					get_account_id_from_seed::<sr25519::Public>("Alice"),
// 				],
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
// 		Vec::new(),
// 		None,
// 		None,
// 		// Properties
// 		Some(bitcountry_properties()),
// 		Extensions {
// 			relay_chain: "westend-dev".into(),
// 			para_id: id.into(),
// 		},
// 	)
// }

pub fn development_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or("Development wasm binary not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        // Name
        "Bit.Country Tewai Chain",
        // ID
        "dev",
        ChainType::Live,
        move || {
            testnet_genesis(
                wasm_binary,
                // Sudo account
                hex!["c0568109deb73ec388a329981e589a8d9baccd64a14107468aefae8806a70f2e"].into(),
                // initial authorities accounts
                vec![
                    (
                        hex!["c0568109deb73ec388a329981e589a8d9baccd64a14107468aefae8806a70f2e"]
                            .into(),
                        hex!["c0568109deb73ec388a329981e589a8d9baccd64a14107468aefae8806a70f2e"]
                            .into(),
                        hex!["20232c0f2f4d25e2a710c39eb8dc02aef9d22347e190c8d0fec32cc8b7164da8"]
                            .unchecked_into(),
                        hex!["4ec1ae0facb941380f72f314a5ef6c3ee012a3e105e34806537e3f3c4a3ff167"]
                            .unchecked_into(),
                    ),
                    (
                        hex!["c0568109deb73ec388a329981e589a8d9baccd64a14107468aefae8806a70f2e"]
                            .into(),
                        hex!["c0568109deb73ec388a329981e589a8d9baccd64a14107468aefae8806a70f2e"]
                            .into(),
                        hex!["301b411ee0106a9c1da1ed28f0b965d2ced9ac6fb9fc34896f5e461a050b0143"]
                            .unchecked_into(),
                        hex!["004a7071e8289209e72c7d757613b57c784b5633646e290a4ee8e75a6906661f"]
                            .unchecked_into(),
                    ),
                ],
                // Pre-funded accounts
                vec![
                    /* Sudo Account */
                    hex!["c0568109deb73ec388a329981e589a8d9baccd64a14107468aefae8806a70f2e"].into(),
                ],
                8.into(),
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
            relay_chain: "rococo".into(),
            para_id: 8888_u32.into(),
        },
    ))
}

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
                wasm_binary,
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                vec![
                    authority_keys_from_seed("Alice"),
                    authority_keys_from_seed("Bob"),
                ],
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
            relay_chain: "rococo".into(),
            para_id: 8_u32.into(),
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
                wasm_binary,
                // Sudo account
                hex!["c0568109deb73ec388a329981e589a8d9baccd64a14107468aefae8806a70f2e"].into(),
                // initial authorities accounts
                vec![
                    (
                        hex!["c0568109deb73ec388a329981e589a8d9baccd64a14107468aefae8806a70f2e"]
                            .into(),
                        hex!["c0568109deb73ec388a329981e589a8d9baccd64a14107468aefae8806a70f2e"]
                            .into(),
                        hex!["20232c0f2f4d25e2a710c39eb8dc02aef9d22347e190c8d0fec32cc8b7164da8"]
                            .unchecked_into(),
                        hex!["4ec1ae0facb941380f72f314a5ef6c3ee012a3e105e34806537e3f3c4a3ff167"]
                            .unchecked_into(),
                    ),
                    (
                        hex!["c0568109deb73ec388a329981e589a8d9baccd64a14107468aefae8806a70f2e"]
                            .into(),
                        hex!["c0568109deb73ec388a329981e589a8d9baccd64a14107468aefae8806a70f2e"]
                            .into(),
                        hex!["301b411ee0106a9c1da1ed28f0b965d2ced9ac6fb9fc34896f5e461a050b0143"]
                            .unchecked_into(),
                        hex!["004a7071e8289209e72c7d757613b57c784b5633646e290a4ee8e75a6906661f"]
                            .unchecked_into(),
                    ),
                ],
                // Pre-funded accounts
                vec![
                    /* Sudo Account */
                    hex!["c0568109deb73ec388a329981e589a8d9baccd64a14107468aefae8806a70f2e"].into(),
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
            relay_chain: "rococo".into(),
            para_id: id.into(),
        },
    ))
}

// fn tewai_session_keys(grandpa: GrandpaId, babe: BabeId) -> bitcountry_runtime::SessionKeys {
//     bitcountry_runtime::SessionKeys { grandpa, babe }
// }

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
    wasm_binary: &[u8],
    root_key: AccountId,
    initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId)>,
    endowed_accounts: Vec<AccountId>,
    id: ParaId,
) -> bitcountry_runtime::GenesisConfig {
    use bitcountry_runtime::{StakerStatus, DOLLARS};

    const INITIAL_BALANCE: u128 = 1_000_000 * DOLLARS;
    const INITIAL_STAKING: u128 = 100_000 * DOLLARS;

    bitcountry_runtime::GenesisConfig {
        frame_system: SystemConfig {
            code: bitcountry_runtime::WASM_BINARY
                .expect("WASM binary was not build, please build it!")
                .to_vec(),
            changes_trie_config: Default::default(),
        },
        pallet_balances: BalancesConfig {
            // Configure endowed accounts with initial balance of 1 billion token to test.
            // balances: endowed_accounts.iter().cloned().map(|k|(k)).collect(),
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, INITIAL_BALANCE))
                .collect(),
        },
        orml_nft: OrmlNFTConfig { tokens: vec![] },
        // pallet_session: Some(SessionConfig {
        //     keys: initial_authorities
        //         .iter()
        //         .map(|x| {
        //             (
        //                 x.0.clone(),
        //                 x.0.clone(),
        //                 tewai_session_keys(x.2.clone(), x.3.clone()),
        //             )
        //         })
        //         .collect::<Vec<_>>(),
        // }),
        // pallet_staking: Some(StakingConfig {
        //     validator_count: initial_authorities.len() as u32 * 2,
        //     minimum_validator_count: initial_authorities.len() as u32,
        //     stakers: initial_authorities
        //         .iter()
        //         .map(|x| {
        //             (
        //                 x.0.clone(),
        //                 x.1.clone(),
        //                 INITIAL_STAKING,
        //                 StakerStatus::Validator,
        //             )
        //         })
        //         .collect(),
        //     invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
        //     slash_reward_fraction: Perbill::from_percent(10),
        //     ..Default::default()
        // }),
        // pallet_grandpa: Some(GrandpaConfig {
        //     authorities: vec![],
        // }),
        // pallet_babe: Some(BabeConfig {
        //     authorities: vec![],
        // }),
        // pallet_aura: Some(AuraConfig {
        // 	authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
        // }),
        // pallet_grandpa: Some(GrandpaConfig {
        // 	authorities: initial_authorities
        // 		.iter()
        // 		.map(|x| (x.1.clone(), 1))
        // 		.collect(),
        // }),
        pallet_collective_Instance1: Default::default(),
        pallet_treasury: Default::default(),
        pallet_sudo: SudoConfig {
            // Assign network admin rights.
            key: root_key,
        },
        // pallet_contracts: Some(ContractsConfig {
        // 	current_schedule: pallet_contracts::Schedule {
        // 		enable_println,
        // 		..Default::default()
        // 	},
        // }),
        parachain_info: bitcountry_runtime::ParachainInfoConfig { parachain_id: id },
        // tokenization: Some(TokenConfig {
        // 	init_token_id: 0
        // })
    }
}

pub fn bitcountry_properties() -> Properties {
    let mut properties = Properties::new();

    properties.insert("ss58Format".into(), 28.into());
    properties.insert("tokenDecimals".into(), 18.into());
    properties.insert("tokenSymbol".into(), "NUUM".into());

    properties
}
