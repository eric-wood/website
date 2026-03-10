use crate::{AppState, Response, views::View, views::info};
use axum::{extract::State, response::Html};
use std::sync::Arc;

pub async fn info(State(state): State<Arc<AppState>>) -> Response {
    let view = info::Info::new();
    let html = view.render(&state.reloader)?;
    Ok(Html(html))
}
