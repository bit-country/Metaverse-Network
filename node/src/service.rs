#[cfg(feature = "with-parachain-runtime")]
mod service_parachain;
#[cfg(feature = "with-tewai-runtime")]
mod service_tewai;
#[cfg(feature = "with-bitcountry-runtime")]
mod service_bitcountry;

#[cfg(feature = "with-bitcountry-runtime")]
pub use service_bitcountry::{new_full, new_light, new_partial};
#[cfg(feature = "with-tewai-runtime")]
pub use service_tewai::{new_full, new_light, new_partial};

use sc_executor::native_executor_instance;

#[cfg(feature = "with-tewai-runtime")]
native_executor_instance!(
    pub BitCountryExecutor,
    tewai_runtime::api::dispatch,
    tewai_runtime::native_version,
    frame_benchmarking::benchmarking::HostFunctions,
);

#[cfg(feature = "with-bitcountry-runtime")]
native_executor_instance!(
    pub BitCountryExecutor,
    bitcountry_runtime::api::dispatch,
    bitcountry_runtime::native_version,
    frame_benchmarking::benchmarking::HostFunctions,
);