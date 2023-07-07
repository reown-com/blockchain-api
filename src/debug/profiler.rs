use {
    crate::state::AppState,
    aws_config::meta::region::RegionProviderChain,
    aws_sdk_s3::{config::Region, primitives::ByteStream, Client},
    axum::{extract::State, response::IntoResponse, Json},
    hyper::{HeaderMap, StatusCode},
    once_cell::sync::Lazy,
    serde::Deserialize,
    std::{path::PathBuf, sync::Arc, time::Duration},
    tokio::sync::Mutex,
};

static UPLOAD_CONTEXT: Lazy<Mutex<Option<UploadContext>>> = Lazy::new(|| Mutex::new(None));
static PROFILER_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

#[derive(Clone)]
struct UploadContext {
    bucket: String,
    client: Client,
}

#[derive(Deserialize)]
pub struct ProfilerPayload {
    profile_duration: u64,
}

pub async fn init_upload_context(config: &crate::env::Config) {
    let mut lock = UPLOAD_CONTEXT.lock().await;
    *lock = create_upload_context(config).await;

    tracing::warn!("profiler: initialized: {}", lock.is_some());
}

async fn create_upload_context(config: &crate::env::Config) -> Option<UploadContext> {
    if let Some(bucket) = config.analytics.export_bucket.as_deref() {
        tracing::warn!(%bucket, "profiler: initializing profiler...");

        let region_provider = RegionProviderChain::first_try(Region::new("eu-central-1"));
        let aws_config = aws_config::from_env().region(region_provider).load().await;

        let client = Client::new(&aws_config);

        Some(UploadContext {
            client,
            bucket: bucket.to_owned(),
        })
    } else {
        None
    }
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<ProfilerPayload>,
) -> impl IntoResponse {
    let auth = headers.get("Authorization");

    if let Some(auth) = auth {
        if let Ok(auth) = auth.to_str() {
            if auth == state.config.debug.secret {
                tokio::spawn(run_profiler(payload));
                return (StatusCode::OK, "Profiler started.");
            }
        }
    }
    (StatusCode::UNAUTHORIZED, "Unauthorized")
}

async fn run_profiler(payload: ProfilerPayload) {
    let Ok(lock) = PROFILER_LOCK.try_lock() else {
        tracing::warn!("profiler: failed to start because already running");
        return;
    };

    if let Err(err) = profile(Duration::from_secs(payload.profile_duration)).await {
        tracing::warn!(?err, "profiler: profiling failed");
        return;
    }

    // Upload the profiles to S3.
    upload_profiles().await;

    drop(lock);
}

async fn profile(duration: Duration) -> anyhow::Result<()> {
    let profiler = dhat::Profiler::new_heap();
    tracing::warn!("profiler: begin profiling");

    // Let the profiler run for specified duration.
    tokio::time::sleep(duration).await;

    drop(profiler);
    tracing::warn!("profiler: end profiling");

    Ok(())
}

async fn upload_profiles() {
    tracing::warn!("profiler: uploading heap profiles in 30s...");

    // Give some time to write profile data to disk if it's done on a different
    // thread.
    tokio::time::sleep(Duration::from_secs(30)).await;

    let profiles_glob = "./dhat-*.json";

    let file_list = match glob::glob(profiles_glob) {
        Ok(file_list) => file_list,
        Err(e) => {
            tracing::warn!(?e, "profiler: failed to read glob pattern");
            return;
        }
    };

    let ctx = {
        let ctx = UPLOAD_CONTEXT.lock().await;

        let Some(ctx) = ctx.as_ref() else {
            tracing::warn!("profiler: upload context not initialized");
            return;
        };

        ctx.clone()
    };

    tracing::warn!("profiler: uploading heap profiles");

    let timestamp = chrono::Utc::now().timestamp_millis();
    let key_prefix = format!("{timestamp}");

    for path in file_list {
        match path {
            Ok(path) => {
                if let Err(err) = upload_file(ctx.clone(), path.clone(), &key_prefix).await {
                    tracing::warn!(?err, "profiler: failed to upload profile");
                }

                if let Err(err) = tokio::fs::remove_file(path).await {
                    tracing::warn!(?err, "profiler: failed to remove profile after upload")
                }
            }

            Err(err) => {
                tracing::warn!(?err, "profiler: glob error");
            }
        }
    }

    tracing::warn!("profiler: finished uploading profiles");
}

async fn upload_file(ctx: UploadContext, path: PathBuf, key_prefix: &str) -> anyhow::Result<()> {
    let metadata = tokio::fs::metadata(&path).await?;

    tracing::warn!(?path, size = %metadata.len(), "profiler: uploading file");

    let file_name = path
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("failed to extract file name"))?
        .to_string_lossy();

    let key = format!("blockchain-api/heap_profiles/{key_prefix}_{file_name}");

    let stream = ByteStream::from_path(&path)
        .await
        .map_err(|err| anyhow::anyhow!("failed to create bytestream: {err:?}"))?;

    ctx.client
        .put_object()
        .bucket(ctx.bucket.as_str())
        .key(key.as_str())
        .body(stream)
        .send()
        .await
        .map_err(|err| dbg!(anyhow::anyhow!("upload error: {err:?}")))?;

    tracing::warn!(%key, bucket = %ctx.bucket, "profiler: upload finished");

    Ok(())
}
