use std::{collections::HashSet, sync::Arc};

use axum::{
    extract::{Path, State},
    response::Html,
};
use axum_extra::extract::Query;
use minijinja::context;
use serde::{self, Deserialize, Serialize};

use crate::{
    AppError, AppState,
    db::{self, Pagination as QueryPagination, PhotoQuery, Sort, SortDirection, SortField},
    models::Tag,
};

#[derive(Serialize)]
struct Pagination {
    page: u32,
    num_pages: u32,
    prev_query: Option<String>,
    next_query: Option<String>,
}

#[derive(Serialize)]
struct SelectedTag {
    tag: String,
    action: String,
}

#[derive(Serialize)]
struct SelectableTag {
    tag: Tag,
    action: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct IndexParams {
    page: Option<u32>,
    limit: Option<u32>,
    tags: Option<Vec<String>>,
    sort: Option<SortField>,
    dir: Option<SortDirection>,
}

impl Default for IndexParams {
    fn default() -> Self {
        Self {
            page: Some(1),
            limit: Some(10),
            tags: Some(vec![]),
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
    let default = IndexParams::default();
    let page = query.page.unwrap_or(default.page.unwrap());
    let limit = query.limit.unwrap_or(default.limit.unwrap());
    let tags = query.tags.clone().unwrap_or_default();
    let sort = query.sort.unwrap_or(default.sort.unwrap());
    let dir = query.dir.unwrap_or(default.dir.unwrap());

    let (total_photos, photos) = db::get_photos(
        &state.pool,
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

    let sort_dir = query.dir;
    let sort_field = query.sort;

    let num_pages = (total_photos as f32 / limit as f32).ceil() as u32;
    let pagination = get_pagination(&query, num_pages)?;

    let all_tags = db::get_tags(&state.pool).await?;
    let (current_tags, tags) = process_tags(&all_tags, &query)?;

    let template = state.template_env.get_template("photos/index")?;
    let rendered = template.render(context! {
        photos => photos,
        tags => tags,
        current_tags => current_tags,
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

fn get_pagination(query: &IndexParams, num_pages: u32) -> anyhow::Result<Pagination> {
    let page = query.page.unwrap_or(1);
    let prev_query = if page > 1 {
        let prev_page = if page == 1 { None } else { Some(page - 1) };
        Some(serde_html_form::to_string(IndexParams {
            page: prev_page,
            tags: query.tags.clone(),
            ..*query
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
        Some(serde_html_form::to_string(IndexParams {
            page: next_page,
            tags: query.tags.clone(),
            ..*query
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

fn process_tags(
    all_tags: &[Tag],
    query: &IndexParams,
) -> anyhow::Result<(Vec<SelectedTag>, Vec<SelectableTag>)> {
    let selected = query.tags.clone().unwrap_or_default();
    let tags = tag_difference(all_tags, &selected);
    let selected_tags: Vec<SelectedTag> = selected
        .iter()
        .map(|tag| {
            let tags_removed = selected.iter().filter(|i| *i != tag).cloned().collect();
            let action = serde_html_form::to_string(IndexParams {
                tags: Some(tags_removed),
                ..*query
            })?;

            Ok(SelectedTag {
                tag: tag.clone(),
                action,
            })
        })
        .collect::<anyhow::Result<Vec<SelectedTag>>>()?;

    let selectable_tags: Vec<SelectableTag> = tags
        .iter()
        .map(|tag| {
            let mut tags_added = selected.clone();
            tags_added.push(tag.name.clone());
            let action = serde_html_form::to_string(IndexParams {
                tags: Some(tags_added),
                ..*query
            })?;

            Ok(SelectableTag {
                tag: tag.clone(),
                action,
            })
        })
        .collect::<anyhow::Result<Vec<SelectableTag>>>()?;

    Ok((selected_tags, selectable_tags))
}

fn tag_difference(tags: &[Tag], selected: &[String]) -> Vec<Tag> {
    let selected: HashSet<String> = HashSet::from_iter(selected.iter().cloned());

    tags.iter()
        .filter(|tag| !selected.contains(&tag.name))
        .cloned()
        .collect()
}
