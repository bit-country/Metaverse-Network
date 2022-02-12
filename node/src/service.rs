//! Service and ServiceFactory implementation. Specialized wrapper over substrate service.

#[cfg(feature = "with-metaverse-runtime")]
pub use metaverse::{new_partial, start_node, ExecutorDispatch as Executor};
#[cfg(feature = "with-pioneer-runtime")]
pub use pioneer::{
	// new_full as pioneer_full, new_light as pioneer_light,
	new_partial as pioneer_partial,
	parachain_build_import_queue,
	start_parachain_node,
	// Executor as pioneer_executor
	ParachainRuntimeExecutor,
};
#[cfg(feature = "with-tewai-runtime")]
pub use tewai::{new_full as tewai_full, new_light as tewai_light, new_partial as tewai_partial};

pub const METAVERSE_RUNTIME_NOT_AVAILABLE: &str =
    "Metaverse runtime is not available. Please compile the node with `--features with-metaverse-runtime` to enable it.";
pub const TEWAI_RUNTIME_NOT_AVAILABLE: &str =
	"Tewai runtime is not available. Please compile the node with `--features with-tewai-runtime` to enable it.";
pub const PIONEER_RUNTIME_NOT_AVAILABLE: &str =
	"Pioneer runtime is not available. Please compile the node with `--features with-pioneer-runtime` to enable it.";

//#[cfg(feature = "with-parachain-runtime")]
//mod service_parachain;
#[cfg(feature = "with-metaverse-runtime")]
mod metaverse;
#[cfg(feature = "with-pioneer-runtime")]
mod pioneer;
#[cfg(feature = "with-tewai-runtime")]
mod tewai;
