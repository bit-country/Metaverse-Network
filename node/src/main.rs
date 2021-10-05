//! Substrate Node Template CLI library.
#![warn(missing_docs)]

mod chain_spec;
#[macro_use]
mod service;
mod cli;
mod command;
mod rpc;

#[cfg(feature = "with-pioneer-runtime")]
mod para_chain_spec;

fn main() -> sc_cli::Result<()> {
	command::run()
}
