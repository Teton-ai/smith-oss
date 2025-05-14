use axum::{Extension, Json};
use axum::{http::StatusCode, response::Result};
use tracing::error;

use crate::State;

pub mod types;

const AUTH_TAG: &str = "auth";

#[tracing::instrument]
#[utoipa::path(
    post,
    path = "/auth/token",
    responses(
        (status = StatusCode::OK, description = "Return found device auth", body = types::DeviceAuth),
        (status = StatusCode::UNAUTHORIZED, description = "Failed to verify token"),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Failed to retrieve device auth"),
    ),
    security(
        ("Access Token" = [])
    ),
    tag = AUTH_TAG
)]
pub async fn verify_token(
    Extension(state): Extension<State>,
    Json(token): Json<types::DeviceTokenForVerification>,
) -> Result<Json<types::DeviceAuth>, StatusCode> {
    let device = sqlx::query_as!(
        types::DeviceAuth,
        "
        SELECT device.serial_number AS serial_number, device.approved AS authorized
        FROM device
        WHERE device.token = $1
        ",
        token.token
    )
    .fetch_optional(&state.pg_pool)
    .await
    .map_err(|err| {
        error!("Failed to get device {err}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if let Some(device) = device {
        Ok(Json(device))
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}
