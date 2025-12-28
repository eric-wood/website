use crate::views::{View, blog::BlogShow};
use crate::{AppState, Response, app_error::AppError};
use axum::{
    extract::{Path, State},
    response::Html,
};
use std::{fs::read_to_string, sync::Arc};

pub async fn show(Path(slug): Path<String>, State(state): State<Arc<AppState>>) -> Response {
    let post = state
        .blog_store
        .get_by_slug(&slug)
        .ok_or(AppError::NotFound)?;

    if state.config.is_prod() {
        let html = read_to_string(&post.cache_path)?;
        return Ok(Html(html));
    }

    let view = BlogShow::new(post);
    let html = view.render(&state.reloader)?;

    Ok(Html(html))
}
