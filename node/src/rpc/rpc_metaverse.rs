//! RPCs implementation.
use fc_rpc::{
	EthBlockDataCacheTask, HexEncodedIdProvider, OverrideHandle, RuntimeApiStorageOverride, SchemaV1Override,
	SchemaV2Override, SchemaV3Override, StorageOverride,
};
use fc_rpc_core::types::{FeeHistoryCache, FeeHistoryCacheLimit, FilterPool};
use fp_storage::EthereumStorageSchema;
use jsonrpc_pubsub::manager::SubscriptionManager;
use jsonrpsee::RpcModule;
use metaverse_runtime::{opaque::Block, AccountId, Balance, Hash, Index};
use pallet_contracts_rpc::ContractsRpc;
use pallet_transaction_payment_rpc::TransactionPaymentRpc;
use sc_cli::SubstrateCli;
use std::collections::BTreeMap;
use std::sync::Arc;
// Substrate
use sc_client_api::{
	backend::{AuxStore, Backend, StateBackend, StorageProvider},
	client::BlockchainEvents,
};
use sc_network::NetworkService;
pub use sc_rpc::{DenyUnsafe, SubscriptionTaskExecutor};
use sc_transaction_pool::{ChainApi, Pool};
use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Backend as BlockchainBackend, Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_runtime::traits::BlakeTwo256;
use substrate_frame_rpc_system::SystemRpc;

use primitives::*;

pub fn open_frontier_backend(config: &sc_service::Configuration) -> Result<Arc<fc_db::Backend<Block>>, String> {
	let config_dir = config
		.base_path
		.as_ref()
		.map(|base_path| base_path.config_dir(config.chain_spec.id()))
		.unwrap_or_else(|| {
			sc_service::BasePath::from_project("", "", "metaverse-node").config_dir(config.chain_spec.id())
		});
	let path = config_dir.join("frontier").join("db");

	Ok(Arc::new(fc_db::Backend::<Block>::new(&fc_db::DatabaseSettings {
		source: fc_db::DatabaseSettingsSrc::RocksDb { path, cache_size: 0 },
	})?))
}

pub fn overrides_handle<C, BE>(client: Arc<C>) -> Arc<OverrideHandle<Block>>
where
	C: ProvideRuntimeApi<Block> + StorageProvider<Block, BE> + AuxStore,
	C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError>,
	C: Send + Sync + 'static,
	C::Api: fp_rpc::EthereumRuntimeRPCApi<Block>,
	BE: Backend<Block> + 'static,
	BE::State: StateBackend<BlakeTwo256>,
{
	let mut overrides_map = BTreeMap::new();
	overrides_map.insert(
		EthereumStorageSchema::V1,
		Box::new(SchemaV1Override::new(client.clone())) as Box<dyn StorageOverride<_> + Send + Sync>,
	);
	overrides_map.insert(
		EthereumStorageSchema::V2,
		Box::new(SchemaV2Override::new(client.clone())) as Box<dyn StorageOverride<_> + Send + Sync>,
	);
	overrides_map.insert(
		EthereumStorageSchema::V3,
		Box::new(SchemaV3Override::new(client.clone())) as Box<dyn StorageOverride<_> + Send + Sync>,
	);

	Arc::new(OverrideHandle {
		schemas: overrides_map,
		fallback: Box::new(RuntimeApiStorageOverride::new(client.clone())),
	})
}

/// A type representing all RPC extensions.
pub type RpcExtension = jsonrpsee::RpcModule<()>;

/// Full client dependencies.
pub struct FullDeps<C, P, A: ChainApi> {
	/// The client instance to use.
	pub client: Arc<C>,
	/// Transaction pool instance.
	pub pool: Arc<P>,
	/// Graph pool instance.
	pub graph: Arc<Pool<A>>,
	/// Whether to deny unsafe calls
	pub deny_unsafe: DenyUnsafe,
	/// The Node authority flag
	pub is_authority: bool,
	/// Whether to enable dev signer
	pub enable_dev_signer: bool,
	/// Network service
	pub network: Arc<NetworkService<Block, Hash>>,
	/// EthFilterApi pool.
	pub filter_pool: Option<FilterPool>,
	/// Backend.
	pub backend: Arc<fc_db::Backend<Block>>,
	/// Maximum number of logs in a query.
	pub max_past_logs: u32,
	/// Fee history cache.
	pub fee_history_cache: FeeHistoryCache,
	/// Maximum fee history cache size.
	pub fee_history_cache_limit: FeeHistoryCacheLimit,
	/// Ethereum data access overrides.
	pub overrides: Arc<OverrideHandle<Block>>,
	/// Cache for Ethereum block data.
	pub block_data_cache: Arc<EthBlockDataCacheTask<Block>>,
	/// Manual seal command sink
	#[cfg(feature = "manual-seal")]
	pub command_sink: Option<futures::channel::mpsc::Sender<sc_consensus_manual_seal::rpc::EngineCommand<Hash>>>,
}

/// Instantiate all Full RPC extensions.
pub fn create_full<C, P, BE, A>(
	deps: FullDeps<C, P, A>,
	subscription_task_executor: SubscriptionTaskExecutor,
) -> Result<RpcModule<()>, Box<dyn std::error::Error + Send + Sync>>
where
	BE: Backend<Block> + 'static,
	BE::State: StateBackend<BlakeTwo256>,
	C: ProvideRuntimeApi<Block> + StorageProvider<Block, BE> + AuxStore,
	C: BlockchainEvents<Block>,
	C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError>,
	C: Send + Sync + 'static,
	C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>,
	C::Api: BlockBuilder<Block>,
	C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
	C::Api: fp_rpc::ConvertTransactionRuntimeApi<Block>,
	C::Api: fp_rpc::EthereumRuntimeRPCApi<Block>,
	P: TransactionPool<Block = Block> + 'static,
	A: ChainApi<Block = Block> + 'static,
{
	use fc_rpc::{
		Eth, EthApiServer, EthDevSigner, EthFilter, EthFilterApiServer, EthPubSub, EthPubSubApiServer, EthSigner, Net,
		NetApiServer, Web3, Web3ApiServer,
	};
	use pallet_transaction_payment_rpc::{TransactionPaymentApiServer, TransactionPaymentRpc};
	use substrate_frame_rpc_system::{SystemApiServer, SystemRpc};

	let mut io = RpcModule::new(());
	let FullDeps {
		client,
		pool,
		graph,
		deny_unsafe,
		is_authority,
		enable_dev_signer,
		network,
		filter_pool,
		backend,
		max_past_logs,
		fee_history_cache,
		fee_history_cache_limit,
		overrides,
		block_data_cache,
		#[cfg(feature = "manual-seal")]
		command_sink,
	} = deps;

	io.merge(SystemRpc::new(Arc::clone(&client), Arc::clone(&pool), deny_unsafe).into_rpc())?;
	io.merge(TransactionPaymentRpc::new(Arc::clone(&client)).into_rpc())?;

	let mut signers = Vec::new();
	if enable_dev_signer {
		signers.push(Box::new(EthDevSigner::new()) as Box<dyn EthSigner>);
	}

	io.merge(
		Eth::new(
			Arc::clone(&client),
			Arc::clone(&pool),
			graph,
			Some(metaverse_runtime::TransactionConverter),
			Arc::clone(&network),
			signers,
			Arc::clone(&overrides),
			Arc::clone(&backend),
			// Is authority.
			is_authority,
			Arc::clone(&block_data_cache),
			fee_history_cache,
			fee_history_cache_limit,
		)
		.into_rpc(),
	)?;

	if let Some(filter_pool) = filter_pool {
		io.merge(
			EthFilter::new(
				client.clone(),
				backend,
				filter_pool,
				500_usize, // max stored filters
				max_past_logs,
				block_data_cache,
			)
			.into_rpc(),
		)?;
	}

	io.merge(
		Net::new(
			client.clone(),
			network.clone(),
			// Whether to format the `peer_count` response as Hex (default) or not.
			true,
		)
		.into_rpc(),
	)?;

	io.merge(Web3::new(client.clone()).into_rpc())?;

	io.merge(EthPubSub::new(pool, client, network, subscription_task_executor, overrides).into_rpc())?;

	#[cfg(feature = "manual-seal")]
	if let Some(command_sink) = command_sink {
		io.merge(
			// We provide the rpc handler with the sending end of the channel to allow the rpc
			// send EngineCommands to the background block authorship task.
			ManualSeal::new(command_sink).into_rpc(),
		)?;
	}

	Ok(io)
}
/*
/// Instantiate all RPC extensions.
pub fn create_full<C, P, BE, A>(
	deps: FullDeps<C, P, A>,
	subscription_task_executor: SubscriptionTaskExecutor,
	overrides: Arc<OverrideHandle<Block>>,
) -> Result<RpcExtension, Box<dyn std::error::Error + Send + Sync>>
where
	C: ProvideRuntimeApi<Block>
		+ HeaderBackend<Block>
		+ AuxStore
		+ StorageProvider<Block, BE>
		+ HeaderMetadata<Block, Error = BlockChainError>
		+ BlockchainEvents<Block>
		+ Send
		+ Sync
		+ 'static,
	C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>
		+ pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>
		+ fp_rpc::ConvertTransactionRuntimeApi<Block>
		+ fp_rpc::EthereumRuntimeRPCApi<Block>
		+ BlockBuilder<Block>,
	P: TransactionPool<Block = Block> + Sync + Send + 'static,
	BE: Backend<Block> + 'static,
	BE::State: StateBackend<BlakeTwo256>,
	BE::Blockchain: BlockchainBackend<Block>,
	A: ChainApi<Block = Block> + 'static,
{
	use fc_rpc::{
		Eth, EthApiServer, EthDevSigner, EthFilter, EthFilterApiServer, EthPubSub,
		EthPubSubApiServer, EthSigner, Net, NetApiServer, Web3, Web3ApiServer,
	};
	use pallet_transaction_payment_rpc::{TransactionPaymentApiServer, TransactionPaymentRpc};
	use substrate_frame_rpc_system::{SystemApiServer, SystemRpc};

	//let mut io = jsonrpc_core::IoHandler::default();
	let mut module = RpcModule::new(());
	let FullDeps {
		client,
		pool,
		graph,
		deny_unsafe,
		is_authority,
		enable_dev_signer,
		network,
		filter_pool,
		backend,
		max_past_logs,
		fee_history_cache,
		fee_history_cache_limit,
		overrides,
		block_data_cache,
		#[cfg(feature = "manual-seal")]
		command_sink,
	} = deps;

	module.merge(SystemRpc::new(client.clone(), pool.clone(), deny_unsafe));
	module.merge(TransactionPaymentRpc::new(client.clone()));

	let max_past_logs: u32 = 10_000;
	let max_stored_filters: usize = 500;
	let block_data_cache = Arc::new(EthBlockDataCacheTask::new(50, 50));

	module.merge(EthApiServer::into_rpc(EthApi::new(
		client.clone(),
		pool.clone(),
		graph,
		network.clone(),
		Default::default(),
		overrides.clone(),
		frontier_backend.clone(),
		is_authority,
		max_past_logs,
		block_data_cache.clone(),
		fp_rpc::format::Geth,
		fee_history_limit,
		fee_history_cache,
	)));

	module.merge(EthFilterApiServer::into_rpc(EthFilterApi::new(
		client.clone(),
		frontier_backend,
		filter_pool,
		max_stored_filters,
		overrides.clone(),
		max_past_logs,
		block_data_cache.clone(),
	)));

	module.merge(
		Net::new(
			client.clone(),
			network.clone(),
			// Whether to format the `peer_count` response as Hex (default) or not.
			true,
		)
		.into_rpc(),
	)?;

	module.merge(Web3::new(client.clone()).into_rpc())?;

	module.merge(
		EthPubSub::new(pool, client, network, subscription_task_executor, overrides).into_rpc(),
	)?;

	// Contracts RPC API extension
	//	module.merge(ContractsRpc::new(client.clone()));

	Ok(module)
}
*/
