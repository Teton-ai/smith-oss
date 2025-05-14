use crate::State;
use axum::{Extension, Json};
use axum::{http::StatusCode, response::Result};
use tracing::error;

pub mod types;

pub async fn get_tags(
    Extension(state): Extension<State>,
) -> Result<Json<Vec<types::Tag>>, StatusCode> {
    let devices = sqlx::query_as!(
        types::Tag,
        r#"SELECT
            t.id,
            t.name,
            t.color
        FROM tag t
        ORDER BY t.id"#
    )
    .fetch_all(&state.pg_pool)
    .await
    .map_err(|err| {
        error!("Failed to get devices {err}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(devices))
}

pub async fn create_tag(
    Extension(state): Extension<State>,
    Json(tag): Json<types::NewTag>,
) -> Result<Json<types::Tag>, StatusCode> {
    let new_tag = sqlx::query_as!(
        types::Tag,
        r#"INSERT INTO tag (name, color)
        VALUES ($1, $2)
        RETURNING id, name, color"#,
        tag.name,
        tag.color
    )
    .fetch_one(&state.pg_pool)
    .await
    .map_err(|err| {
        error!("Failed to create tag {err}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(new_tag))
}
