use std::sync::Arc;

use crate::{
    AppState, Response,
    db::{self, Pagination, PhotoQuery, Sort, SortDirection, SortField},
    views::{View, home::HomeIndex},
};
use axum::{extract::State, response::Html};

pub async fn index(State(state): State<Arc<AppState>>) -> Response {
    let blog_posts = state.blog_store.all();
    let projects = state.project_store.all();

    let (_, photos) = db::get_photos(
        &state.photos_db_pool,
        PhotoQuery {
            sort: Sort {
                field: SortField::TakenAt,
                direction: SortDirection::Desc,
            },
            pagination: Pagination { limit: 10, page: 1 },
            tags: vec![],
        },
    )
    .await?;

    let view = HomeIndex::new(blog_posts, projects, photos);
    let html = view.render(&state.reloader)?;

    Ok(Html(html))
}
