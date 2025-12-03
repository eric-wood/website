use std::{collections::HashSet, sync::Arc};

use axum::{extract::State, response::Html};
use axum_extra::extract::Query;
use minijinja::context;
use serde::{self, Deserialize, Serialize};
use serde_valid::Validate;

use crate::{
    AppState, Response,
    db::{self, Pagination as QueryPagination, PhotoQuery, Sort, SortDirection, SortField},
    models::Tag,
    templates::render,
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
    name: String,
    action: String,
}

#[derive(Serialize)]
struct SelectableTag {
    tag: Tag,
    action: String,
}

#[derive(Deserialize, Serialize, Clone, Validate)]
pub struct IndexParams {
    #[validate(minimum = 1)]
    page: Option<u32>,

    #[validate(minimum = 1)]
    #[validate(maximum = 100)]
    limit: Option<u32>,

    tags: Option<Vec<String>>,
    sort: Option<SortField>,
    dir: Option<SortDirection>,
}

impl Default for IndexParams {
    fn default() -> Self {
        Self {
            page: Some(1),
            limit: Some(30),
            tags: Some(vec![]),
            sort: Some(SortField::TakenAt),
            dir: Some(SortDirection::Desc),
        }
    }
}

static SORT_FIELDS: [SortField; 2] = [SortField::TakenAt, SortField::CreatedAt];

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

    let sort_dir = query.dir;
    let sort_field = query.sort;
    let sort_link = sort_link(&query, dir)?;

    let num_pages = (total_photos as f32 / limit as f32).ceil() as u32;
    let pagination = get_pagination(&query, num_pages)?;

    let all_tags = db::get_tags(&state.photos_db_pool, &query.tags).await?;
    let (current_tags, tags) = process_tags(&all_tags, &query)?;

    let rendered = render(
        &state.reloader,
        "photos/index",
        context! {
            photos,
            tags,
            current_tags,
            sort_dir,
            sort_field,
            pagination,
            sort_fields => SORT_FIELDS,
            sort_link,
        },
    )?;

    Ok(Html(rendered))
}

fn process_tags(
    all_tags: &[Tag],
    query: &IndexParams,
) -> anyhow::Result<(Vec<SelectedTag>, Vec<SelectableTag>)> {
    let selected = query.tags.clone().unwrap_or_default();
    let tags = tag_difference(all_tags, &selected);
    let selected_tags: Vec<SelectedTag> = selected
        .iter()
        .enumerate()
        .map(|(i, tag)| {
            let tags_removed = selected
                .iter()
                .enumerate()
                .filter_map(|(j, s)| if i == j { None } else { Some(s.clone()) })
                .collect();
            let action = serde_html_form::to_string(IndexParams {
                tags: Some(tags_removed),
                page: None,
                ..*query
            })?;

            Ok(SelectedTag {
                name: tag.clone(),
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
                page: None,
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
    let selected: HashSet<&String> = HashSet::from_iter(selected.iter());

    tags.iter()
        .filter(|tag| !selected.contains(&tag.name))
        .cloned()
        .collect()
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

fn sort_link(query: &IndexParams, dir: SortDirection) -> anyhow::Result<String> {
    let new_dir = match dir {
        SortDirection::Asc => SortDirection::Desc,
        SortDirection::Desc => SortDirection::Asc,
    };

    Ok(serde_html_form::to_string(IndexParams {
        dir: Some(new_dir),
        tags: query.tags.clone(),
        ..*query
    })?)
}
