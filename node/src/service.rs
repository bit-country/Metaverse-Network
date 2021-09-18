//! Service and ServiceFactory implementation. Specialized wrapper over substrate service.

//mod standalone;

// Declare an instance of the native executor named `Executor`. Include the wasm binary as the
// equivalent wasm code.
//native_executor_instance!(
//    pub Executor,
//    metaverse_runtime::api::dispatch,
//    metaverse_runtime::native_version,
//    frame_benchmarking::benchmarking::HostFunctions,
//);

//#[cfg(feature = "with-parachain-runtime")]
//mod service_parachain;
//#[cfg(feature = "with-tewai-runtime")]
//mod service_tewai;
#[cfg(feature = "with-metaverse-runtime")]
mod metaverse;
#[cfg(feature = "with-metaverse-runtime")]
pub use metaverse::{new_full, new_light, new_partial, Executor};
//#[cfg(feature = "with-tewai-runtime")]
//pub use service_tewai::{new_full, new_light, new_partial};
//
//use sc_executor::native_executor_instance;
//
//#[cfg(feature = "with-tewai-runtime")]
//native_executor_instance!(
//    pub MetaverseExecutor,
//    tewai_runtime::api::dispatch,
//    tewai_runtime::native_version,
//    frame_benchmarking::benchmarking::HostFunctions,
//);
//
//#[cfg(feature = "with-metaverse-runtime")]
//native_executor_instance!(
//    pub MetaverseExecutor,
//    metaverse_runtime::api::dispatch,
//    metaverse_runtime::native_version,
//    frame_benchmarking::benchmarking::HostFunctions,
//);
