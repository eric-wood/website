use crate::date_time::DateTime;

use serde::Serialize;
use sqlx::FromRow;

#[derive(FromRow, Serialize)]
pub struct Photo {
    pub id: String,
    pub caption: String,
    pub filename: String,
    #[sqlx(try_from = "String")]
    pub taken_at: DateTime,
    #[sqlx(try_from = "String")]
    pub created_at: DateTime,
}

#[derive(FromRow, Serialize)]
pub struct Tag {
    pub name: String,
    pub count: u32,
}
