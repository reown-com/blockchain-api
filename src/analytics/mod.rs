use anyhow::Context;
use aws_sdk_s3::Client as S3Client;
pub use config::Config;
use gorgon::batcher::{AwsExporter, AwsExporterOpts, BatchCollectorOpts};
use gorgon::geoip::{AnalyticsGeoData, GeoIpReader};
use gorgon::{Analytics, NoopCollector};
pub use message_info::{LegacyMessageInfo, MessageInfo};
use std::net::IpAddr;
use std::sync::Arc;
use tracing::info;

mod config;
mod message_info;

#[derive(Clone)]
pub struct RPCAnalytics {
    messages: Analytics<MessageInfo>,
    legacy_messages: Analytics<LegacyMessageInfo>,
    geoip: GeoIpReader,
}

const AWS_REGION: &str = "eu-central-1";

impl RPCAnalytics {
    pub async fn new(config: &Config, proxy_ip: IpAddr) -> anyhow::Result<Self> {
        if let Some(export_bucket) = config.export_bucket.as_deref() {
            let legacy_export_bucket = config.legacy_export_bucket.as_deref().unwrap();
            let aws_config = aws_config::from_env().region(AWS_REGION).load().await;
            let s3_client = S3Client::new(&aws_config);
            let geoip_params = (&config.geoip_db_bucket, &config.geoip_db_key);

            let geoip = if let (Some(bucket), Some(key)) = geoip_params {
                info!(%bucket, %key, "initializing geoip database from aws s3");

                GeoIpReader::from_aws_s3(&s3_client, bucket, key)
                    .await
                    .context("failed to load geoip database from s3")?
            } else {
                info!("analytics geoip lookup is disabled");

                GeoIpReader::empty()
            };

            Self::with_aws_export(
                s3_client,
                legacy_export_bucket,
                export_bucket,
                proxy_ip,
                geoip,
            )
        } else {
            Ok(Self::with_noop_export())
        }
    }

    pub fn with_noop_export() -> Self {
        info!("initializing analytics with noop export");

        Self {
            messages: Analytics::new(NoopCollector),
            legacy_messages: Analytics::new(NoopCollector),
            geoip: GeoIpReader::empty(),
        }
    }

    pub fn with_aws_export(
        s3_client: S3Client,
        legacy_bucket_name: &str,
        bucket_name: &str,
        node_ip: IpAddr,
        geoip: GeoIpReader,
    ) -> anyhow::Result<Self> {
        info!(%bucket_name, "initializing analytics with aws export");

        let node_ip: Arc<str> = node_ip.to_string().into();

        let legacy_messages = {
            let legacy_opts = BatchCollectorOpts::default();
            let exporter = AwsExporter::new(AwsExporterOpts {
                export_name: "message_parquet",
                file_extension: "parquet",
                bucket_name: legacy_bucket_name.into(),
                s3_client: s3_client.clone(),
                node_ip: node_ip.clone(),
            });

            Analytics::new(gorgon::batcher::create_parquet_collector::<
                LegacyMessageInfo,
                _,
            >(legacy_opts, exporter)?)
        };

        let messages = {
            let opts = BatchCollectorOpts::default();
            let exporter = AwsExporter::new(AwsExporterOpts {
                export_name: "message_parquet",
                file_extension: "parquet",
                bucket_name: bucket_name.into(),
                s3_client,
                node_ip,
            });

            Analytics::new(gorgon::batcher::create_parquet_collector::<MessageInfo, _>(
                opts, exporter,
            )?)
        };

        Ok(Self {
            messages,
            legacy_messages,
            geoip,
        })
    }

    pub fn message(&self, data: MessageInfo) {
        self.legacy_messages.collect(data.clone().into());
        self.messages.collect(data);
    }

    pub fn lookup_geo_data(&self, addr: IpAddr) -> Option<AnalyticsGeoData> {
        self.geoip.lookup_geo_data(addr)
    }
}
