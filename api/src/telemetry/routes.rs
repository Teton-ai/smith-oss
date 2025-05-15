use std::error;

use crate::State;
use crate::db::DeviceWithToken;
use crate::modem::schema::Modem;
use axum::body::{Body, to_bytes};
use axum::extract::Request;
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde::Deserialize;

pub async fn victoria(
    Extension(state): Extension<State>,
    req: Request<Body>,
) -> Result<StatusCode, StatusCode> {
    let client = state.victoria_client.clone();

    let method = req.method().clone();
    let mut headers = req.headers().clone();

    headers.remove("authorization");

    let body_bytes = to_bytes(req.into_body(), usize::MAX).await.map_err(|err| {
        error!("Failed to read body bytes: {}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let request = client
        .request(method, "https://metrics.teton.ai/opentelemetry/v1/metrics")
        .headers(headers)
        .body(body_bytes);

    let response = request.send().await.map_err(|err| {
        error!("Failed to send request: {}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let status = response.status();

    Ok(status)
}

pub async fn service() -> Result<StatusCode, StatusCode> {
    Ok(StatusCode::OK)
}

#[derive(Debug, Deserialize)]
pub struct NewModem {
    pub imei: String,
    pub network_provider: String,
}

pub async fn modem(
    device: DeviceWithToken,
    Extension(state): Extension<State>,
    Json(modem): Json<Option<NewModem>>,
) -> Result<StatusCode, StatusCode> {
    tokio::spawn(async move {
        match modem {
            Some(modem) => {
                let _ = Modem::save_modem(
                    device.serial_number,
                    modem.imei,
                    modem.network_provider,
                    &state.pg_pool,
                )
                .await;
            }
            None => {
                let _ = Modem::clear_modem(device.serial_number, &state.pg_pool).await;
            }
        }
    });
    Ok(StatusCode::OK)
}
