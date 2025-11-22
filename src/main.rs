use axum::{Router, response::Html, routing::get};
use dotenvy::dotenv;
use minijinja_autoreload::AutoReloader;
use std::{env, sync::Arc};
mod app_error;
use app_error::AppError;
mod date_time;
mod db;
mod models;
mod routes;
mod templates;
use models::{Photo, Tag};
use sqlx::SqlitePool;
use templates::load_templates_dyn;
use tower_http::services::ServeDir;

struct AppState {
    reloader: AutoReloader,
    pool: SqlitePool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenv();
    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?)
        .await
        .expect("Where's the database???");

    let reloader = load_templates_dyn();
    let app_state = Arc::new(AppState { reloader, pool });
    let app = Router::new()
        .nest_service("/photos/assets", ServeDir::new(&env::var("ASSETS_PATH")?))
        .nest_service(
            "/photos/thumbnails",
            ServeDir::new(&env::var("THUMBNAIL_PATH")?),
        )
        .nest_service("/photos/images", ServeDir::new(&env::var("IMAGE_PATH")?))
        .route("/photos", get(routes::photos::index))
        .route("/photos/{id}", get(routes::photos::show))
        .with_state(app_state)
        .fallback(handler_404);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await?;

    Ok(())
}

async fn handler_404() -> anyhow::Result<Html<String>, app_error::AppError> {
    Err(AppError::NotFound)
}
