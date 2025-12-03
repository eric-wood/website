use crate::{AppState, Response, templates::render};
use axum::{
    extract::{Path, State},
    response::Html,
};
use minijinja::context;
use std::sync::Arc;

pub async fn show(Path(slug): Path<String>, State(state): State<Arc<AppState>>) -> Response {
    let rendered = render(
        &state.reloader,
        "blog/show",
        context! {
            slug
        },
    )?;

    Ok(Html(rendered))
}
