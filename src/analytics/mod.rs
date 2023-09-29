use {
    anyhow::Context,
    aws_config::meta::region::RegionProviderChain,
    aws_sdk_s3::{config::Region, Client as S3Client},
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
pub use {config::Config, identity_lookup_info::IdentityLookupInfo, message_info::MessageInfo};

mod config;
mod identity_lookup_info;
mod message_info;

#[derive(Clone)]
pub struct RPCAnalytics {
    messages: Analytics<MessageInfo>,
    identity_lookups: Analytics<IdentityLookupInfo>,
    geoip_resolver: Option<Arc<MaxMindResolver>>,
}

impl RPCAnalytics {
    pub async fn new(config: &Config, api_ip: IpAddr) -> anyhow::Result<Self> {
        match config.export_bucket.as_deref() {
            Some(export_bucket) => {
                let region_provider = RegionProviderChain::first_try(Region::new("eu-central-1"));
                let shared_config = aws_config::from_env().region(region_provider).load().await;

                let aws_config = if let Some(s3_endpoint) = &config.s3_endpoint {
                    info!(%s3_endpoint, "initializing analytics with custom s3 endpoint");

                    aws_sdk_s3::config::Builder::from(&shared_config)
                        .endpoint_url(s3_endpoint)
                        .build()
                } else {
                    aws_sdk_s3::config::Builder::from(&shared_config).build()
                };

                let s3_client = S3Client::from_conf(aws_config);

                let geoip = match (&config.geoip_db_bucket, &config.geoip_db_key) {
                    (Some(bucket), Some(key)) => {
                        info!(%bucket, %key, "initializing geoip database from aws s3");

                        let resolver = MaxMindResolver::from_aws_s3(&s3_client, bucket, key)
                            .await
                            .context("failed to load geoip database from s3")?;

                        Some(resolver)
                    }
                    _ => {
                        info!("analytics geoip lookup is disabled");

                        None
                    }
                };

                Self::with_aws_export(s3_client, export_bucket, api_ip, geoip)
            }
            None => Ok(Self::with_noop_export()),
        }
    }

    fn with_noop_export() -> Self {
        info!("initializing analytics with noop export");

        Self {
            messages: Analytics::new(NoopCollector),
            identity_lookups: Analytics::new(NoopCollector),
            geoip_resolver: None,
        }
    }

    fn with_aws_export(
        s3_client: S3Client,
        export_bucket: &str,
        node_ip: IpAddr,
        geo_resolver: Option<MaxMindResolver>,
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
                bucket_name,
                s3_client,
                node_ip,
            });

            Analytics::new(ParquetWriter::new(opts, exporter)?)
        };

        Ok(Self {
            messages,
            identity_lookups,
            geoip_resolver: geo_resolver.map(Arc::new),
        })
    }

    pub fn message(&self, data: MessageInfo) {
        self.messages.collect(data);
    }

    pub fn identity_lookup(&self, data: IdentityLookupInfo) {
        self.identity_lookups.collect(data);
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
