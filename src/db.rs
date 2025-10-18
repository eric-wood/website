use std::convert;

use crate::{Photo, Tag};
use serde::{Deserialize, Serialize};
use sqlx::{QueryBuilder, Sqlite, SqlitePool};

#[derive(Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SortDirection {
    Asc,
    Desc,
}

impl convert::From<SortDirection> for String {
    fn from(value: SortDirection) -> String {
        match value {
            SortDirection::Asc => "asc",
            SortDirection::Desc => "desc",
        }
        .to_string()
    }
}

impl SortDirection {
    pub fn to_sql(self) -> String {
        match self {
            Self::Asc => "ASC",
            Self::Desc => "DESC",
        }
        .to_string()
    }
}

#[derive(Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SortField {
    TakenAt,
    CreatedAt,
}

impl convert::From<SortField> for String {
    fn from(value: SortField) -> String {
        match value {
            SortField::TakenAt => "taken_at",
            SortField::CreatedAt => "created_at",
        }
        .to_string()
    }
}

impl SortField {
    pub fn to_sql(self) -> String {
        self.into()
    }
}

pub struct Sort {
    pub direction: SortDirection,
    pub field: SortField,
}

impl Sort {
    pub fn to_sql(&self) -> String {
        let field = self.field.to_sql();
        let direction = self.direction.to_sql();
        format!("{field} {direction}")
    }
}

pub struct Pagination {
    pub limit: u32,
    pub page: u32,
}

pub struct PhotoQuery {
    pub sort: Sort,
    pub pagination: Pagination,
    pub tag: Option<String>,
}

pub async fn get_photos(pool: &SqlitePool, pq: PhotoQuery) -> anyhow::Result<(u32, Vec<Photo>)> {
    let photos = build_photo_query(&pq, false)
        .build_query_as()
        .fetch_all(pool)
        .await?;

    // I like how convenient `QueryAs` is but it doesn't make it easy for me to select a COUNT for
    // the results on top of it so whatever! One more query!!
    let (count,): (u32,) = build_photo_query(&pq, true)
        .build_query_as()
        .fetch_one(pool)
        .await?;

    Ok((count, photos))
}

fn build_photo_query(pq: &PhotoQuery, count: bool) -> QueryBuilder<Sqlite> {
    let offset = pq.pagination.limit * (pq.pagination.page - 1);

    let select = if count {
        "SELECT COUNT(*) FROM photos "
    } else {
        "SELECT * FROM photos "
    };
    let mut query = QueryBuilder::new(select);

    if let Some(tag) = pq.tag.clone() {
        query
            .push("JOIN photo_tags ON photo_tags.tag = ")
            .push_bind(tag)
            .push(" AND photo_tags.photo_id = photos.id ");
    };

    if !count {
        let sort_sql = pq.sort.to_sql();
        query.push(format!("ORDER BY {sort_sql}"));

        query
            .push(" LIMIT ")
            .push_bind(pq.pagination.limit)
            .push(" OFFSET ")
            .push_bind(offset);
    }

    query
}

pub async fn get_photo(pool: &SqlitePool, id: String) -> anyhow::Result<Photo> {
    let photo: Photo = sqlx::query_as("SELECT * FROM photos WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await?;

    Ok(photo)
}

pub async fn get_tags(pool: &SqlitePool) -> anyhow::Result<Vec<Tag>> {
    let tags: Vec<Tag> = sqlx::query_as("SELECT * FROM tags ORDER BY count DESC")
        .fetch_all(pool)
        .await?;

    Ok(tags)
}
