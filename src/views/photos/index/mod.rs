use std::collections::HashSet;

use minijinja::context;
use minijinja_autoreload::AutoReloader;
use serde::{Deserialize, Serialize};
use serde_valid::Validate;

use crate::{
    Photo,
    db::{SortDirection, SortField},
    models::Tag,
    templates::render,
    views::View,
};

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

#[derive(Serialize)]
struct Pagination {
    page: u32,
    num_pages: u32,
    prev_query: Option<String>,
    next_query: Option<String>,
}

static SORT_FIELDS: [SortField; 2] = [SortField::TakenAt, SortField::CreatedAt];

#[derive(Deserialize, Serialize, Clone, Validate)]
pub struct IndexParams {
    #[validate(minimum = 1)]
    pub page: Option<u32>,

    #[validate(minimum = 1)]
    #[validate(maximum = 100)]
    pub limit: Option<u32>,

    pub tags: Option<Vec<String>>,
    pub sort: Option<SortField>,
    pub dir: Option<SortDirection>,
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

pub struct PhotosIndex {
    query: IndexParams,
    photos: Vec<Photo>,
    total_photos: u32,
    tags: Vec<Tag>,
}

impl PhotosIndex {
    pub fn new(query: IndexParams, photos: Vec<Photo>, tags: Vec<Tag>, total_photos: u32) -> Self {
        Self {
            query,
            photos,
            total_photos,
            tags,
        }
    }

    fn process_tags(&self) -> anyhow::Result<(Vec<SelectedTag>, Vec<SelectableTag>)> {
        let selected_tag_strings = self.query.tags.clone().unwrap_or_default();
        let tags = tag_difference(&self.tags, &selected_tag_strings);
        let selected_tags: Vec<SelectedTag> = selected_tag_strings
            .iter()
            .enumerate()
            .map(|(i, tag)| {
                let tags_removed = selected_tag_strings
                    .iter()
                    .enumerate()
                    .filter_map(|(j, s)| if i == j { None } else { Some(s.clone()) })
                    .collect();
                let action = serde_html_form::to_string(IndexParams {
                    tags: Some(tags_removed),
                    page: None,
                    ..self.query
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
                let mut tags_added = selected_tag_strings.clone();
                tags_added.push(tag.name.clone());
                let action = serde_html_form::to_string(IndexParams {
                    tags: Some(tags_added),
                    page: None,
                    ..self.query
                })?;

                Ok(SelectableTag {
                    tag: tag.clone(),
                    action,
                })
            })
            .collect::<anyhow::Result<Vec<SelectableTag>>>()?;

        Ok((selected_tags, selectable_tags))
    }

    fn get_pagination(&self, limit: u32) -> anyhow::Result<Pagination> {
        let num_pages = (self.total_photos as f32 / limit as f32).ceil() as u32;
        let page = self.query.page.unwrap_or(1);
        let prev_query = if page > 1 {
            let prev_page = if page == 1 { None } else { Some(page - 1) };
            Some(serde_html_form::to_string(IndexParams {
                page: prev_page,
                tags: self.query.tags.clone(),
                ..self.query
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
                tags: self.query.tags.clone(),
                ..self.query
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

    fn sort_link(&self, dir: SortDirection) -> anyhow::Result<String> {
        let new_dir = match dir {
            SortDirection::Asc => SortDirection::Desc,
            SortDirection::Desc => SortDirection::Asc,
        };

        Ok(serde_html_form::to_string(IndexParams {
            dir: Some(new_dir),
            tags: self.query.tags.clone(),
            ..self.query
        })?)
    }
}

impl View for PhotosIndex {
    fn render(&self, reloader: &AutoReloader) -> anyhow::Result<String> {
        let default = IndexParams::default();
        let limit = self.query.limit.unwrap_or(default.limit.unwrap());
        let sort_dir = self.query.dir.unwrap_or(default.dir.unwrap());
        let sort_field = self.query.sort.unwrap_or(default.sort.unwrap());

        let (current_tags, tags) = self.process_tags()?;

        let pagination = self.get_pagination(limit)?;
        let sort_link = self.sort_link(sort_dir)?;

        let html = render(
            reloader,
            "views/photos/index",
            context! {
                photos => self.photos,
                tags,
                current_tags,
                sort_dir,
                sort_field,
                pagination,
                sort_fields => SORT_FIELDS,
                sort_link,
            },
        )?;

        Ok(html)
    }
}

fn tag_difference(tags: &[Tag], selected: &[String]) -> Vec<Tag> {
    let selected: HashSet<&String> = HashSet::from_iter(selected.iter());

    tags.iter()
        .filter(|tag| !selected.contains(&tag.name))
        .cloned()
        .collect()
}
