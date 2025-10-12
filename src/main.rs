use axum::{
    Router,
    extract::{Path, State},
    response::Html,
    routing::get,
};
use dotenv::dotenv;
use minijinja::{Environment, context};
use std::{env, sync::Arc};
mod app_error;
use app_error::AppError;
mod date_time;
mod db;
mod models;
mod templates;
use models::{Photo, Tag};
use sqlx::SqlitePool;
use templates::load_templates;
use tower_http::services::ServeDir;

use crate::db::{Pagination, PhotoQuery, Sort, SortDirection, SortField};

struct AppState {
    template_env: Environment<'static>,
    pool: SqlitePool,
}

async fn root(State(state): State<Arc<AppState>>) -> Result<Html<String>, AppError> {
    let photos = db::get_photos(
        &state.pool,
        PhotoQuery {
            sort: Sort {
                field: SortField::TakenAt,
                direction: SortDirection::Asc,
            },
            pagination: Pagination { limit: 10, page: 1 },
            tag: Some("me".to_string()),
        },
    )
    .await?;

    let tags = db::get_tags(&state.pool).await?;
    let template = state.template_env.get_template("photos/index")?;
    let rendered = template.render(context! {
        photos => photos,
        tags => tags,
    })?;

    Ok(Html(rendered))
}

async fn show(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, AppError> {
    let photo = db::get_photo(&state.pool, id).await?;
    let template = state.template_env.get_template("photos/show")?;
    let rendered = template.render(context! {
        photo => photo
    })?;

    Ok(Html(rendered))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?)
        .await
        .expect("Where's the database???");

    let template_env = load_templates()?;
    let app_state = Arc::new(AppState { template_env, pool });
    let app = Router::new()
        .route("/", get(root))
        .route("/{id}", get(show))
        .with_state(app_state)
        .nest_service("/thumbnails", ServeDir::new(&env::var("THUMBNAIL_PATH")?))
        .nest_service("/images", ServeDir::new(&env::var("IMAGE_PATH")?));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await?;

    Ok(())
}
