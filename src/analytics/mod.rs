use {
    aws_sdk_s3::Client as S3Client,
    std::{net::IpAddr, sync::Arc},
    tap::TapFallible,
    tracing::info,
    wc::{
        analytics::{
            collectors::{batch::BatchOpts, noop::NoopCollector},
            exporters::aws::{AwsExporter, AwsOpts},
            writers::parquet::ParquetWriter,
            Analytics,
        },
        geoip::{self, MaxMindResolver, Resolver},
    },
};
pub use {
    config::Config,
    history_lookup_info::HistoryLookupInfo,
    identity_lookup_info::IdentityLookupInfo,
    message_info::MessageInfo,
};

mod config;
mod history_lookup_info;
mod identity_lookup_info;
mod message_info;

#[derive(Clone)]
pub struct RPCAnalytics {
    messages: Analytics<MessageInfo>,
    identity_lookups: Analytics<IdentityLookupInfo>,
    history_lookups: Analytics<HistoryLookupInfo>,
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
            messages: Analytics::new(NoopCollector),
            identity_lookups: Analytics::new(NoopCollector),
            history_lookups: Analytics::new(NoopCollector),
            geoip_resolver: None,
        }
    }

    fn with_aws_export(
        s3_client: S3Client,
        export_bucket: &str,
        node_ip: IpAddr,
        geoip_resolver: Option<Arc<MaxMindResolver>>,
    ) -> anyhow::Result<Self> {
        info!(%export_bucket, "initializing analytics with aws export");

        let opts = BatchOpts {
            event_queue_limit: 8192,
            ..Default::default()
        };
        let bucket_name: Arc<str> = export_bucket.into();
        let node_ip: Arc<str> = node_ip.to_string().into();

        let messages = {
            let exporter = AwsExporter::new(AwsOpts {
                export_prefix: "blockchain-api/rpc-requests",
                export_name: "rpc_requests",
                file_extension: "parquet",
                bucket_name: bucket_name.clone(),
                s3_client: s3_client.clone(),
                node_ip: node_ip.clone(),
            });

            Analytics::new(ParquetWriter::new(opts.clone(), exporter)?)
        };

        let identity_lookups = {
            let exporter = AwsExporter::new(AwsOpts {
                export_prefix: "blockchain-api/identity-lookups",
                export_name: "identity_lookups",
                file_extension: "parquet",
                bucket_name: bucket_name.clone(),
                s3_client: s3_client.clone(),
                node_ip: node_ip.clone(),
            });

            Analytics::new(ParquetWriter::new(opts.clone(), exporter)?)
        };

        let history_lookups = {
            let exporter = AwsExporter::new(AwsOpts {
                export_prefix: "blockchain-api/history-lookups",
                export_name: "history_lookups",
                file_extension: "parquet",
                bucket_name,
                s3_client,
                node_ip,
            });

            Analytics::new(ParquetWriter::new(opts, exporter)?)
        };

        Ok(Self {
            messages,
            identity_lookups,
            history_lookups,
            geoip_resolver,
        })
    }

    pub fn message(&self, data: MessageInfo) {
        self.messages.collect(data);
    }

    pub fn identity_lookup(&self, data: IdentityLookupInfo) {
        self.identity_lookups.collect(data);
    }

    pub fn history_lookup(&self, data: HistoryLookupInfo) {
        self.history_lookups.collect(data);
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
