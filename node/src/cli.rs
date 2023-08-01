use std::path::PathBuf;

use clap::Parser;
use cumulus_client_cli;
use url::Url;

use crate::chain_spec;
use crate::chain_spec::Extensions;

fn validate_relay_chain_url(arg: &str) -> Result<(), String> {
	let url = Url::parse(arg).map_err(|e| e.to_string())?;

	if url.scheme() == "ws" {
		Ok(())
	} else {
		Err(format!(
			"'{}' URL scheme not supported. Only websocket RPC is currently supported",
			url.scheme()
		))
	}
}

#[cfg(feature = "manual-seal")]
#[derive(Debug, Copy, Clone, clap::ArgEnum)]
pub enum Sealing {
	// Seal using rpc method.
	Manual,
	// Seal when transaction is executed.
	Instant,
}

#[cfg(feature = "manual-seal")]
impl Default for Sealing {
	fn default() -> Sealing {
		Sealing::Manual
	}
}

#[allow(missing_docs)]
#[derive(Debug, clap::Parser)]
pub struct RunCmd {
	#[allow(missing_docs)]
	#[command(flatten)]
	pub base: sc_cli::RunCmd,

	/// Choose sealing method.
	#[cfg(feature = "manual-seal")]
	#[arg(long, arg_enum, ignore_case = true)]
	pub sealing: Sealing,

	#[arg(long)]
	pub enable_dev_signer: bool,

	/// Maximum number of logs in a query.
	#[arg(long, default_value = "10000")]
	pub max_past_logs: u32,

	/// Maximum fee history cache size.
	#[arg(long, default_value = "2048")]
	pub fee_history_limit: u64,

	/// The dynamic-fee pallet target gas price set by block author
	#[arg(long, default_value = "1")]
	pub target_gas_price: u64,

	/// Run node as collator.
	///
	/// Note that this is the same as running with `--validator`.
	#[arg(long, conflicts_with = "validator")]
	pub collator: bool,

	/// Relay chain arguments
	#[arg(
		long,
		value_parser = validate_relay_chain_url,
		num_args = 0..,
		alias = "relay-chain-rpc-url"
	)]
	pub relay_chain_rpc_urls: Vec<Url>,
}

impl RunCmd {
	/// Create a [`NormalizedRunCmd`] which merges the `collator` cli argument into `validator` to
	/// have only one.
	pub fn normalize(&self) -> cumulus_client_cli::NormalizedRunCmd {
		let mut new_base = self.base.clone();

		new_base.validator = self.base.validator || self.collator;

		cumulus_client_cli::NormalizedRunCmd { base: new_base }
	}

	/// Create [`CollatorOptions`] representing options only relevant to parachain collator nodes
	pub fn collator_options(&self) -> cumulus_client_cli::CollatorOptions {
		cumulus_client_cli::CollatorOptions {
			relay_chain_rpc_urls: self.relay_chain_rpc_urls.clone().into(),
		}
	}
}

#[derive(Debug, Parser)]
#[command(
	propagate_version = true,
	args_conflicts_with_subcommands = true,
	subcommand_negates_reqs = true
)]
pub struct Cli {
	#[command(subcommand)]
	pub subcommand: Option<Subcommand>,

	#[command(flatten)]
	pub run: RunCmd,

	/// Relaychain arguments
	#[arg(raw = true)]
	pub relaychain_args: Vec<String>,
}

#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
	/// Key management cli utilities
	#[command(subcommand)]
	Key(sc_cli::KeySubcommand),
	/// Build a chain specification.
	BuildSpec(sc_cli::BuildSpecCmd),

	/// Validate blocks.
	CheckBlock(sc_cli::CheckBlockCmd),

	/// Export blocks.
	ExportBlocks(sc_cli::ExportBlocksCmd),

	/// Export the state of a given block into a chain spec.
	ExportState(sc_cli::ExportStateCmd),

	/// Import blocks.
	ImportBlocks(sc_cli::ImportBlocksCmd),

	/// Remove the whole chain.
	PurgeChain(sc_cli::PurgeChainCmd),

	/// Remove the parachain
	PurgeChainParachain(cumulus_client_cli::PurgeChainCmd),

	/// Revert the chain to a previous state.
	Revert(sc_cli::RevertCmd),

	/// The custom benchmark subcommand benchmarking runtime pallets.
	#[command(subcommand)]
	Benchmark(frame_benchmarking_cli::BenchmarkCmd),

	/// Db meta columns information.
	// FrontierDb(fc_cli::FrontierDbCmd),

	/// Export the genesis state of the parachain.
	#[command(alias = "export-genesis-state")]
	ExportGenesisState(cumulus_client_cli::ExportGenesisStateCommand),

	/// Export the genesis wasm of the parachain.
	#[command(alias = "export-genesis-wasm")]
	ExportGenesisWasm(cumulus_client_cli::ExportGenesisWasmCommand),

	/// Try some command against runtime state.
	#[cfg(feature = "try-runtime")]
	TryRuntime(try_runtime_cli::TryRuntimeCmd),
}

#[derive(Debug)]
pub struct RelayChainCli {
	/// The actual relay chain cli object.
	pub base: polkadot_cli::RunCmd,

	/// Optional chain id that should be passed to the relay chain.
	pub chain_id: Option<String>,

	/// The base path that should be used by the relay chain.
	pub base_path: Option<PathBuf>,
}

impl RelayChainCli {
	/// Parse the relay chain CLI parameters using the para chain `Configuration`.
	pub fn new<'a>(
		para_config: &sc_service::Configuration,
		relay_chain_args: impl Iterator<Item = &'a String>,
	) -> Self {
		let extension = chain_spec::Extensions::try_get(&*para_config.chain_spec);
		let chain_id = extension.map(|e| e.relay_chain.clone());
		let base_path = para_config.base_path.as_ref().map(|x| x.path().join("polkadot"));
		Self {
			base_path,
			chain_id,
			base: polkadot_cli::RunCmd::parse_from(relay_chain_args),
		}
	}
}
