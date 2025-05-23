use crate::State;
use crate::db::DeviceWithToken;
use crate::modem::schema::Modem;
use axum::body::{Body, to_bytes};
use axum::extract::Request;
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde::Deserialize;
use tracing::error;

pub async fn victoria(
    Extension(state): Extension<State>,
    req: Request<Body>,
) -> Result<StatusCode, StatusCode> {
    let client_config = state
        .config
        .victoria_metrics_client
        .as_ref()
        .ok_or(StatusCode::NOT_IMPLEMENTED)?;

    let (parts, body) = req.into_parts();
    let method = parts.method;
    let mut headers = parts.headers;

    headers.remove("authorization");
    let body_bytes = to_bytes(body, usize::MAX).await.map_err(|err| {
        error!("Failed to read body bytes: {}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let response = client_config
        .client
        .request(method, &client_config.url)
        .headers(headers)
        .body(body_bytes)
        .send()
        .await;

    match response {
        Ok(res) => Ok(res.status()),
        Err(err) => {
            error!(error = %err, "Failed to forward request to VictoriaMetrics");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
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
