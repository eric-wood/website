use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    response::Html,
};
use minijinja::context;
use serde::{self, Deserialize, Serialize};

use crate::{
    AppError, AppState,
    db::{self, Pagination as QueryPagination, PhotoQuery, Sort, SortDirection, SortField},
};

#[derive(Serialize)]
pub struct Pagination {
    page: u32,
    num_pages: u32,
    prev_query: Option<String>,
    next_query: Option<String>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct IndexParams {
    page: Option<u32>,
    limit: Option<u32>,
    tag: Option<String>,
    sort: Option<SortField>,
    dir: Option<SortDirection>,
}

impl Default for IndexParams {
    fn default() -> Self {
        Self {
            page: Some(1),
            limit: Some(10),
            tag: None,
            sort: Some(SortField::TakenAt),
            dir: Some(SortDirection::Desc),
        }
    }
}

static SORT_FIELDS: [SortField; 2] = [SortField::TakenAt, SortField::CreatedAt];

pub async fn index(
    query: Query<IndexParams>,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, AppError> {
    let query = query.0;
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(10);
    let tag = query.tag.clone();
    let sort = query.sort.unwrap_or(SortField::TakenAt);
    let dir = query.dir.unwrap_or(SortDirection::Desc);
    let (total_photos, photos) = db::get_photos(
        &state.pool,
        PhotoQuery {
            sort: Sort {
                field: sort,
                direction: dir,
            },
            pagination: QueryPagination { limit, page },
            tag: tag.to_owned(),
        },
    )
    .await?;

    let current_tag = tag.clone();
    let mut tags = db::get_tags(&state.pool).await?;
    if let Some(tag) = tag {
        tags.retain(|t| t.name != tag);
    }

    let sort_dir = query.dir;
    let sort_field = query.sort;
    let template = state.template_env.get_template("photos/index")?;
    let num_pages = total_photos / limit;
    let pagination = get_pagination(query, num_pages)?;
    let rendered = template.render(context! {
        photos => photos,
        tags => tags,
        current_tag => current_tag,
        sort_dir => sort_dir,
        sort_field => sort_field,
        sort_fields => SORT_FIELDS,
        pagination => pagination,
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

fn get_pagination(query: IndexParams, num_pages: u32) -> anyhow::Result<Pagination> {
    let page = query.page.unwrap_or(1);
    let prev_query = if page > 1 {
        let prev_page = if page == 1 { None } else { Some(page - 1) };
        Some(serde_urlencoded::to_string(IndexParams {
            page: prev_page,
            tag: query.tag.clone(),
            ..query
        })?)
    } else {
        None
    };

    let next_query = if page < num_pages {
        let next_page = if page < num_pages {
            Some(page + 1)
        } else {
            None
        };
        Some(serde_urlencoded::to_string(IndexParams {
            page: next_page,
            tag: query.tag.clone(),
            ..query
        })?)
    } else {
        None
    };

    Ok(Pagination {
        page,
        num_pages,
        prev_query,
        next_query,
    })
}
