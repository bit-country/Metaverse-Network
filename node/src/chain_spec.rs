use sc_chain_spec::ChainSpecExtension;
use sp_core::{Pair, Public, crypto::UncheckedInto, sr25519};
use serde::{Serialize, Deserialize};
use bitcountry_runtime::{
    AuthorityDiscoveryConfig, BabeConfig, BalancesConfig, ContractsConfig, CouncilConfig,
    DemocracyConfig, GrandpaConfig, ImOnlineConfig, SessionConfig, SessionKeys, StakerStatus,
    StakingConfig, ElectionsConfig, IndicesConfig, SocietyConfig, SudoConfig, SystemConfig,
    TechnicalCommitteeConfig, wasm_binary_unwrap,
};
use bitcountry_runtime::Block;
use bitcountry_runtime::constants::currency::*;
use sc_service::ChainType;
use hex_literal::hex;
use sc_telemetry::TelemetryEndpoints;
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_consensus_babe::{AuthorityId as BabeId};
use pallet_im_online::sr25519::{AuthorityId as ImOnlineId};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_runtime::{Perbill, traits::{Verify, IdentifyAccount}};
use sc_chain_spec::Properties;

pub use primitives::{AccountId, Balance, Signature};
pub use bitcountry_runtime::GenesisConfig;

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

type AccountPublic = <Signature as Verify>::Signer;

/// Node `ChainSpec` extensions.
///
/// Additional parameters for some Substrate core modules,
/// customizable from the chain spec.
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
    /// Block numbers with known hashes.
    pub fork_blocks: sc_client_api::ForkBlocks<Block>,
    /// Known bad block hashes.
    pub bad_blocks: sc_client_api::BadBlocks<Block>,
}

/// Specialized `ChainSpec`.
pub type ChainSpec = sc_service::GenericChainSpec<
    GenesisConfig,
    Extensions,
>;

fn session_keys(
    grandpa: GrandpaId,
    babe: BabeId,
    im_online: ImOnlineId,
    authority_discovery: AuthorityDiscoveryId,
) -> SessionKeys {
    SessionKeys { grandpa, babe, im_online, authority_discovery }
}


/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
    where
        AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Helper function to generate stash, controller and session key from seed
pub fn authority_keys_from_seed(seed: &str) -> (
    AccountId,
    AccountId,
    GrandpaId,
    BabeId,
    ImOnlineId,
    AuthorityDiscoveryId,
) {
    (
        get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
        get_account_id_from_seed::<sr25519::Public>(seed),
        get_from_seed::<GrandpaId>(seed),
        get_from_seed::<BabeId>(seed),
        get_from_seed::<ImOnlineId>(seed),
        get_from_seed::<AuthorityDiscoveryId>(seed),
    )
}

fn development_config_genesis() -> GenesisConfig {
    testnet_genesis(
        vec![
            authority_keys_from_seed("Alice"),
        ],
        vec![],
        get_account_id_from_seed::<sr25519::Public>("Alice"),
        None,
        true,
    )
}

pub fn development_config() -> ChainSpec {
    ChainSpec::from_genesis(
        "Bit.Country Dev Chain",
        "dev",
        ChainType::Development,
        development_config_genesis,
        vec![],
        None,
        None,
        Some(bitcountry_properties()),
        Default::default(),
    )
}

fn local_testnet_genesis() -> GenesisConfig {
    testnet_genesis(
        vec![
            authority_keys_from_seed("Alice"),
            authority_keys_from_seed("Bob"),
        ],
        vec![],
        get_account_id_from_seed::<sr25519::Public>("Alice"),
        None,
        false,
    )
}

/// Local testnet config (multivalidator Alice + Bob)
pub fn local_testnet_config() -> ChainSpec {
    ChainSpec::from_genesis(
        "Bit.Country Test Chain",
        "local_testnet",
        ChainType::Local,
        local_testnet_genesis,
        vec![],
        None,
        None,
        Some(bitcountry_properties()),
        Default::default(),
    )
}

fn tewai_testnet_genesis() -> GenesisConfig {
    let initial_authorities: Vec<(
        AccountId,
        AccountId,
        GrandpaId,
        BabeId,
        ImOnlineId,
        AuthorityDiscoveryId,
    )> = vec![(
                  //5Gsdy68uCzZFq6SRmQjprH2tLMdCcPeQzKJJYPiaDFEjbGaC
                  hex!["d4bc601183193eaa19e75d6204612e07a29983263808244121f8137727d9d37a"].into(),
                  //5Dqy8KtwmGJd6Tkar8Va3Uw7xvX4RQAhrygUk3T8vUxDXf2a
                  hex!["4ec1ae0facb941380f72f314a5ef6c3ee012a3e105e34806537e3f3c4a3ff167"].into(),
                  //5CnqqYWT3TRFSNtUeuBGqAy3W6xFC4cDxMrNLrRejWDronbi
                  hex!["20232c0f2f4d25e2a710c39eb8dc02aef9d22347e190c8d0fec32cc8b7164da8"].unchecked_into(),
                  hex!["4ec1ae0facb941380f72f314a5ef6c3ee012a3e105e34806537e3f3c4a3ff167"].unchecked_into(),
                  hex!["4ec1ae0facb941380f72f314a5ef6c3ee012a3e105e34806537e3f3c4a3ff167"].unchecked_into(),
                  hex!["4ec1ae0facb941380f72f314a5ef6c3ee012a3e105e34806537e3f3c4a3ff167"].unchecked_into(),
              ), (
                  //5HU23akSFB1bXQjoz7GUHxqjdNXBQRbiooQ81W3cCE1o7ccu
                  hex!["eef509d51bd1bb8d7949a9b70bfb38eadde15e15746a4ef61d9c980696363d2a"].into(),
                  //5C55yE7mBnunBTys1PRwmuT8FA9N4V7e73Tn5LNhM8TPy2ne
                  hex!["004a7071e8289209e72c7d757613b57c784b5633646e290a4ee8e75a6906661f"].into(),
                  //5EsijiPvJpyHW5XX9L3ZpQEeMfACPSzpXHvMSeTz33qUmGMv
                  hex!["7c53f6add892707b55875073c2a781c4e6358372995faca0cb81c24875673814"].unchecked_into(),
                  hex!["004a7071e8289209e72c7d757613b57c784b5633646e290a4ee8e75a6906661f"].unchecked_into(),
                  hex!["004a7071e8289209e72c7d757613b57c784b5633646e290a4ee8e75a6906661f"].unchecked_into(),
                  hex!["004a7071e8289209e72c7d757613b57c784b5633646e290a4ee8e75a6906661f"].unchecked_into(),
              )];
    // generated with secret: subkey inspect "$secret"/fir
    let root_key: AccountId = hex![
		// 5Dqy8KtwmGJd6Tkar8Va3Uw7xvX4RQAhrygUk3T8vUxDXf2a
		"4ec1ae0facb941380f72f314a5ef6c3ee012a3e105e34806537e3f3c4a3ff167"
	].into();

    let endowed_accounts: Vec<AccountId> = vec![root_key.clone()];

    testnet_genesis(
        initial_authorities,
        vec![],
        root_key,
        Some(endowed_accounts),
        false,
    )
}

pub fn tewai_testnet_config() -> ChainSpec {
    ChainSpec::from_genesis(
        "Bit.Country Tewai Chain",
        "tewai_testnet",
        ChainType::Live,
        local_testnet_genesis,
        vec![],
        None,
        None,
        Some(bitcountry_properties()),
        Default::default(),
    )
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
    initial_authorities: Vec<(
        AccountId,
        AccountId,
        GrandpaId,
        BabeId,
        ImOnlineId,
        AuthorityDiscoveryId,
    )>,
    initial_nominators: Vec<AccountId>,
    root_key: AccountId,
    endowed_accounts: Option<Vec<AccountId>>,
    enable_println: bool,
) -> bitcountry_runtime::GenesisConfig {
    //Initial endowned if no endowned accounts
    let mut endowed_accounts: Vec<AccountId> = endowed_accounts.unwrap_or_else(|| {
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
        ]
    });

    // endow all authorities and nominators.
    initial_authorities.iter().map(|x| &x.0).chain(initial_nominators.iter()).for_each(|x| {
        if !endowed_accounts.contains(&x) {
            endowed_accounts.push(x.clone())
        }
    });

    // stakers: all validators and nominators.
    let mut rng = rand::thread_rng();
    let stakers = initial_authorities
        .iter()
        .map(|x| (x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator))
        .chain(initial_nominators.iter().map(|x| {
            use rand::{seq::SliceRandom, Rng};
            let limit = (32 as usize).min(initial_authorities.len());
            let count = rng.gen::<usize>() % limit;
            let nominations = initial_authorities
                .as_slice()
                .choose_multiple(&mut rng, count)
                .into_iter()
                .map(|choice| choice.0.clone())
                .collect::<Vec<_>>();
            (x.clone(), x.clone(), STASH, StakerStatus::Nominator(nominations))
        }))
        .collect::<Vec<_>>();

    let num_endowed_accounts = endowed_accounts.len();

    const ENDOWMENT: Balance = 100_000_000_000 * DOLLARS;
    const STASH: Balance = ENDOWMENT / 1000;

    bitcountry_runtime::GenesisConfig {
        frame_system: Some(SystemConfig {
            code: wasm_binary_unwrap().to_vec(),
            changes_trie_config: Default::default(),
        }),
        pallet_balances: Some(BalancesConfig {
            balances: endowed_accounts.iter().cloned()
                .map(|x| (x, ENDOWMENT))
                .collect()
        }),
        pallet_indices: Some(IndicesConfig {
            indices: vec![],
        }),
        pallet_session: Some(SessionConfig {
            keys: initial_authorities.iter().map(|x| {
                (x.0.clone(), x.0.clone(), session_keys(
                    x.2.clone(),
                    x.3.clone(),
                    x.4.clone(),
                    x.5.clone(),
                ))
            }).collect::<Vec<_>>(),
        }),
        pallet_staking: Some(StakingConfig {
            validator_count: initial_authorities.len() as u32 * 2,
            minimum_validator_count: initial_authorities.len() as u32,
            invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
            slash_reward_fraction: Perbill::from_percent(10),
            stakers,
            ..Default::default()
        }),
        pallet_democracy: Some(DemocracyConfig::default()),
        pallet_elections_phragmen: Some(ElectionsConfig {
            members: endowed_accounts.iter()
                .take((num_endowed_accounts + 1) / 2)
                .cloned()
                .map(|member| (member, STASH))
                .collect(),
        }),
        pallet_collective_Instance1: Some(CouncilConfig::default()),
        pallet_collective_Instance2: Some(TechnicalCommitteeConfig {
            members: endowed_accounts.iter()
                .take((num_endowed_accounts + 1) / 2)
                .cloned()
                .collect(),
            phantom: Default::default(),
        }),
        pallet_contracts: Some(ContractsConfig {
            current_schedule: pallet_contracts::Schedule {
                enable_println, // this should only be enabled on development chains
                ..Default::default()
            },
        }),
        pallet_sudo: Some(SudoConfig {
            key: root_key,
        }),
        pallet_babe: Some(BabeConfig {
            authorities: vec![],
        }),
        pallet_im_online: Some(ImOnlineConfig {
            keys: vec![],
        }),
        pallet_authority_discovery: Some(AuthorityDiscoveryConfig {
            keys: vec![],
        }),
        pallet_grandpa: Some(GrandpaConfig {
            authorities: vec![],
        }),
        pallet_treasury: Some(Default::default()),
        pallet_society: Some(SocietyConfig {
            members: endowed_accounts.iter()
                .take((num_endowed_accounts + 1) / 2)
                .cloned()
                .collect(),
            pot: 0,
            max_members: 999,
        }),
        pallet_vesting: Some(Default::default()),
        // continuum: Some(ContinuumConfig {
        //     initial_active_session: Default::default(),
        //     initial_auction_rate: 5,
        //     initial_max_bound: (-100, 100),
        //     spot_price: 5 * DOLLARS,
        // }),
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