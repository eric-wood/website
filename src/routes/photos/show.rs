use crate::{AppError, AppState, db};
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
    let photo = db::get_photo(&state.pool, id).await?;
    let template = state.template_env.get_template("photos/show")?;
    let rendered = template.render(context! {
        photo => photo
    })?;

    Ok(Html(rendered))
}
