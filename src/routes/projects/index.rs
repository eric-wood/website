use crate::{
    AppState, Response,
    views::{View, blog::BlogIndex},
};
use axum::{
    extract::{Query, State},
    response::Html,
};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct IndexParams {
    pub tag: Option<String>,
}

pub async fn index(query: Query<IndexParams>, State(state): State<Arc<AppState>>) -> Response {
    let posts = if let Some(tag) = query.tag.clone() {
        state.project_store.get_by_tag(&tag)
    } else {
        state.project_store.all()
    };

    let tags = state.project_store.all_tags();
    let view = BlogIndex::new(posts, tags, query.tag.clone());
    let html = view.render(&state.reloader)?;

    Ok(Html(html))
}
