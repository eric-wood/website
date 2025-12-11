use crate::{
    AppState, Response,
    views::{View, blog::BlogIndex},
};
use axum::{extract::State, response::Html};
use std::sync::Arc;

pub async fn index(State(state): State<Arc<AppState>>) -> Response {
    let slugs: Vec<&String> = state.blog_slugs.keys().collect();
    let view = BlogIndex::new(slugs);
    let html = view.render(&state.reloader)?;

    Ok(Html(html))
}
