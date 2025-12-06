use crate::{AppState, Response, app_error::AppError, blog::render_post, templates::render};
use axum::{
    extract::{Path, State},
    response::Html,
};
use minijinja::context;
use std::sync::Arc;

pub async fn show(Path(slug): Path<String>, State(state): State<Arc<AppState>>) -> Response {
    let post = state.blog_slugs.get(&slug).ok_or(AppError::NotFound)?;
    let body = render_post(&post.file_path)?;

    let rendered = render(
        &state.reloader,
        "blog/show",
        context! {
            post,
            body
        },
    )?;

    Ok(Html(rendered))
}
