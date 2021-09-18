#[cfg(feature = "with-metaverse-runtime")]
mod rpc_metaverse;
#[cfg(feature = "with-metaverse-runtime")]
pub use rpc_metaverse::{create_full, FullDeps};
#[cfg(feature = "with-tewai-runtime")]
pub mod rpc_tewai;
