use crate::Photo;
use sqlx::SqlitePool;

pub async fn get_photos(pool: &SqlitePool) -> anyhow::Result<Vec<Photo>> {
    let photos = sqlx::query_as::<_, Photo>(
        "SELECT id, caption, filename, taken_at, created_at FROM photos",
    )
    .fetch_all(pool)
    .await?;

    Ok(photos)
}
