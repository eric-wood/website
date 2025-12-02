use axum::{
    Router,
    http::{HeaderValue, header},
    response::Html,
    routing::get,
};
use dotenvy::dotenv;
use minijinja_autoreload::AutoReloader;
use std::{env, sync::Arc};
use tower::ServiceBuilder;
mod app_error;
use app_error::AppError;
mod date_time;
mod db;
mod models;
mod routes;
mod templates;
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
use init_tracing_opentelemetry::TracingConfig;
use models::{Photo, Tag};
use sqlx::SqlitePool;
use templates::load_templates_dyn;
use tower_http::{services::ServeDir, set_header::SetResponseHeaderLayer};

struct AppState {
    reloader: AutoReloader,
    pool: SqlitePool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenv();
    let environment = env::var("ENVIRONMENT").unwrap_or("development".to_string());
    let tracing_config = if environment == "production" {
        TracingConfig::production()
    } else {
        TracingConfig::development()
    };

    let _guard = tracing_config.init_subscriber()?;

    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?)
        .await
        .expect("Where's the database???");

    tracing::info!("connected to DB");

    let reloader = load_templates_dyn();
    let app_state = Arc::new(AppState { reloader, pool });
    let app = Router::new()
        .nest_service(
            "/photos/assets",
            ServiceBuilder::new()
                .layer(SetResponseHeaderLayer::overriding(
                    header::CACHE_CONTROL,
                    HeaderValue::from_static("public, max-age=31536000, immutible"),
                ))
                .service(ServeDir::new(&env::var("ASSETS_PATH")?)),
        )
        .nest_service(
            "/photos/thumbnails",
            ServiceBuilder::new()
                .layer(SetResponseHeaderLayer::overriding(
                    header::CACHE_CONTROL,
                    HeaderValue::from_static("public, max-age=31536000, immutible"),
                ))
                .service(ServeDir::new(&env::var("THUMBNAIL_PATH")?)),
        )
        .nest_service(
            "/photos/images",
            ServiceBuilder::new()
                .layer(SetResponseHeaderLayer::overriding(
                    header::CACHE_CONTROL,
                    HeaderValue::from_static("public, max-age=31536000, immutible"),
                ))
                .service(ServeDir::new(&env::var("IMAGE_PATH")?)),
        )
        .route("/photos", get(routes::photos::index))
        .route("/photos/{id}", get(routes::photos::show))
        .with_state(app_state)
        .fallback(handler_404)
        .layer(OtelInResponseLayer)
        .layer(OtelAxumLayer::default());
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await?;

    Ok(())
}

async fn handler_404() -> anyhow::Result<Html<String>, app_error::AppError> {
    Err(AppError::NotFound)
}
