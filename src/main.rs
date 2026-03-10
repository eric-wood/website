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
mod post;
use init_tracing_opentelemetry::TracingConfig;
use models::{Photo, Tag};
use sqlx::SqlitePool;
use templates::load_templates_dyn;
use tower_http::{services::ServeDir, set_header::SetResponseHeaderLayer};
mod config;
use config::Config;
mod views;

use post::PostStore;

struct AppState {
    config: Config,
    reloader: AutoReloader,
    photos_db_pool: SqlitePool,
    blog_store: PostStore,
    project_store: PostStore,
}

type Response = Result<Html<String>, AppError>;

pub fn bootstrap_cache(config: &Config) -> anyhow::Result<()> {
    let cache_path = Path::new(&config.cache_path);
    fs::create_dir_all(cache_path).expect("failed to create cache directory");
    fs::create_dir_all(cache_path.join("blog"))?;
    Ok(())
}

static ONE_YEAR: usize = 525600;

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

    let cache_path = Path::new(&config.cache_path);
    let blog_path = Path::new(&config.blog_posts_path);
    let blog_cache_path = cache_path.join("blog");
    let blog_store = PostStore::new(blog_path, blog_cache_path.as_path())?;

    let project_path = Path::new(&config.projects_path);
    let project_cache_path = cache_path.join("projects");
    let project_store = PostStore::new(project_path, project_cache_path.as_path())?;

    let reloader = load_templates_dyn(&config);
    let app = Router::new()
        .nest_service(
            "/photos/assets",
            ServiceBuilder::new()
                .layer(cache(ONE_YEAR))
                .service(ServeDir::new(&config.assets_path)),
        )
        .nest_service(
            "/photos/thumbnails",
            ServiceBuilder::new()
                .layer(cache(ONE_YEAR))
                .service(ServeDir::new(&config.photos_thumbnail_path)),
        )
        .nest_service(
            "/photos/images",
            ServiceBuilder::new()
                .layer(cache(ONE_YEAR))
                .service(ServeDir::new(&config.photos_image_path)),
        )
        .nest_service(
            "/assets",
            ServiceBuilder::new()
                .layer(cache(ONE_YEAR))
                .service(ServeDir::new(&config.content_assets_path)),
        )
        .route("/photos", get(routes::photos::index).route_layer(cache(10)))
        .route(
            "/photos/{id}",
            get(routes::photos::show).route_layer(cache(10)),
        )
        .route("/blog", get(routes::blog::index).route_layer(cache(10)))
        .route(
            "/blog/{slug}",
            get(routes::blog::show).route_layer(cache(10)),
        )
        .route(
            "/projects",
            get(routes::projects::index).route_layer(cache(10)),
        )
        .route(
            "/projects/{slug}",
            get(routes::projects::show).route_layer(cache(10)),
        )
        .route("/info", get(routes::info::info))
        .route_layer(cache(1))
        .route("/", get(routes::home::index).route_layer(cache(10)));

    let app_state = Arc::new(AppState {
        config,
        reloader,
        photos_db_pool,
        blog_store,
        project_store,
    });

    post::cache_posts(&app_state)?;

    let app = app
        .with_state(app_state)
        .fallback(handler_404)
        .layer(OtelInResponseLayer)
        .layer(OtelAxumLayer::default());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await?;

    Ok(())
}

fn cache(minutes: usize) -> SetResponseHeaderLayer<HeaderValue> {
    let age = minutes * 60;
    let header = format!("public, max-age={age}, immutible");
    SetResponseHeaderLayer::overriding(
        header::CACHE_CONTROL,
        HeaderValue::from_str(&header).unwrap_or_else(|_| panic!("invalid header: {header}")),
    )
}

async fn handler_404() -> anyhow::Result<Html<String>, app_error::AppError> {
    Err(AppError::NotFound)
}
