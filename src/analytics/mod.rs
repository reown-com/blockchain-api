pub use {
    account_names_info::AccountNameRegistration,
    balance_lookup_info::BalanceLookupInfo,
    chain_abstraction_info::{
        ChainAbstractionBridgingInfo, ChainAbstractionFundingInfo, ChainAbstractionInitialTxInfo,
    },
    config::Config,
    exchange_event_info::ExchangeEventInfo,
    history_lookup_info::HistoryLookupInfo,
    identity_lookup_info::IdentityLookupInfo,
    message_info::*,
    onramp_history_lookup_info::OnrampHistoryLookupInfo,
};
use {
    aws_sdk_s3::Client as S3Client,
    std::{net::IpAddr, sync::Arc, time::Duration},
    tap::TapFallible,
    tracing::{debug, info},
    wc::{
        analytics::{
            self, AnalyticsExt, ArcCollector, AwsConfig, AwsExporter, BatchCollector,
            BatchObserver, CollectionObserver, Collector, CollectorConfig, ExportObserver,
            ParquetBatchFactory, CollectionError
        },
        geoip::{self, MaxMindResolver, Resolver},
        metrics::{counter, BoolLabel, StringLabel},
    },
};

mod account_names_info;
mod balance_lookup_info;
mod chain_abstraction_info;
mod config;
pub mod exchange_event_info;
mod history_lookup_info;
mod identity_lookup_info;
mod message_info;
mod onramp_history_lookup_info;

const ANALYTICS_EXPORT_TIMEOUT: Duration = Duration::from_secs(30);
const DATA_QUEUE_CAPACITY: usize = 8192;

#[derive(Clone, Copy)]
enum DataKind {
    RpcRequests,
    IdentityLookups,
    HistoryLookups,
    OnrampHistoryLookups,
    BalanceLookups,
    NameRegistrations,
    ChainAbstraction,
    ExchangeEvents,
}

impl DataKind {
    #[inline]
    fn as_str(&self) -> &'static str {
        match self {
            Self::RpcRequests => "rpc_requests",
            Self::IdentityLookups => "identity_lookups",
            Self::HistoryLookups => "history_lookups",
            Self::OnrampHistoryLookups => "onramp_history_lookups",
            Self::BalanceLookups => "balance_lookups",
            Self::NameRegistrations => "name_registrations",
            Self::ChainAbstraction => "chain_abstraction",
            Self::ExchangeEvents => "exchange_events",
        }
    }
}

#[derive(Clone, Copy)]
struct Observer(DataKind);

impl<T, E> BatchObserver<T, E> for Observer
where
    E: std::error::Error,
{
    fn observe_batch_serialization(&self, elapsed: Duration, res: &Result<Vec<u8>, E>) {
        let size = res.as_deref().map(|data| data.len()).unwrap_or(0);
        let elapsed = elapsed.as_millis() as u64;
        counter!("analytics_batches_finished",
            StringLabel<"data_kind", String> => self.0.as_str(),
            BoolLabel<"success"> => res.is_ok()
        )
        .increment(1);

        if let Err(err) = res {
            tracing::warn!(
                ?err,
                data_kind = self.0.as_str(),
                "failed to serialize analytics batch"
            );
        } else {
            tracing::debug!(
                size,
                elapsed,
                data_kind = self.0.as_str(),
                "analytics data batch serialized"
            );
        }
    }
}

impl<T, E> CollectionObserver<T, E> for Observer
where
    E: std::error::Error,
{
    fn observe_collection(&self, res: &Result<(), E>) {
        counter!("analytics_records_collected",
            StringLabel<"data_kind", String> => self.0.as_str(),
            BoolLabel<"success"> => res.is_ok()
        )
        .increment(1);

        if let Err(err) = res {
            tracing::warn!(
                ?err,
                data_kind = self.0.as_str(),
                "failed to collect analytics data"
            );
        }
    }
}

impl<E> ExportObserver<E> for Observer
where
    E: std::error::Error,
{
    fn observe_export(&self, elapsed: Duration, res: &Result<(), E>) {
        counter!("analytics_batches_exported",
            StringLabel<"data_kind", String> => self.0.as_str(),
            BoolLabel<"success"> => res.is_ok()
        )
        .increment(1);

        let elapsed = elapsed.as_millis() as u64;

        if let Err(err) = res {
            tracing::warn!(
                ?err,
                elapsed,
                data_kind = self.0.as_str(),
                "analytics export failed"
            );
        } else {
            tracing::error!(
                elapsed,
                data_kind = self.0.as_str(),
                "analytics export failed"
            );
        }
    }
}

#[derive(Clone)]
pub struct RPCAnalytics {
    messages: ArcCollector<MessageInfo>,
    identity_lookups: ArcCollector<IdentityLookupInfo>,
    history_lookups: ArcCollector<HistoryLookupInfo>,
    onramp_history_lookups: ArcCollector<OnrampHistoryLookupInfo>,
    balance_lookups: ArcCollector<BalanceLookupInfo>,
    name_registrations: ArcCollector<AccountNameRegistration>,

    chain_abstraction_funding: ArcCollector<ChainAbstractionFundingInfo>,
    chain_abstraction_bridging: ArcCollector<ChainAbstractionBridgingInfo>,
    chain_abstraction_initial_tx: ArcCollector<ChainAbstractionInitialTxInfo>,

    exchange_events: ArcCollector<ExchangeEventInfo>,
    geoip_resolver: Option<Arc<MaxMindResolver>>,
}

impl RPCAnalytics {
    pub async fn new(
        config: &Config,
        s3_client: S3Client,
        geoip_resolver: Option<Arc<MaxMindResolver>>,
        api_ip: IpAddr,
    ) -> anyhow::Result<Self> {
        if let Some(export_bucket) = config.export_bucket.as_deref() {
            Self::with_aws_export(s3_client, export_bucket, api_ip, geoip_resolver)
        } else if config.export_bucket.as_deref().is_none() {
            Ok(Self::with_noop_export())
        } else {
            unreachable!()
        }
    }

    fn with_noop_export() -> Self {
        info!("initializing analytics with noop export");

        Self {
            messages: analytics::noop_collector().boxed_shared(),
            identity_lookups: analytics::noop_collector().boxed_shared(),
            history_lookups: analytics::noop_collector().boxed_shared(),
            onramp_history_lookups: analytics::noop_collector().boxed_shared(),
            balance_lookups: analytics::noop_collector().boxed_shared(),
            name_registrations: analytics::noop_collector().boxed_shared(),

            chain_abstraction_funding: analytics::noop_collector().boxed_shared(),
            chain_abstraction_bridging: analytics::noop_collector().boxed_shared(),
            chain_abstraction_initial_tx: analytics::noop_collector().boxed_shared(),

            exchange_events: analytics::noop_collector().boxed_shared(),
            geoip_resolver: None,
        }
    }

    fn with_aws_export(
        s3_client: S3Client,
        export_bucket: &str,
        node_addr: IpAddr,
        geoip_resolver: Option<Arc<MaxMindResolver>>,
    ) -> anyhow::Result<Self> {
        let observer = Observer(DataKind::RpcRequests);
        let messages = BatchCollector::new(
            CollectorConfig {
                data_queue_capacity: DATA_QUEUE_CAPACITY,
                ..Default::default()
            },
            ParquetBatchFactory::new(Default::default()).with_observer(observer),
            AwsExporter::new(AwsConfig {
                export_prefix: "blockchain-api/rpc-requests".to_owned(),
                export_name: "rpc_requests".to_owned(),
                node_addr,
                file_extension: "parquet".to_owned(),
                bucket_name: export_bucket.to_owned(),
                s3_client: s3_client.clone(),
                upload_timeout: ANALYTICS_EXPORT_TIMEOUT,
            })
            .with_observer(observer),
        )
        .with_observer(observer)
        .boxed_shared();

        let observer = Observer(DataKind::IdentityLookups);
        let identity_lookups = BatchCollector::new(
            CollectorConfig {
                data_queue_capacity: DATA_QUEUE_CAPACITY,
                ..Default::default()
            },
            ParquetBatchFactory::new(Default::default()).with_observer(observer),
            AwsExporter::new(AwsConfig {
                export_prefix: "blockchain-api/identity-lookups".to_owned(),
                export_name: "identity_lookups".to_owned(),
                node_addr,
                file_extension: "parquet".to_owned(),
                bucket_name: export_bucket.to_owned(),
                s3_client: s3_client.clone(),
                upload_timeout: ANALYTICS_EXPORT_TIMEOUT,
            })
            .with_observer(observer),
        )
        .with_observer(observer)
        .boxed_shared();

        let observer = Observer(DataKind::HistoryLookups);
        let history_lookups = BatchCollector::new(
            CollectorConfig {
                data_queue_capacity: DATA_QUEUE_CAPACITY,
                ..Default::default()
            },
            ParquetBatchFactory::new(Default::default()).with_observer(observer),
            AwsExporter::new(AwsConfig {
                export_prefix: "blockchain-api/history-lookups".to_owned(),
                export_name: "history_lookups".to_owned(),
                node_addr,
                file_extension: "parquet".to_owned(),
                bucket_name: export_bucket.to_owned(),
                s3_client: s3_client.clone(),
                upload_timeout: ANALYTICS_EXPORT_TIMEOUT,
            })
            .with_observer(observer),
        )
        .with_observer(observer)
        .boxed_shared();

        let observer = Observer(DataKind::OnrampHistoryLookups);
        let onramp_history_lookups = BatchCollector::new(
            CollectorConfig {
                data_queue_capacity: DATA_QUEUE_CAPACITY,
                ..Default::default()
            },
            ParquetBatchFactory::new(Default::default()).with_observer(observer),
            AwsExporter::new(AwsConfig {
                export_prefix: "blockchain-api/onramp-history-lookups".to_owned(),
                export_name: "onramp-history_lookups".to_owned(),
                node_addr,
                file_extension: "parquet".to_owned(),
                bucket_name: export_bucket.to_owned(),
                s3_client: s3_client.clone(),
                upload_timeout: ANALYTICS_EXPORT_TIMEOUT,
            })
            .with_observer(observer),
        )
        .with_observer(observer)
        .boxed_shared();

        let observer = Observer(DataKind::BalanceLookups);
        let balance_lookups = BatchCollector::new(
            CollectorConfig {
                data_queue_capacity: DATA_QUEUE_CAPACITY,
                ..Default::default()
            },
            ParquetBatchFactory::new(Default::default()).with_observer(observer),
            AwsExporter::new(AwsConfig {
                export_prefix: "blockchain-api/balance-lookups".to_owned(),
                export_name: "balance_lookups".to_owned(),
                node_addr,
                file_extension: "parquet".to_owned(),
                bucket_name: export_bucket.to_owned(),
                s3_client: s3_client.clone(),
                upload_timeout: ANALYTICS_EXPORT_TIMEOUT,
            })
            .with_observer(observer),
        )
        .with_observer(observer)
        .boxed_shared();

        let observer = Observer(DataKind::NameRegistrations);
        let name_registrations = BatchCollector::new(
            CollectorConfig {
                data_queue_capacity: DATA_QUEUE_CAPACITY,
                ..Default::default()
            },
            ParquetBatchFactory::new(Default::default()).with_observer(observer),
            AwsExporter::new(AwsConfig {
                export_prefix: "blockchain-api/name-registrations".to_owned(),
                export_name: "name_registrations".to_owned(),
                node_addr,
                file_extension: "parquet".to_owned(),
                bucket_name: export_bucket.to_owned(),
                s3_client: s3_client.clone(),
                upload_timeout: ANALYTICS_EXPORT_TIMEOUT,
            })
            .with_observer(observer),
        )
        .with_observer(observer)
        .boxed_shared();

        let observer = Observer(DataKind::ChainAbstraction);
        let chain_abstraction_bridging = BatchCollector::new(
            CollectorConfig {
                data_queue_capacity: DATA_QUEUE_CAPACITY,
                ..Default::default()
            },
            ParquetBatchFactory::new(Default::default()).with_observer(observer),
            AwsExporter::new(AwsConfig {
                export_prefix: "blockchain-api/chain_abstraction_bridging".to_owned(),
                export_name: "bridging_info".to_owned(),
                node_addr,
                file_extension: "parquet".to_owned(),
                bucket_name: export_bucket.to_owned(),
                s3_client: s3_client.clone(),
                upload_timeout: ANALYTICS_EXPORT_TIMEOUT,
            })
            .with_observer(observer),
        )
        .with_observer(observer)
        .boxed_shared();

        let chain_abstraction_funding = BatchCollector::new(
            CollectorConfig {
                data_queue_capacity: DATA_QUEUE_CAPACITY,
                ..Default::default()
            },
            ParquetBatchFactory::new(Default::default()).with_observer(observer),
            AwsExporter::new(AwsConfig {
                export_prefix: "blockchain-api/chain_abstraction_funding".to_owned(),
                export_name: "funding_info".to_owned(),
                node_addr,
                file_extension: "parquet".to_owned(),
                bucket_name: export_bucket.to_owned(),
                s3_client: s3_client.clone(),
                upload_timeout: ANALYTICS_EXPORT_TIMEOUT,
            })
            .with_observer(observer),
        )
        .with_observer(observer)
        .boxed_shared();

        let chain_abstraction_initial_tx = BatchCollector::new(
            CollectorConfig {
                data_queue_capacity: DATA_QUEUE_CAPACITY,
                ..Default::default()
            },
            ParquetBatchFactory::new(Default::default()).with_observer(observer),
            AwsExporter::new(AwsConfig {
                export_prefix: "blockchain-api/chain_abstraction_initial_tx".to_owned(),
                export_name: "initial_tx".to_owned(),
                node_addr,
                file_extension: "parquet".to_owned(),
                bucket_name: export_bucket.to_owned(),
                s3_client: s3_client.clone(),
                upload_timeout: ANALYTICS_EXPORT_TIMEOUT,
            })
            .with_observer(observer),
        )
        .with_observer(observer)
        .boxed_shared();

        let observer = Observer(DataKind::ExchangeEvents);
        let exchange_events = BatchCollector::new(
            CollectorConfig {
                data_queue_capacity: DATA_QUEUE_CAPACITY,
                ..Default::default()
            },
            ParquetBatchFactory::new(Default::default()).with_observer(observer),
            AwsExporter::new(AwsConfig {
                export_prefix: "blockchain-api/exchange-events".to_owned(),
                export_name: "exchange_events".to_owned(),
                node_addr,
                file_extension: "parquet".to_owned(),
                bucket_name: export_bucket.to_owned(),
                s3_client: s3_client.clone(),
                upload_timeout: ANALYTICS_EXPORT_TIMEOUT,
            })
            .with_observer(observer),
        )
        .with_observer(observer)
        .boxed_shared();

        Ok(Self {
            messages,
            identity_lookups,
            history_lookups,
            onramp_history_lookups,
            balance_lookups,
            name_registrations,

            chain_abstraction_bridging,
            chain_abstraction_funding,
            chain_abstraction_initial_tx,

            exchange_events,
            geoip_resolver,
        })
    }

    pub fn message(&self, data: MessageInfo) {
        if let Err(err) = self.messages.collect(data) {
            tracing::warn!(
                ?err,
                data_kind = DataKind::RpcRequests.as_str(),
                "failed to collect analytics"
            );
        }
    }

    pub fn identity_lookup(&self, data: IdentityLookupInfo) {
        if let Err(err) = self.identity_lookups.collect(data) {
            tracing::warn!(
                ?err,
                data_kind = DataKind::IdentityLookups.as_str(),
                "failed to collect analytics"
            );
        }
    }

    pub fn history_lookup(&self, data: HistoryLookupInfo) {
        if let Err(err) = self.history_lookups.collect(data) {
            tracing::warn!(
                ?err,
                data_kind = DataKind::HistoryLookups.as_str(),
                "failed to collect analytics"
            );
        }
    }

    pub fn onramp_history_lookup(&self, data: OnrampHistoryLookupInfo) {
        if let Err(err) = self.onramp_history_lookups.collect(data) {
            tracing::warn!(
                ?err,
                data_kind = DataKind::OnrampHistoryLookups.as_str(),
                "failed to collect analytics"
            );
        }
    }

    pub fn balance_lookup(&self, data: BalanceLookupInfo) {
        if let Err(err) = self.balance_lookups.collect(data) {
            tracing::warn!(
                ?err,
                data_kind = DataKind::BalanceLookups.as_str(),
                "failed to collect analytics"
            );
        }
    }

    pub fn name_registration(&self, data: AccountNameRegistration) {
        if let Err(err) = self.name_registrations.collect(data) {
            tracing::warn!(
                ?err,
                data_kind = DataKind::NameRegistrations.as_str(),
                "failed to collect analytics"
            );
        }
    }

    pub fn chain_abstraction_funding(&self, data: ChainAbstractionFundingInfo) {
        if let Err(err) = self.chain_abstraction_funding.collect(data) {
            tracing::warn!(
                ?err,
                data_kind = DataKind::ChainAbstraction.as_str(),
                "failed to collect analytics for chain abstraction funding"
            );
        }
    }

    pub fn chain_abstraction_bridging(&self, data: ChainAbstractionBridgingInfo) {
        if let Err(err) = self.chain_abstraction_bridging.collect(data) {
            tracing::warn!(
                ?err,
                data_kind = DataKind::ChainAbstraction.as_str(),
                "failed to collect analytics for chain abstraction bridging"
            );
        }
    }

    pub fn chain_abstraction_initial_tx(&self, data: ChainAbstractionInitialTxInfo) {
        if let Err(err) = self.chain_abstraction_initial_tx.collect(data) {
            tracing::warn!(
                ?err,
                data_kind = DataKind::ChainAbstraction.as_str(),
                "failed to collect analytics for chain abstraction initial tx"
            );
        }
    }

    pub fn geoip_resolver(&self) -> &Option<Arc<MaxMindResolver>> {
        &self.geoip_resolver
    }

    pub fn lookup_geo_data(&self, addr: IpAddr) -> Option<geoip::Data> {
        self.geoip_resolver
            .as_ref()?
            .lookup_geo_data(addr)
            .tap_err(|err| debug!(?err, "failed to lookup geoip data"))
            .ok()
    }

    pub fn exchange_transaction_event(&self, data: ExchangeEventInfo) -> Result<(), CollectionError> {
        self.exchange_events.collect(data)
    }
}
