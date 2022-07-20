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

use std::{io::Write, net::SocketAddr, sync::Arc};

use codec::Encode;
use cumulus_client_service::genesis::generate_genesis_block;
use cumulus_primitives_core::ParaId;
use frame_benchmarking_cli::{BenchmarkCmd, SUBSTRATE_REFERENCE_HARDWARE};
use log::info;
use sc_cli::{
	ChainSpec, CliConfiguration, DefaultConfigurationValues, ImportParams, KeystoreParams, NetworkParams, Result, Role,
	RuntimeVersion, SharedParams, SubstrateCli,
};
use sc_service::config::{BasePath, PrometheusConfig};
use sc_service::PartialComponents;
use sp_core::hexdisplay::HexDisplay;
use sp_runtime::traits::{AccountIdConversion, Block as BlockT};

#[cfg(feature = "with-continuum-runtime")]
use continuum_runtime::RuntimeApi;
use metaverse_runtime::Block;
#[cfg(feature = "with-pioneer-runtime")]
use pioneer_runtime::RuntimeApi;

#[cfg(feature = "with-continuum-runtime")]
use crate::service::{continuum_partial, ContinuumParachainRuntimeExecutor};
#[cfg(feature = "with-pioneer-runtime")]
use crate::service::{pioneer_partial, ParachainRuntimeExecutor};
use crate::service::{CONTINUUM_RUNTIME_NOT_AVAILABLE, METAVERSE_RUNTIME_NOT_AVAILABLE, PIONEER_RUNTIME_NOT_AVAILABLE};
use crate::{
	chain_spec,
	cli::{Cli, RelayChainCli, Subcommand},
	service,
	service::ExecutorDispatch,
};

fn load_spec(id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
	Ok(match id {
		#[cfg(feature = "with-metaverse-runtime")]
		"dev" => Box::new(chain_spec::metaverse::development_config()?),
		#[cfg(feature = "with-metaverse-runtime")]
		"" | "local" => Box::new(chain_spec::metaverse::local_testnet_config()?),
		#[cfg(feature = "with-metaverse-runtime")]
		"metaverse" => Box::new(chain_spec::metaverse::development_config()?),
		#[cfg(feature = "with-metaverse-runtime")]
		"metaverse-testnet" => Box::new(chain_spec::metaverse::metaverse_testnet_config()?),
		#[cfg(feature = "with-pioneer-runtime")]
		"pioneer-dev" => Box::new(chain_spec::pioneer::development_config()),
		#[cfg(feature = "with-pioneer-runtime")]
		"pioneer-roc" => Box::new(chain_spec::pioneer::roc_pioneer_testnet_config()),
		#[cfg(feature = "with-pioneer-runtime")]
		"pioneer" => Box::new(chain_spec::pioneer::pioneer_network_config_json()?),
		#[cfg(feature = "with-continuum-runtime")]
		"continuum-dev" => Box::new(chain_spec::continuum::development_config()),
		#[cfg(feature = "with-continuum-runtime")]
		"continuum" => Box::new(chain_spec::continuum::continuum_genesis_config()),
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
		2020
	}

	fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
		load_spec(id)
	}

	fn native_runtime_version(spec: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
		if spec.id().starts_with("metaverse") {
			#[cfg(feature = "with-metaverse-runtime")]
			return &metaverse_runtime::VERSION;
			#[cfg(not(feature = "with-metaverse-runtime"))]
			panic!("{}", service::METAVERSE_RUNTIME_NOT_AVAILABLE);
		} else if spec.id().starts_with("pioneer") {
			#[cfg(feature = "with-pioneer-runtime")]
			return &pioneer_runtime::VERSION;
			#[cfg(not(feature = "with-pioneer-runtime"))]
			panic!("{}", service::PIONEER_RUNTIME_NOT_AVAILABLE);
		} else if spec.id().starts_with("continuum") {
			#[cfg(feature = "with-continuum-runtime")]
			return &continuum_runtime::VERSION;
			#[cfg(not(feature = "with-continuum-runtime"))]
			panic!("{}", service::CONTINUUM_RUNTIME_NOT_AVAILABLE);
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

#[allow(clippy::borrowed_box)]
fn extract_genesis_wasm(chain_spec: &Box<dyn sc_service::ChainSpec>) -> Result<Vec<u8>> {
	let mut storage = chain_spec.build_storage()?;

	storage
		.top
		.remove(sp_core::storage::well_known_keys::CODE)
		.ok_or_else(|| "Could not find wasm file in genesis state!".into())
}

macro_rules! construct_async_run {
	(|$components:ident, $cli:ident, $cmd:ident, $config:ident| $( $code:tt )* ) => {{
		let runner = $cli.create_runner($cmd)?;
		runner.async_run(|$config| {
			let $components = pioneer_partial::<
				RuntimeApi,
				ParachainRuntimeExecutor,
				_
			>(
				&$config,
				crate::service::parachain_build_import_queue,
			)?;
			let task_manager = $components.task_manager;
			{ $( $code )* }.map(|v| (v, task_manager))
		})
	}}
}

macro_rules! continuum_construct_async_run {
	(|$components:ident, $cli:ident, $cmd:ident, $config:ident| $( $code:tt )* ) => {{
		let runner = $cli.create_runner($cmd)?;
		runner.async_run(|$config| {
			let $components = continuum_partial::<
				RuntimeApi,
				ContinuumParachainRuntimeExecutor,
				_
			>(
				&$config,
				crate::service::continuum_build_import_queue,
			)?;
			let task_manager = $components.task_manager;
			{ $( $code )* }.map(|v| (v, task_manager))
		})
	}}
}

/// Parse and run command line arguments
pub fn run() -> sc_cli::Result<()> {
	let cli = Cli::from_args();

	match &cli.subcommand {
		Some(Subcommand::BuildSpec(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
		}
		Some(Subcommand::CheckBlock(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			if chain_spec.id().starts_with("pioneer") {
				#[cfg(feature = "with-pioneer-runtime")]
				{
					construct_async_run!(|components, cli, cmd, config| {
						Ok(cmd.run(components.client, components.import_queue))
					})
				}
				#[cfg(not(feature = "with-pioneer-runtime"))]
				Err(PIONEER_RUNTIME_NOT_AVAILABLE.into())
			} else if chain_spec.id().starts_with("continuum") {
				#[cfg(feature = "with-continuum-runtime")]
				{
					continuum_construct_async_run!(|components, cli, cmd, config| {
						Ok(cmd.run(components.client, components.import_queue))
					})
				}
				#[cfg(not(feature = "with-continuum-runtime"))]
				Err(CONTINUUM_RUNTIME_NOT_AVAILABLE.into())
			} else {
				#[cfg(feature = "with-metaverse-runtime")]
				{
					runner.async_run(|config| {
						let PartialComponents {
							client,
							task_manager,
							import_queue,
							..
						} = service::new_partial(&config, &cli)?;
						Ok((cmd.run(client, import_queue), task_manager))
					})
				}
				#[cfg(not(feature = "with-metaverse-runtime"))]
				Err(METAVERSE_RUNTIME_NOT_AVAILABLE.into())
			}
		}
		Some(Subcommand::ExportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			if chain_spec.id().starts_with("pioneer") {
				#[cfg(feature = "with-pioneer-runtime")]
				{
					construct_async_run!(|components, cli, cmd, config| {
						Ok(cmd.run(components.client, config.database))
					})
				}
				#[cfg(not(feature = "with-pioneer-runtime"))]
				Err(PIONEER_RUNTIME_NOT_AVAILABLE.into())
			} else if chain_spec.id().starts_with("continuum") {
				#[cfg(feature = "with-continuum-runtime")]
				{
					continuum_construct_async_run!(|components, cli, cmd, config| {
						Ok(cmd.run(components.client, config.database))
					})
				}
				#[cfg(not(feature = "with-continuum-runtime"))]
				Err(CONTINUUM_RUNTIME_NOT_AVAILABLE.into())
			} else {
				#[cfg(feature = "with-metaverse-runtime")]
				{
					runner.async_run(|config| {
						let PartialComponents {
							client,
							task_manager,
							import_queue: _,
							..
						} = service::new_partial(&config, &cli)?;
						Ok((cmd.run(client, config.database), task_manager))
					})
				}
				#[cfg(not(feature = "with-metaverse-runtime"))]
				Err(METAVERSE_RUNTIME_NOT_AVAILABLE.into())
			}
		}
		Some(Subcommand::ExportState(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			if chain_spec.id().starts_with("pioneer") {
				#[cfg(feature = "with-pioneer-runtime")]
				{
					construct_async_run!(|components, cli, cmd, config| {
						Ok(cmd.run(components.client, config.chain_spec))
					})
				}
				#[cfg(not(feature = "with-pioneer-runtime"))]
				Err(PIONEER_RUNTIME_NOT_AVAILABLE.into())
			} else if chain_spec.id().starts_with("continuum") {
				#[cfg(feature = "with-continuum-runtime")]
				{
					continuum_construct_async_run!(|components, cli, cmd, config| {
						Ok(cmd.run(components.client, config.chain_spec))
					})
				}
				#[cfg(not(feature = "with-continuum-runtime"))]
				Err(CONTINUUM_RUNTIME_NOT_AVAILABLE.into())
			} else {
				#[cfg(feature = "with-metaverse-runtime")]
				{
					runner.async_run(|config| {
						let PartialComponents {
							client,
							task_manager,
							import_queue: _,
							..
						} = service::new_partial(&config, &cli)?;
						Ok((cmd.run(client, config.chain_spec), task_manager))
					})
				}
				#[cfg(not(feature = "with-metaverse-runtime"))]
				Err(METAVERSE_RUNTIME_NOT_AVAILABLE.into())
			}
		}
		Some(Subcommand::ImportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			if chain_spec.id().starts_with("pioneer") {
				#[cfg(feature = "with-pioneer-runtime")]
				{
					construct_async_run!(|components, cli, cmd, config| {
						Ok(cmd.run(components.client, components.import_queue))
					})
				}
				#[cfg(not(feature = "with-pioneer-runtime"))]
				Err(PIONEER_RUNTIME_NOT_AVAILABLE.into())
			} else if chain_spec.id().starts_with("continuum") {
				#[cfg(feature = "with-continuum-runtime")]
				{
					continuum_construct_async_run!(|components, cli, cmd, config| {
						Ok(cmd.run(components.client, components.import_queue))
					})
				}
				#[cfg(not(feature = "with-continuum-runtime"))]
				Err(CONTINUUM_RUNTIME_NOT_AVAILABLE.into())
			} else {
				#[cfg(feature = "with-metaverse-runtime")]
				{
					runner.async_run(|config| {
						let PartialComponents {
							client,
							task_manager,
							import_queue,
							..
						} = service::new_partial(&config, &cli)?;
						Ok((cmd.run(client, import_queue), task_manager))
					})
				}
				#[cfg(not(feature = "with-metaverse-runtime"))]
				Err(METAVERSE_RUNTIME_NOT_AVAILABLE.into())
			}
		}
		Some(Subcommand::PurgeChain(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			if chain_spec.id().starts_with("pioneer") {
				#[cfg(feature = "with-pioneer-runtime")]
				{
					runner.sync_run(|config| {
						let polkadot_cli = RelayChainCli::new(
							&config,
							[RelayChainCli::executable_name()]
								.iter()
								.chain(cli.relaychain_args.iter()),
						);

						let polkadot_config = SubstrateCli::create_configuration(
							&polkadot_cli,
							&polkadot_cli,
							config.tokio_handle.clone(),
						)
						.map_err(|err| format!("Relay chain argument error: {}", err))?;

						cmd.run(config.database)
					})
				}
				#[cfg(not(feature = "with-pioneer-runtime"))]
				Err(PIONEER_RUNTIME_NOT_AVAILABLE.into())
			} else if chain_spec.id().starts_with("continuum") {
				#[cfg(feature = "with-continuum-runtime")]
				{
					runner.sync_run(|config| {
						let polkadot_cli = RelayChainCli::new(
							&config,
							[RelayChainCli::executable_name()]
								.iter()
								.chain(cli.relaychain_args.iter()),
						);

						let polkadot_config = SubstrateCli::create_configuration(
							&polkadot_cli,
							&polkadot_cli,
							config.tokio_handle.clone(),
						)
						.map_err(|err| format!("Relay chain argument error: {}", err))?;

						cmd.run(config.database)
					})
				}
				#[cfg(not(feature = "with-continuum-runtime"))]
				Err(CONTINUUM_RUNTIME_NOT_AVAILABLE.into())
			} else {
				#[cfg(feature = "with-metaverse-runtime")]
				{
					runner.sync_run(|config| cmd.run(config.database))
				}
				#[cfg(not(feature = "with-metaverse-runtime"))]
				Err(METAVERSE_RUNTIME_NOT_AVAILABLE.into())
			}
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
			let chain_spec = &runner.config().chain_spec;

			if chain_spec.id().starts_with("pioneer") {
				#[cfg(feature = "with-pioneer-runtime")]
				{
					construct_async_run!(|components, cli, cmd, config| {
						Ok(cmd.run(components.client, components.backend, None))
					})
				}
				#[cfg(not(feature = "with-pioneer-runtime"))]
				Err(PIONEER_RUNTIME_NOT_AVAILABLE.into())
			} else if chain_spec.id().starts_with("continuum") {
				#[cfg(feature = "with-continuum-runtime")]
				{
					continuum_construct_async_run!(|components, cli, cmd, config| {
						Ok(cmd.run(components.client, components.backend, None))
					})
				}
				#[cfg(not(feature = "with-continuum-runtime"))]
				Err(CONTINUUM_RUNTIME_NOT_AVAILABLE.into())
			} else {
				#[cfg(feature = "with-metaverse-runtime")]
				{
					runner.async_run(|config| {
						let PartialComponents {
							client,
							task_manager,
							backend,
							..
						} = service::new_partial(&config, &cli)?;
						let aux_revert = Box::new(|client, _, blocks| {
							sc_finality_grandpa::revert(client, blocks)?;
							Ok(())
						});
						Ok((cmd.run(client, backend, Some(aux_revert)), task_manager))
					})
				}
				#[cfg(not(feature = "with-metaverse-runtime"))]
				Err(METAVERSE_RUNTIME_NOT_AVAILABLE.into())
			}
		}
		Some(Subcommand::ExportGenesisState(params)) => {
			let mut builder = sc_cli::LoggerBuilder::new("");
			builder.with_profiling(sc_tracing::TracingReceiver::Log, "");
			let _ = builder.init();

			let spec = load_spec(&params.chain.clone().unwrap_or_default())?;
			let state_version = Cli::native_runtime_version(&spec).state_version();
			let block: Block = generate_genesis_block(&spec, state_version)?;
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
			let mut builder = sc_cli::LoggerBuilder::new("");
			builder.with_profiling(sc_tracing::TracingReceiver::Log, "");
			let _ = builder.init();

			let raw_wasm_blob = extract_genesis_wasm(&cli.load_spec(&params.chain.clone().unwrap_or_default())?)?;
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
		Some(Subcommand::Benchmark(cmd)) => {
			let runner = cli.create_runner(cmd)?;

			runner.sync_run(|config| {
				// This switch needs to be in the client, since the client decides
				// which sub-commands it wants to support.
				match cmd {
					BenchmarkCmd::Pallet(cmd) => {
						if !cfg!(feature = "runtime-benchmarks") {
							return Err("Runtime benchmarking wasn't enabled when building the node. \
							You can enable it with `--features runtime-benchmarks`."
								.into());
						}

						cmd.run::<Block, service::ExecutorDispatch>(config)
					}
					BenchmarkCmd::Block(cmd) => {
						let PartialComponents { client, .. } = service::new_partial(&config, &cli)?;
						cmd.run(client)
					}
					BenchmarkCmd::Storage(cmd) => {
						let PartialComponents { client, backend, .. } = service::new_partial(&config, &cli)?;
						let db = backend.expose_db();
						let storage = backend.expose_storage();

						cmd.run(config, client, db, storage)
					}
					BenchmarkCmd::Overhead(_cmd) => Err("Unsupported benchmarking command".into()),
					BenchmarkCmd::Machine(cmd) => cmd.run(&config, SUBSTRATE_REFERENCE_HARDWARE.clone()),
				}
			})
		}

		Some(Subcommand::Key(cmd)) => cmd.run(&cli),

		#[cfg(feature = "try-runtime")]
		Some(Subcommand::TryRuntime(cmd)) => {
			if cfg!(feature = "try-runtime") {
				let runner = cli.create_runner(cmd)?;

				// grab the task manager.
				let registry = &runner.config().prometheus_config.as_ref().map(|cfg| &cfg.registry);
				let task_manager = sc_service::TaskManager::new(runner.config().tokio_handle.clone(), *registry)
					.map_err(|e| format!("Error: {:?}", e))?;

				runner.async_run(|config| Ok((cmd.run::<Block, service::ExecutorDispatch>(config), task_manager)))
			} else {
				Err("Try-runtime must be enabled by `--features try-runtime`.".into())
			}
		}
		None => {
			let runner = cli.create_runner(&cli.run.normalize())?;
			let chain_spec = &runner.config().chain_spec;

			info!("Metaverse Node - Chain_spec id: {}", chain_spec.id());

			#[cfg(feature = "with-pioneer-runtime")]
			if chain_spec.id().starts_with("pioneer") {
				info!("Runtime {}:", chain_spec.id());
				let collator_options = cli.run.collator_options();
				return runner.run_node_until_exit(|config| async move {
					let para_id = chain_spec::Extensions::try_get(&*config.chain_spec)
						.map(|e| e.para_id)
						.ok_or_else(|| "Could not find parachain ID in chain-spec.")?;

					let polkadot_cli = RelayChainCli::new(
						&config,
						[RelayChainCli::executable_name()]
							.iter()
							.chain(cli.relaychain_args.iter()),
					);

					let id = ParaId::from(para_id);

					let parachain_account =
						AccountIdConversion::<polkadot_primitives::v2::AccountId>::into_account_truncating(&id);

					let state_version = RelayChainCli::native_runtime_version(&config.chain_spec).state_version();
					let block: Block =
						generate_genesis_block(&config.chain_spec, state_version).map_err(|e| format!("{:?}", e))?;
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

					crate::service::start_parachain_node(config, polkadot_config, collator_options, id)
						.await
						.map(|r| r.0)
						.map_err(Into::into)
				});
			}
			#[cfg(feature = "with-continuum-runtime")]
			if chain_spec.id().starts_with("continuum") {
				info!("Runtime {}:", chain_spec.id());
				let collator_options = cli.run.collator_options();
				return runner.run_node_until_exit(|config| async move {
					let para_id = chain_spec::Extensions::try_get(&*config.chain_spec)
						.map(|e| e.para_id)
						.ok_or_else(|| "Could not find parachain ID in chain-spec.")?;

					let polkadot_cli = RelayChainCli::new(
						&config,
						[RelayChainCli::executable_name()]
							.iter()
							.chain(cli.relaychain_args.iter()),
					);

					let id = ParaId::from(para_id);

					let parachain_account =
						AccountIdConversion::<polkadot_primitives::v2::AccountId>::into_account_truncating(&id);

					let state_version = RelayChainCli::native_runtime_version(&config.chain_spec).state_version();
					let block: Block =
						generate_genesis_block(&config.chain_spec, state_version).map_err(|e| format!("{:?}", e))?;
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

					crate::service::continuum_start_parachain_node(config, polkadot_config, collator_options, id)
						.await
						.map(|r| r.0)
						.map_err(Into::into)
				});
			}
			#[cfg(feature = "with-metaverse-runtime")]
			info!("Hit metaverse runtime");
			info!("Chain spec: {}", chain_spec.id());
			runner.run_node_until_exit(|config| async move {
				match config.role {
					_ => service::new_full(config, &cli),
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

	fn prometheus_config(
		&self,
		default_listen_port: u16,
		chain_spec: &Box<dyn ChainSpec>,
	) -> Result<Option<PrometheusConfig>> {
		self.base.base.prometheus_config(default_listen_port, chain_spec)
	}

	fn init<F>(
		&self,
		_support_url: &String,
		_impl_version: &String,
		_logger_hook: F,
		_config: &sc_service::Configuration,
	) -> Result<()>
	where
		F: FnOnce(&mut sc_cli::LoggerBuilder, &sc_service::Configuration),
	{
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
