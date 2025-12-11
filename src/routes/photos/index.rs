use std::sync::Arc;

use axum::{extract::State, response::Html};
use axum_extra::extract::Query;
use serde_valid::Validate;

use crate::{
    AppState, Response,
    db::{self, Pagination as QueryPagination, PhotoQuery, Sort},
    views::{self, View, photos::IndexParams},
};

pub async fn index(query: Query<IndexParams>, State(state): State<Arc<AppState>>) -> Response {
    query.validate()?;
    let query = query.0;
    let default = IndexParams::default();
    let page = query.page.unwrap_or(default.page.unwrap());
    let limit = query.limit.unwrap_or(default.limit.unwrap());
    let tags = query.tags.clone().unwrap_or_default();
    let sort = query.sort.unwrap_or(default.sort.unwrap());
    let dir = query.dir.unwrap_or(default.dir.unwrap());

    let (total_photos, photos) = db::get_photos(
        &state.photos_db_pool,
        PhotoQuery {
            sort: Sort {
                field: sort,
                direction: dir,
            },
            pagination: QueryPagination { limit, page },
            tags: tags.clone(),
        },
    )
    .await?;

    let tags = db::get_tags(&state.photos_db_pool, &query.tags).await?;

    let view = views::photos::PhotosIndex::new(query, photos, tags, total_photos);
    let html = view.render(&state.reloader)?;

    Ok(Html(html))
}
