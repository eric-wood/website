use crate::{
    AppError, AppState, Response, db,
    views::{View, photos::PhotosShow},
};
use axum::{
    extract::{Path, State},
    response::Html,
};
use std::sync::Arc;

pub async fn show(Path(id): Path<String>, State(state): State<Arc<AppState>>) -> Response {
    let photo = db::get_photo(&state.photos_db_pool, &id)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AppError::NotFound,
            _ => AppError::DbError(e),
        })?;

    let tags = db::get_photo_tags(&state.photos_db_pool, &id).await?;

    let view = PhotosShow::new(photo, tags);
    let html = view.render(&state.reloader)?;

    Ok(Html(html))
}
