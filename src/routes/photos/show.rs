use crate::{AppError, AppState, db, templates::render};
use axum::{
    extract::{Path, State},
    response::Html,
};
use minijinja::context;
use std::sync::Arc;

pub async fn show(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, AppError> {
    let photo = db::get_photo(&state.pool, &id).await?;
    let tags = db::get_photo_tags(&state.pool, &id).await?;
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
