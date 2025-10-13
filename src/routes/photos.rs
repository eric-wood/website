use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    response::Html,
};
use minijinja::context;
use serde::{self, Deserialize};

use crate::{
    AppError, AppState,
    db::{self, Pagination, PhotoQuery, Sort, SortDirection, SortField},
};

#[derive(Deserialize)]
#[serde(default)]
pub struct IndexParams {
    page: u32,
    limit: u32,
    tag: Option<String>,
    sort: SortField,
    dir: SortDirection,
}

impl Default for IndexParams {
    fn default() -> Self {
        Self {
            page: 1,
            limit: 10,
            tag: None,
            sort: SortField::TakenAt,
            dir: SortDirection::Desc,
        }
    }
}

static SORT_FIELDS: [SortField; 2] = [SortField::TakenAt, SortField::CreatedAt];

pub async fn index(
    query: Query<IndexParams>,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, AppError> {
    let photos = db::get_photos(
        &state.pool,
        PhotoQuery {
            sort: Sort {
                field: query.sort,
                direction: query.dir,
            },
            pagination: Pagination {
                limit: query.limit,
                page: query.page,
            },
            tag: query.tag.to_owned(),
        },
    )
    .await?;

    let current_tag = query.tag.clone();
    let tags = db::get_tags(&state.pool).await?;
    let sort_dir = query.dir;
    let sort_field = query.sort;
    let template = state.template_env.get_template("photos/index")?;
    let rendered = template.render(context! {
        photos => photos,
        tags => tags,
        current_tag => current_tag,
        sort_dir => sort_dir,
        sort_field => sort_field,
        sort_fields => SORT_FIELDS,
    })?;

    Ok(Html(rendered))
}

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
