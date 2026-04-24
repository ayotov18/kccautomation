use axum::{Json, Router, extract::DefaultBodyLimit, middleware as axum_middleware, routing::get};
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

mod error;
mod middleware;
mod routes;
mod state;

use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,kcc_api=debug".into()),
        )
        .init();

    // Database connection pool
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://kcc:kcc_dev_password@localhost:5432/kcc".to_string());
    let db = PgPoolOptions::new()
        .max_connections(20)
        .connect(&database_url)
        .await?;

    // Run migrations
    sqlx::migrate!("../../migrations").run(&db).await?;

    // S3 client — supports MinIO via AWS_ENDPOINT_URL; force_path_style only for local dev
    let endpoint_override = std::env::var("AWS_ENDPOINT_URL").ok();
    let s3_region = std::env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string());
    let mut aws_config_loader =
        aws_config::from_env().region(aws_config::meta::region::RegionProviderChain::first_try(
            aws_sdk_s3::config::Region::new(s3_region),
        ));
    if let Some(ref endpoint) = endpoint_override {
        aws_config_loader = aws_config_loader.endpoint_url(endpoint.as_str());
    }
    let aws_config = aws_config_loader.load().await;

    let s3_config = aws_sdk_s3::config::Builder::from(&aws_config)
        .force_path_style(endpoint_override.is_some()) // only for MinIO/local
        .build();
    let s3 = aws_sdk_s3::Client::from_conf(s3_config);

    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let s3_bucket = std::env::var("S3_BUCKET").unwrap_or_else(|_| "kcc-files-prod".to_string());
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "dev-secret-change-in-production-minimum-32-chars!!".to_string());

    // Redis — connect at startup so misconfiguration fails fast
    let redis_client = redis::Client::open(redis_url.as_str())
        .map_err(|e| anyhow::anyhow!("Invalid Redis URL '{redis_url}': {e}"))?;
    let redis_conn = redis_client
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| anyhow::anyhow!("Redis connection failed at '{redis_url}': {e}"))?;
    tracing::info!("Redis connected at {redis_url}");
    let redis = std::sync::Arc::new(tokio::sync::Mutex::new(redis_conn));

    // Check ODA File Converter availability at startup
    let dwg_conversion_available = kcc_dxf::dwg_converter::DwgConverter::auto_detect().is_ok();
    if dwg_conversion_available {
        tracing::info!("ODA File Converter detected — DWG uploads enabled");
    } else {
        tracing::warn!("ODA File Converter not found — DWG uploads will be rejected. Only DXF files accepted.");
    }

    // S3 storage health check at startup
    let endpoint_info = std::env::var("AWS_ENDPOINT_URL")
        .map(|e| format!("custom endpoint: {e}"))
        .unwrap_or_else(|_| "AWS S3 (default endpoint)".to_string());
    tracing::info!(bucket = %s3_bucket, endpoint = %endpoint_info, "Storage config");

    match s3.head_bucket().bucket(&s3_bucket).send().await {
        Ok(_) => tracing::info!(bucket = %s3_bucket, "Object storage reachable — bucket exists"),
        Err(e) => tracing::error!(
            bucket = %s3_bucket, endpoint = %endpoint_info,
            error = %e, "Object storage NOT reachable — uploads will fail. Check AWS_ENDPOINT_URL, credentials, and bucket name."
        ),
    }

    let state = AppState {
        db,
        s3,
        s3_bucket,
        jwt_secret,
        redis,
        dwg_conversion_available,
    };

    // Routes requiring authentication
    let protected_routes = Router::new()
        .nest(
            "/drawings",
            routes::drawings::drawing_routes()
                .merge(routes::viewer::viewer_token_routes()),
        )
        .nest("/jobs", routes::jobs::job_routes())
        .nest("/reports", routes::reports::report_routes())
        .nest("/features", routes::features::feature_routes())
        .nest("/render", routes::render::render_routes())
        .nest("/config", routes::config::config_routes())
        .nest("/projects", routes::projects::project_routes())
        .nest("/boq", routes::boq::boq_routes())
        .nest("/costs", routes::costs::cost_routes())
        .nest("/assemblies", routes::assemblies::assembly_routes())
        .nest("/schedule", routes::schedule::schedule_routes())
        .nest("/costmodel", routes::costmodel::costmodel_routes())
        .nest("/validation", routes::validation::validation_routes())
        .nest("/tendering", routes::tendering::tendering_routes())
        .nest("/cde", routes::cde::cde_routes())
        .nest("/takeoff", routes::takeoff::takeoff_routes())
        .merge(routes::kss::kss_routes())
        .merge(routes::analyze::analyze_routes())
        .merge(routes::prices::price_routes())
        .merge(routes::quantities::quantity_routes())
        .merge(routes::corrections::correction_routes())
        .route_layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::auth::auth_middleware,
        ));

    // Public routes (auth + HMAC-signed viewer source stream)
    let public_routes = Router::new()
        .nest("/auth", routes::auth::auth_routes())
        .merge(routes::viewer::viewer_source_routes());

    let app = Router::new()
        .nest("/api/v1", public_routes.merge(protected_routes))
        .route("/health", get(health_check))
        .layer(DefaultBodyLimit::max(50 * 1024 * 1024)) // 50MB for DXF/DWG uploads
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let host = std::env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("API_PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("{host}:{port}");

    tracing::info!("KCC API listening on {addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}
