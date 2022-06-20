//! Service and ServiceFactory implementation. Specialized wrapper over substrate service.

#[cfg(feature = "with-metaverse-runtime")]
pub use metaverse::{new_full, new_partial, ExecutorDispatch};
#[cfg(feature = "with-pioneer-runtime")]
pub use pioneer::{
	// new_full as pioneer_full, new_light as pioneer_light,
	new_partial as pioneer_partial,
	parachain_build_import_queue,
	start_parachain_node,
	// Executor as pioneer_executor
	ParachainRuntimeExecutor,
};

pub const METAVERSE_RUNTIME_NOT_AVAILABLE: &str =
    "Metaverse runtime is not available. Please compile the node with `--features with-metaverse-runtime` to enable it.";
pub const PIONEER_RUNTIME_NOT_AVAILABLE: &str =
	"Pioneer runtime is not available. Please compile the node with `--features with-pioneer-runtime` to enable it.";

#[cfg(feature = "with-metaverse-runtime")]
pub mod metaverse;
#[cfg(feature = "with-pioneer-runtime")]
pub mod pioneer;
