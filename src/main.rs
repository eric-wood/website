#![feature(string_replace_in_place)]
use axum::{
    Router,
    http::{HeaderValue, header},
    response::Html,
    routing::get,
};
use dotenvy::dotenv;
use minijinja_autoreload::AutoReloader;
use std::{fs, path::Path, sync::Arc};
use tower::ServiceBuilder;
mod app_error;
use app_error::AppError;
mod date_time;
mod db;
mod models;
mod routes;
mod templates;
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
mod blog;
use init_tracing_opentelemetry::TracingConfig;
use models::{Photo, Tag};
use sqlx::SqlitePool;
use templates::load_templates_dyn;
use tower_http::{services::ServeDir, set_header::SetResponseHeaderLayer};
mod config;
use config::Config;
mod views;

use crate::blog::BlogStore;

struct AppState {
    config: Config,
    reloader: AutoReloader,
    photos_db_pool: SqlitePool,
    blog_store: BlogStore,
}

type Response = Result<Html<String>, AppError>;

pub fn bootstrap_cache(config: &Config) -> anyhow::Result<()> {
    let cache_path = Path::new(&config.cache_path);
    fs::create_dir_all(cache_path).expect("failed to create cache directory");
    fs::create_dir_all(cache_path.join("blog"))?;
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenv();

    let config = Config::new()?;
    bootstrap_cache(&config)?;

    let tracing_config = if config.is_prod() {
        TracingConfig::production()
    } else {
        TracingConfig::development()
    };

    let _guard = tracing_config.init_subscriber()?;

    let photos_db_pool = SqlitePool::connect(&config.photos_db_path)
        .await
        .expect("Where's the database???");

    tracing::info!("connected to DB");

    let blog_store = blog::BlogStore::new(&config)?;

    let reloader = load_templates_dyn(&config);
    let app = Router::new()
        .nest_service(
            "/photos/assets",
            ServiceBuilder::new()
                .layer(SetResponseHeaderLayer::overriding(
                    header::CACHE_CONTROL,
                    HeaderValue::from_static("public, max-age=31536000, immutible"),
                ))
                .service(ServeDir::new(&config.assets_path)),
        )
        .nest_service(
            "/photos/thumbnails",
            ServiceBuilder::new()
                .layer(SetResponseHeaderLayer::overriding(
                    header::CACHE_CONTROL,
                    HeaderValue::from_static("public, max-age=31536000, immutible"),
                ))
                .service(ServeDir::new(&config.photos_thumbnail_path)),
        )
        .nest_service(
            "/photos/images",
            ServiceBuilder::new()
                .layer(SetResponseHeaderLayer::overriding(
                    header::CACHE_CONTROL,
                    HeaderValue::from_static("public, max-age=31536000, immutible"),
                ))
                .service(ServeDir::new(&config.photos_image_path)),
        )
        .route("/photos", get(routes::photos::index))
        .route("/photos/{id}", get(routes::photos::show))
        .route("/blog", get(routes::blog::index))
        .route("/blog/{slug}", get(routes::blog::show));

    let app_state = Arc::new(AppState {
        config,
        reloader,
        photos_db_pool,
        blog_store,
    });

    blog::cache_posts(&app_state)?;

    let app = app
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
