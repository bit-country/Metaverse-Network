#[cfg(feature = "with-metaverse-runtime")]
mod rpc_metaverse;
#[cfg(feature = "with-metaverse-runtime")]
pub use rpc_metaverse::{create_full, FullDeps};
#[cfg(feature = "with-pioneer-runtime")]
mod rpc_pioneer;
#[cfg(feature = "with-tewai-runtime")]
pub mod rpc_tewai;
#[cfg(feature = "with-pioneer-runtime")]
pub use rpc_pioneer::{create_full as pioneer_crate_full, FullDeps as pioneer_fulldeps};
