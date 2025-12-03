use crate::{AppError, AppState, Response, db, templates::render};
use axum::{
    extract::{Path, State},
    response::Html,
};
use minijinja::context;
use std::sync::Arc;

pub async fn show(Path(id): Path<String>, State(state): State<Arc<AppState>>) -> Response {
    let photo = db::get_photo(&state.photos_db_pool, &id)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AppError::NotFound,
            _ => AppError::DbError(e),
        })?;

    let tags = db::get_photo_tags(&state.photos_db_pool, &id).await?;
    let aperture = format!("{:.1}", photo.aperture);
    let focal_length = format!("{:.0}", photo.focal_length);
    let shutter_speed = format!("1/{:.0}s", 1.0 / photo.shutter_speed);

    let rendered = render(
        &state.reloader,
        "photos/show",
        context! {
            aperture,
            focal_length,
            shutter_speed,
            photo,
            tags,
        },
    )?;

    Ok(Html(rendered))
}
