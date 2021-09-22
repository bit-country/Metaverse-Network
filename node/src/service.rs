//! Service and ServiceFactory implementation. Specialized wrapper over substrate service.

//#[cfg(feature = "with-parachain-runtime")]
//mod service_parachain;
#[cfg(feature = "with-metaverse-runtime")]
mod metaverse;
#[cfg(feature = "with-tewai-runtime")]
mod tewai;
#[cfg(feature = "with-metaverse-runtime")]
pub use metaverse::{new_full, new_light, new_partial, Executor};
#[cfg(feature = "with-tewai-runtime")]
pub use tewai::{new_full as tewai_full, new_light as tewai_light, new_partial as tewai_partial};
