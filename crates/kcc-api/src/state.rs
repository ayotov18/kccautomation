use aws_sdk_s3::Client as S3Client;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub s3: S3Client,
    pub s3_bucket: String,
    pub jwt_secret: String,
    /// Shared multiplexed Redis connection, established at startup.
    pub redis: Arc<Mutex<redis::aio::MultiplexedConnection>>,
    /// Whether ODA File Converter is available for DWG→DXF conversion.
    pub dwg_conversion_available: bool,
}
