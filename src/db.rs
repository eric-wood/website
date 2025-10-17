use std::convert;

use crate::{Photo, Tag};
use serde::{Deserialize, Serialize};
use sqlx::{QueryBuilder, Sqlite, SqlitePool, query::QueryAs};

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

pub async fn get_photo_count(pool: &SqlitePool) -> anyhow::Result<u32> {
    let result: (u32,) = sqlx::query_as("SELECT COUNT(*) FROM photos")
        .fetch_one(pool)
        .await?;
    Ok(result.0)
}

pub async fn get_photos(pool: &SqlitePool, pq: PhotoQuery) -> anyhow::Result<Vec<Photo>> {
    let offset = pq.pagination.limit * (pq.pagination.page - 1);

    let mut query: QueryBuilder<Sqlite> = QueryBuilder::new("SELECT * FROM photos ");

    if let Some(tag) = pq.tag {
        query
            .push("JOIN photo_tags ON photo_tags.tag = ")
            .push_bind(tag)
            .push(" AND photo_tags.photo_id = photos.id ");
    };

    let sort_sql = pq.sort.to_sql();
    query.push(format!("ORDER BY {sort_sql}"));

    query
        .push(" LIMIT ")
        .push_bind(pq.pagination.limit)
        .push(" OFFSET ")
        .push_bind(offset);

    let query: QueryAs<'_, Sqlite, Photo, _> = query.build_query_as();
    let photos = query.fetch_all(pool).await?;

    Ok(photos)
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
