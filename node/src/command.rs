// This file is part of Substrate.

// Copyright (C) 2017-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::{
	chain_spec,
	cli::{Cli, RelayChainCli, Subcommand},
	service,
};

use codec::Encode;
use cumulus_client_service::genesis::generate_genesis_block;
use cumulus_primitives_core::ParaId;
use log::info;
use metaverse_runtime::Block;
use sc_cli::{
	ChainSpec, CliConfiguration, DefaultConfigurationValues, ImportParams, KeystoreParams, NetworkParams, Result, Role,
	RuntimeVersion, SharedParams, SubstrateCli,
};
use sc_service::config::{BasePath, PrometheusConfig};
use sc_service::PartialComponents;
use sp_core::hexdisplay::HexDisplay;
use sp_runtime::traits::{AccountIdConversion, Block as BlockT};
use std::{io::Write, net::SocketAddr};

fn load_spec(id: &str, para_id: ParaId) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
	Ok(match id {
		"dev" => Box::new(chain_spec::metaverse::development_config()?),
		"" | "local" => Box::new(chain_spec::metaverse::local_testnet_config()?),
		#[cfg(feature = "with-metaverse-runtime")]
		"metaverse" => Box::new(chain_spec::metaverse::metaverse_testnet_config()?),
		#[cfg(feature = "with-tewai-runtime")]
		"tewai" => Box::new(chain_spec::tewai::tewai_testnet_config()?),
		#[cfg(feature = "with-tewai-runtime")]
		"tewai-dev" => Box::new(chain_spec::tewai::development_config()),
		"pioneer" => Box::new(chain_spec::pioneer::local_testnet_config(para_id)),
		"pioneer-live" => Box::new(chain_spec::pioneer::pioneer_network_config(para_id)),
		path => Box::new(chain_spec::metaverse::ChainSpec::from_json_file(
			std::path::PathBuf::from(path),
		)?),
	})
}

impl SubstrateCli for Cli {
	fn impl_name() -> String {
		"Metaverse Network Node".into()
	}

	fn impl_version() -> String {
		env!("SUBSTRATE_CLI_IMPL_VERSION").into()
	}

	fn description() -> String {
		env!("CARGO_PKG_DESCRIPTION").into()
	}

	fn author() -> String {
		env!("CARGO_PKG_AUTHORS").into()
	}

	fn support_url() -> String {
		"support.anonymous.an".into()
	}

	fn copyright_start_year() -> i32 {
		2017
	}

	fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
		polkadot_cli::Cli::from_iter([RelayChainCli::executable_name()].iter()).load_spec(id)
	}

	fn native_runtime_version(spec: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
		if spec.id().starts_with("metaverse") {
			#[cfg(feature = "with-metaverse-runtime")]
			return &metaverse_runtime::VERSION;
			#[cfg(not(feature = "with-metaverse-runtime"))]
			panic!("{}", service::METAVERSE_RUNTIME_NOT_AVAILABLE);
		} else if spec.id().starts_with("tewai") {
			#[cfg(feature = "with-tewai-runtime")]
			return &tewai_runtime::VERSION;
			#[cfg(not(feature = "with-tewai-runtime"))]
			panic!("{}", service::TEWAI_RUNTIME_NOT_AVAILABLE);
		} else if spec.id().starts_with("pioneer") {
			#[cfg(feature = "with-pioneer-runtime")]
			return &pioneer_runtime::VERSION;
			#[cfg(not(feature = "with-pioneer-runtime"))]
			panic!("{}", service::PIONEER_RUNTIME_NOT_AVAILABLE);
		} else {
			#[cfg(feature = "with-metaverse-runtime")]
			return &metaverse_runtime::VERSION;
			#[cfg(not(feature = "with-metaverse-runtime"))]
			panic!("{}", service::METAVERSE_RUNTIME_NOT_AVAILABLE);
		}
	}
}

impl SubstrateCli for RelayChainCli {
	fn impl_name() -> String {
		"Metaverse Collator Node".into()
	}

	fn impl_version() -> String {
		env!("SUBSTRATE_CLI_IMPL_VERSION").into()
	}

	fn description() -> String {
		env!("CARGO_PKG_DESCRIPTION").into()
	}

	fn author() -> String {
		env!("CARGO_PKG_AUTHORS").into()
	}

	fn support_url() -> String {
		"support.anonymous.an".into()
	}

	fn copyright_start_year() -> i32 {
		2017
	}

	fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
		polkadot_cli::Cli::from_iter([RelayChainCli::executable_name().to_string()].iter()).load_spec(id)
	}

	fn native_runtime_version(chain_spec: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
		polkadot_cli::Cli::native_runtime_version(chain_spec)
	}
}

fn extract_genesis_wasm(chain_spec: &Box<dyn sc_service::ChainSpec>) -> sc_cli::Result<Vec<u8>> {
	let mut storage = chain_spec.build_storage()?;

	storage
		.top
		.remove(sp_core::storage::well_known_keys::CODE)
		.ok_or_else(|| "Could not find wasm file in genesis state!".into())
}

/// Parse and run command line arguments
pub fn run() -> sc_cli::Result<()> {
	let cli = Cli::from_args();

	match &cli.subcommand {
		Some(Subcommand::Key(cmd)) => cmd.run(&cli),
		Some(Subcommand::BuildSpec(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
		}
		Some(Subcommand::CheckBlock(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let PartialComponents {
					client,
					task_manager,
					import_queue,
					..
				} = service::new_partial(&config)?;
				Ok((cmd.run(client, import_queue), task_manager))
			})
		}
		Some(Subcommand::ExportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let PartialComponents {
					client, task_manager, ..
				} = service::new_partial(&config)?;
				Ok((cmd.run(client, config.database), task_manager))
			})
		}
		Some(Subcommand::ExportState(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let PartialComponents {
					client, task_manager, ..
				} = service::new_partial(&config)?;
				Ok((cmd.run(client, config.chain_spec), task_manager))
			})
		}
		Some(Subcommand::ImportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let PartialComponents {
					client,
					task_manager,
					import_queue,
					..
				} = service::new_partial(&config)?;
				Ok((cmd.run(client, import_queue), task_manager))
			})
		}
		Some(Subcommand::PurgeChain(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run(config.database))
		}
		Some(Subcommand::PurgeChainParachain(cmd)) => {
			let runner = cli.create_runner(cmd)?;

			runner.sync_run(|config| {
				let polkadot_cli = RelayChainCli::new(
					&config,
					[RelayChainCli::executable_name()]
						.iter()
						.chain(cli.relaychain_args.iter()),
				);

				let polkadot_config =
					SubstrateCli::create_configuration(&polkadot_cli, &polkadot_cli, config.tokio_handle.clone())
						.map_err(|err| format!("Relay chain argument error: {}", err))?;

				cmd.run(config, polkadot_config)
			})
		}
		Some(Subcommand::Revert(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let PartialComponents {
					client,
					task_manager,
					backend,
					..
				} = service::new_partial(&config)?;
				Ok((cmd.run(client, backend), task_manager))
			})
		}
		Some(Subcommand::Benchmark(cmd)) => {
			if cfg!(feature = "runtime-benchmarks") {
				let runner = cli.create_runner(cmd)?;

				runner.sync_run(|config| cmd.run::<Block, service::Executor>(config))
			} else {
				Err(
					"Benchmarking wasn't enabled when building the node. You can enable it with \
				     `--features runtime-benchmarks`."
						.into(),
				)
			}
		}
		Some(Subcommand::ExportGenesisState(params)) => {
			info!(
				"ExportGenesisState load_spec: {}",
				&params.chain.clone().unwrap_or_default()
			);

			let mut builder = sc_cli::LoggerBuilder::new("");
			builder.with_profiling(sc_tracing::TracingReceiver::Log, "");
			let _ = builder.init();

			let block: Block = generate_genesis_block(&load_spec(
				&params.chain.clone().unwrap_or("pioneer".into()),
				params.parachain_id.unwrap_or(2096).into(),
			)?)?;
			let raw_header = block.header().encode();
			let output_buf = if params.raw {
				raw_header
			} else {
				format!("0x{:?}", HexDisplay::from(&block.header().encode())).into_bytes()
			};

			if let Some(output) = &params.output {
				std::fs::write(output, output_buf)?;
			} else {
				std::io::stdout().write_all(&output_buf)?;
			}

			Ok(())
		}
		Some(Subcommand::ExportGenesisWasm(params)) => {
			info!(
				"ExportGenesisWasm load_spec: {}",
				&params.chain.clone().unwrap_or_default()
			);

			let mut builder = sc_cli::LoggerBuilder::new("");
			builder.with_profiling(sc_tracing::TracingReceiver::Log, "");
			let _ = builder.init();

			let raw_wasm_blob =
				extract_genesis_wasm(&cli.load_spec(&params.chain.clone().unwrap_or("pioneer-live".into()))?)?;
			let output_buf = if params.raw {
				raw_wasm_blob
			} else {
				format!("0x{:?}", HexDisplay::from(&raw_wasm_blob)).into_bytes()
			};

			if let Some(output) = &params.output {
				std::fs::write(output, output_buf)?;
			} else {
				std::io::stdout().write_all(&output_buf)?;
			}

			Ok(())
		}
		None => {
			let runner = cli.create_runner(&cli.run.normalize())?;
			let chain_spec = &runner.config().chain_spec;

			info!("Metaverse Node - Chain_spec id: {}", chain_spec.id());
			#[cfg(feature = "with-tewai-runtime")]
			if chain_spec.id().starts_with("tewai") {
				return runner.run_node_until_exit(|config| async move {
					match config.role {
						Role::Light => service::tewai_light(config),
						_ => service::tewai_full(config),
					}
					.map_err(sc_cli::Error::Service)
				});
			}

			#[cfg(feature = "with-pioneer-runtime")]
			if chain_spec.id().starts_with("pioneer") {
				return runner.run_node_until_exit(|config| async move {
					let para_id = chain_spec::Extensions::try_get(&*config.chain_spec).map(|e| e.para_id);

					let polkadot_cli = RelayChainCli::new(
						&config,
						[RelayChainCli::executable_name()]
							.iter()
							.chain(cli.relaychain_args.iter()),
					);

					let id = ParaId::from(cli.run.parachain_id.or(para_id).unwrap_or(2096));

					let parachain_account =
						AccountIdConversion::<polkadot_primitives::v0::AccountId>::into_account(&id);

					let block: Block = generate_genesis_block(&config.chain_spec).map_err(|e| format!("{:?}", e))?;
					let genesis_state = format!("0x{:?}", HexDisplay::from(&block.header().encode()));

					let tokio_handle = config.tokio_handle.clone();
					let polkadot_config =
						SubstrateCli::create_configuration(&polkadot_cli, &polkadot_cli, tokio_handle)
							.map_err(|err| format!("Relay chain argument error: {}", err))?;

					info!("Parachain id: {:?}", id);
					info!("Parachain Account: {}", parachain_account);
					info!("Parachain genesis state: {}", genesis_state);
					info!(
						"Is collating: {}",
						if config.role.is_authority() { "yes" } else { "no" }
					);

					crate::service::start_node(config, polkadot_config, id)
						.await
						.map(|r| r.0)
						.map_err(Into::into)
				});
			}

			#[cfg(feature = "with-metaverse-runtime")]
			runner.run_node_until_exit(|config| async move {
				match config.role {
					Role::Light => service::new_light(config),
					_ => service::new_full(config),
				}
				.map_err(sc_cli::Error::Service)
			})
		}
	}
}

impl DefaultConfigurationValues for RelayChainCli {
	fn p2p_listen_port() -> u16 {
		30334
	}

	fn rpc_ws_listen_port() -> u16 {
		9945
	}

	fn rpc_http_listen_port() -> u16 {
		9934
	}

	fn prometheus_listen_port() -> u16 {
		9616
	}
}

impl CliConfiguration<Self> for RelayChainCli {
	fn shared_params(&self) -> &SharedParams {
		self.base.base.shared_params()
	}

	fn import_params(&self) -> Option<&ImportParams> {
		self.base.base.import_params()
	}

	fn network_params(&self) -> Option<&NetworkParams> {
		self.base.base.network_params()
	}

	fn keystore_params(&self) -> Option<&KeystoreParams> {
		self.base.base.keystore_params()
	}

	fn base_path(&self) -> Result<Option<BasePath>> {
		Ok(self
			.shared_params()
			.base_path()
			.or_else(|| self.base_path.clone().map(Into::into)))
	}

	fn rpc_http(&self, default_listen_port: u16) -> Result<Option<SocketAddr>> {
		self.base.base.rpc_http(default_listen_port)
	}

	fn rpc_ipc(&self) -> Result<Option<String>> {
		self.base.base.rpc_ipc()
	}

	fn rpc_ws(&self, default_listen_port: u16) -> Result<Option<SocketAddr>> {
		self.base.base.rpc_ws(default_listen_port)
	}

	fn prometheus_config(&self, default_listen_port: u16) -> Result<Option<PrometheusConfig>> {
		self.base.base.prometheus_config(default_listen_port)
	}

	fn init<C: SubstrateCli>(&self) -> Result<()> {
		unreachable!("PolkadotCli is never initialized; qed");
	}

	fn chain_id(&self, is_dev: bool) -> Result<String> {
		let chain_id = self.base.base.chain_id(is_dev)?;

		Ok(if chain_id.is_empty() {
			self.chain_id.clone().unwrap_or_default()
		} else {
			chain_id
		})
	}

	fn role(&self, is_dev: bool) -> Result<sc_service::Role> {
		self.base.base.role(is_dev)
	}

	fn transaction_pool(&self) -> Result<sc_service::config::TransactionPoolOptions> {
		self.base.base.transaction_pool()
	}

	fn state_cache_child_ratio(&self) -> Result<Option<usize>> {
		self.base.base.state_cache_child_ratio()
	}

	fn rpc_methods(&self) -> Result<sc_service::config::RpcMethods> {
		self.base.base.rpc_methods()
	}

	fn rpc_ws_max_connections(&self) -> Result<Option<usize>> {
		self.base.base.rpc_ws_max_connections()
	}

	fn rpc_cors(&self, is_dev: bool) -> Result<Option<Vec<String>>> {
		self.base.base.rpc_cors(is_dev)
	}

	fn default_heap_pages(&self) -> Result<Option<u64>> {
		self.base.base.default_heap_pages()
	}

	fn force_authoring(&self) -> Result<bool> {
		self.base.base.force_authoring()
	}

	fn disable_grandpa(&self) -> Result<bool> {
		self.base.base.disable_grandpa()
	}

	fn max_runtime_instances(&self) -> Result<Option<usize>> {
		self.base.base.max_runtime_instances()
	}

	fn announce_block(&self) -> Result<bool> {
		self.base.base.announce_block()
	}

	fn telemetry_endpoints(&self, chain_spec: &Box<dyn ChainSpec>) -> Result<Option<sc_telemetry::TelemetryEndpoints>> {
		self.base.base.telemetry_endpoints(chain_spec)
	}
}
