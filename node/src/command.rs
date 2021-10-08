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
	cli::{Cli, Subcommand},
	service,
};
use log::info;
use metaverse_runtime::Block;
use sc_cli::{ChainSpec, Role, RuntimeVersion, SubstrateCli};
use sc_service::PartialComponents;

impl SubstrateCli for Cli {
	fn impl_name() -> String {
		"Bit.Country Metaverse Network Node".into()
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

	fn load_spec(&self, id: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
		Ok(match id {
			"dev" => Box::new(chain_spec::metaverse::development_config()?),
			"" | "local" => Box::new(chain_spec::metaverse::local_testnet_config()?),
			#[cfg(feature = "with-metaverse-runtime")]
			"metaverse" => Box::new(chain_spec::metaverse::metaverse_testnet_config()?),
			#[cfg(feature = "with-tewai-runtime")]
			"tewai" => Box::new(chain_spec::tewai::tewai_testnet_config()?),
			#[cfg(feature = "with-tewai-runtime")]
			"tewai-dev" => Box::new(chain_spec::tewai::development_config()),
			path => Box::new(chain_spec::metaverse::ChainSpec::from_json_file(
				std::path::PathBuf::from(path),
			)?),
		})
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
		} else {
			#[cfg(feature = "with-metaverse-runtime")]
			return &metaverse_runtime::VERSION;
			#[cfg(not(feature = "with-metaverse-runtime"))]
			panic!("{}", service::METAVERSE_RUNTIME_NOT_AVAILABLE);
		}
	}
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
		None => {
			let runner = cli.create_runner(&cli.run)?;
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
			};

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
