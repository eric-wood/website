use axum::{Router, extract::State, response::Html, routing::get};
use dotenv::dotenv;
use minijinja::{Environment, context};
use std::{env, sync::Arc};
mod app_error;
use app_error::AppError;
mod db;
mod photos;
use photos::Photo;
use sqlx::SqlitePool;
use tower_http::services::ServeDir;

struct AppState {
    template_env: Environment<'static>,
    pool: SqlitePool,
}

async fn root(State(state): State<Arc<AppState>>) -> Result<Html<String>, AppError> {
    let photos = db::get_photos(&state.pool).await?;
    let template = state.template_env.get_template("index")?;
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

    let mut template_env = Environment::new();
    template_env.add_template("layout", include_str!("../templates/layout.jinja"))?;
    template_env.add_template("index", include_str!("../templates/index.jinja"))?;

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
