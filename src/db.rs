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
    pub tags: Vec<String>,
}

pub async fn get_photos(
    pool: &SqlitePool,
    pq: PhotoQuery,
) -> Result<(u32, Vec<Photo>), sqlx::Error> {
    let photos = build_photo_query(&pq)
        .build_query_as()
        .fetch_all(pool)
        .await?;

    // I like how convenient `QueryAs` is but it doesn't make it easy for me to select a COUNT for
    // the results on top of it so whatever! One more query!!
    let (count,): (u32,) = build_photo_count_query(&pq)
        .build_query_as()
        .fetch_one(pool)
        .await?;

    Ok((count, photos))
}

fn build_photo_query(pq: &PhotoQuery) -> QueryBuilder<'_, Sqlite> {
    let offset = pq.pagination.limit * (pq.pagination.page - 1);

    let mut query = QueryBuilder::new("SELECT photos.* FROM photos");

    if !pq.tags.is_empty() {
        query.push(", photo_tags WHERE photo_tags.tag IN (");
        let mut separated = query.separated(", ");
        for tag in pq.tags.iter().cloned() {
            separated.push_bind(tag);
        }
        separated.push_unseparated(")");
        query
            .push(" AND photo_tags.photo_id = photos.id")
            .push(" GROUP BY photos.id HAVING COUNT(photo_tags.photo_id)=")
            .push_bind(pq.tags.len() as u32);
    };

    let sort_sql = pq.sort.to_sql();
    query.push(format!(" ORDER BY {sort_sql}"));

    query
        .push(" LIMIT ")
        .push_bind(pq.pagination.limit)
        .push(" OFFSET ")
        .push_bind(offset);

    query
}

fn build_photo_count_query(pq: &PhotoQuery) -> QueryBuilder<'_, Sqlite> {
    if pq.tags.is_empty() {
        return QueryBuilder::new("SELECT COUNT(*) FROM photos");
    }

    let mut query =
        QueryBuilder::new("SELECT COUNT(*) FROM (SELECT 1 FROM photo_tags WHERE tag IN (");
    let mut separated = query.separated(", ");
    for tag in pq.tags.iter().cloned() {
        separated.push_bind(tag);
    }
    separated.push_unseparated(")");
    query
        .push(" GROUP BY photo_id HAVING COUNT(photo_id)=")
        .push_bind(pq.tags.len() as u32)
        .push(")");

    query
}

pub async fn get_photo(pool: &SqlitePool, id: &String) -> Result<Photo, sqlx::Error> {
    let photo: Photo = sqlx::query_as("SELECT * FROM photos WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await?;

    Ok(photo)
}

pub async fn get_tags(
    pool: &SqlitePool,
    selected_tags: &Option<Vec<String>>,
) -> anyhow::Result<Vec<Tag>> {
    let tags = if let Some(selected_tags) = selected_tags {
        let mut query = QueryBuilder::new(
            r#"
        WITH filtered_photos AS (
            SELECT photos.id FROM photos, photo_tags
            WHERE photo_tags.tag IN ("#,
        );
        let mut separated = query.separated(", ");
        for tag in selected_tags.iter().clone() {
            separated.push_bind(tag);
        }
        separated.push_unseparated(") ");
        query
            .push(
                r#"
            AND photo_tags.photo_id = photos.id
            GROUP BY photos.id
            HAVING COUNT(photo_tags.photo_id)="#,
            )
            .push_bind(selected_tags.len() as u32)
            .push(
                r#"
            )
            SELECT DISTINCT photo_tags.tag, COUNT(photo_tags.tag) FROM photo_tags
            JOIN filtered_photos ON filtered_photos.id = photo_tags.photo_id
            GROUP BY photo_tags.tag
            ORDER BY count(photo_tags.tag) DESC
            "#,
            );
        let results: Vec<(String, u32)> = query.build_query_as().fetch_all(pool).await?;
        results
            .into_iter()
            .map(|(name, count)| Tag { name, count })
            .collect()
    } else {
        let tags: Vec<Tag> = sqlx::query_as("SELECT * FROM tags ORDER BY count DESC")
            .fetch_all(pool)
            .await?;
        tags
    };

    Ok(tags)
}

pub async fn get_photo_tags(pool: &SqlitePool, photo_id: &String) -> anyhow::Result<Vec<Tag>> {
    let tags: Vec<Tag> = sqlx::query_as(
        r#"
        SELECT tags.*
        FROM tags
        JOIN photo_tags
        WHERE photo_tags.tag = tags.name AND photo_id = ?
        ORDER BY count DESC
        "#,
    )
    .bind(photo_id)
    .fetch_all(pool)
    .await?;

    Ok(tags)
}
