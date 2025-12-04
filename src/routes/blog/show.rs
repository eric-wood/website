use crate::{AppState, Response, app_error::AppError, templates::render};
use axum::{
    extract::{Path, State},
    response::Html,
};
use minijinja::context;
use std::{fs::read_to_string, sync::Arc};

pub async fn show(Path(slug): Path<String>, State(state): State<Arc<AppState>>) -> Response {
    let path = state.blog_slugs.get(&slug).ok_or(AppError::NotFound)?;
    let md = read_to_string(path)?;
    let body = markdown::to_html(&md);
    let rendered = render(
        &state.reloader,
        "blog/show",
        context! {
            body
        },
    )?;

    Ok(Html(rendered))
}
