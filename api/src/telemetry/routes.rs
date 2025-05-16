use crate::State;
use crate::db::DeviceWithToken;
use crate::modem::schema::Modem;
use axum::body::{Body, to_bytes};
use axum::extract::Request;
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde::Deserialize;
use tracing::{error, info};

pub async fn victoria(
    Extension(state): Extension<State>,
    req: Request<Body>,
) -> Result<StatusCode, StatusCode> {
    let client = match &state.config.victoria_metrics_client {
        Some(client_config) => client_config,
        None => return Err(StatusCode::NOT_IMPLEMENTED),
    };

    info!("Victoria Metrics request: {:?}", req);

    let (method, mut headers, body) = {
        let (parts, body) = req.into_parts();
        (parts.method, parts.headers, body)
    };

    headers.remove("authorization");

    let body_bytes = to_bytes(body, usize::MAX).await.map_err(|err| {
        error!("Failed to read body bytes: {}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let request = client
        .client
        .request(method, &client.url)
        .headers(headers)
        .body(body_bytes);

    info!("Victoria Proxy Request {:?}", request);

    let request = request.send().await;

    info!("Victoria Metrics {:?}", request);

    let response = request.map_err(|err| {
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
