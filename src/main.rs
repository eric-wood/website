use axum::{Router, extract::State, response::Html, routing::get};
use dotenv::dotenv;
use minijinja::{Environment, context};
use std::{env, sync::Arc};
mod app_error;
use app_error::AppError;
mod date_time;
mod db;
mod photos;
mod templates;
use photos::Photo;
use sqlx::SqlitePool;
use templates::load_templates;
use tower_http::services::ServeDir;

struct AppState {
    template_env: Environment<'static>,
    pool: SqlitePool,
}

async fn root(State(state): State<Arc<AppState>>) -> Result<Html<String>, AppError> {
    let photos = db::get_photos(&state.pool).await?;
    let template = state.template_env.get_template("photos/index")?;
    let rendered = template.render(context! {
        photos => photos,
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
        .with_state(app_state)
        .nest_service("/thumbnails", ServeDir::new(&env::var("THUMBNAIL_PATH")?))
        .nest_service("/images", ServeDir::new(&env::var("IMAGE_PATH")?));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await?;

    Ok(())
}
