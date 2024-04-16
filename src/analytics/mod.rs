use {
    aws_sdk_s3::Client as S3Client,
    std::{net::IpAddr, sync::Arc, time::Duration},
    tap::TapFallible,
    tracing::info,
    wc::{
        analytics::{
            self,
            AnalyticsExt,
            ArcCollector,
            AwsConfig,
            AwsExporter,
            BatchCollector,
            BatchObserver,
            CollectionObserver,
            Collector,
            CollectorConfig,
            ExportObserver,
            ParquetBatchFactory,
        },
        geoip::{self, MaxMindResolver, Resolver},
        metrics::otel,
    },
};
pub use {
    balance_lookup_info::BalanceLookupInfo,
    config::Config,
    history_lookup_info::HistoryLookupInfo,
    identity_lookup_info::IdentityLookupInfo,
    message_info::MessageInfo,
    onramp_history_lookup_info::OnrampHistoryLookupInfo,
};

mod balance_lookup_info;
mod config;
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
        }
    }

    #[inline]
    fn as_kv(&self) -> otel::KeyValue {
        otel::KeyValue::new("data_kind", self.as_str())
    }
}

fn success_kv(success: bool) -> otel::KeyValue {
    otel::KeyValue::new("success", success)
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

        wc::metrics::counter!("analytics_batches_finished", 1, &[
            self.0.as_kv(),
            success_kv(res.is_ok())
        ]);

        if let Err(err) = res {
            tracing::warn!(
                ?err,
                data_kind = self.0.as_str(),
                "failed to serialize analytics batch"
            );
        } else {
            tracing::info!(
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
        wc::metrics::counter!("analytics_records_collected", 1, &[
            self.0.as_kv(),
            success_kv(res.is_ok())
        ]);

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
        wc::metrics::counter!("analytics_batches_exported", 1, &[
            self.0.as_kv(),
            success_kv(res.is_ok())
        ]);

        let elapsed = elapsed.as_millis() as u64;

        if let Err(err) = res {
            tracing::warn!(
                ?err,
                elapsed,
                data_kind = self.0.as_str(),
                "analytics export failed"
            );
        } else {
            tracing::info!(
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
                s3_client,
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

    pub fn geoip_resolver(&self) -> &Option<Arc<MaxMindResolver>> {
        &self.geoip_resolver
    }

    pub fn lookup_geo_data(&self, addr: IpAddr) -> Option<geoip::Data> {
        self.geoip_resolver
            .as_ref()?
            .lookup_geo_data(addr)
            .tap_err(|err| info!(?err, "failed to lookup geoip data"))
            .ok()
    }
}
