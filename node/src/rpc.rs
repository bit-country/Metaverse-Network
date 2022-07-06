#[cfg(feature = "with-metaverse-runtime")]
pub use rpc_metaverse::{create_full, open_frontier_backend, overrides_handle, FullDeps};
#[cfg(feature = "with-pioneer-runtime")]
pub use rpc_pioneer::{create_full as pioneer_create_full, FullDeps as pioneer_fulldeps};
#[cfg(feature = "with-continuum-runtime")]
pub use rpc_pioneer::{create_full as continuum_create_full, FullDeps as continuum_fulldeps};

#[cfg(feature = "with-continuum-runtime")]
mod rpc_continuum;
#[cfg(feature = "with-metaverse-runtime")]
mod rpc_metaverse;
#[cfg(feature = "with-pioneer-runtime")]
mod rpc_pioneer;
