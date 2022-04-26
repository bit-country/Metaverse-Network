use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use serde::{Deserialize, Serialize};

#[cfg(feature = "with-metaverse-runtime")]
pub mod metaverse;

#[cfg(feature = "with-metaverse-runtime")]
pub use metaverse::*;

#[cfg(feature = "with-pioneer-runtime")]
pub mod pioneer;

#[cfg(feature = "with-pioneer-runtime")]
pub use pioneer::*;

#[cfg(feature = "with-tewai-runtime")]
pub mod tewai;

#[cfg(feature = "with-tewai-runtime")]
pub use tewai::*;

/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
	/// The relay chain of the Parachain.
	pub relay_chain: String,
	/// The id of the Parachain.
	pub para_id: u32,
}

impl Extensions {
	/// Try to get the extension from the given `ChainSpec`.
	pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
		sc_chain_spec::get_extension(chain_spec.extensions())
	}
}
