use crate::{AppState, Response, templates::render};
use axum::{extract::State, response::Html};
use minijinja::context;
use std::sync::Arc;

pub async fn index(State(state): State<Arc<AppState>>) -> Response {
    let slugs: Vec<&String> = state.blog_slugs.keys().collect();
    let rendered = render(
        &state.reloader,
        "blog/index",
        context! {
            slugs
        },
    )?;

    Ok(Html(rendered))
}
