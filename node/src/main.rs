//! Substrate Node Template CLI library.
#![warn(missing_docs)]

mod chain_spec;
#[macro_use]
mod service;
mod cli;
//mod command;
mod pioneer_command;
mod rpc;

fn main() -> sc_cli::Result<()> {
	pioneer_command::run()
}
