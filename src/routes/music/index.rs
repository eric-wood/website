use std::sync::Arc;

use crate::{
    AppState, Response,
    views::{self, View},
};
use axum::{extract::State, response::Html};

pub async fn index(State(state): State<Arc<AppState>>) -> Response {
    let view = views::music::MusicIndex::new();
    let html = view.render(&state.reloader)?;
    Ok(Html(html))
}
