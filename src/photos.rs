use serde::Serialize;
use sqlx::FromRow;

#[derive(FromRow, Serialize)]
pub struct Photo {
    pub id: String,
    pub caption: String,
    pub filename: String,
    // pub taken_at TEXT,
    // pub created_at TEXT
}
