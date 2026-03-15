use crate::{AppState, Response, views::View, views::resume};
use axum::{extract::State, response::Html};
use std::{fs::read_to_string, sync::Arc};

pub async fn resume(State(state): State<Arc<AppState>>) -> Response {
    let data_string = read_to_string(state.config.resume_path.clone())?;
    let data = serde_yaml::from_str(&data_string)?;
    let view = resume::Resume::new(data);
    let html = view.render(&state.reloader)?;
    Ok(Html(html))
}
