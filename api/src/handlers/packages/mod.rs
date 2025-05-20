use std::io::{Cursor, Read};

use crate::State;
use crate::handlers::distributions::types::Package;
use axum::{
    Extension, Json,
    extract::Path,
    response::{IntoResponse, Response},
};
use axum::{http::StatusCode, response::Result};
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use smith::utils::schema;
use tempfile::NamedTempFile;
use tracing::{debug, error};

const PACKAGES_TAG: &str = "packages";

#[utoipa::path(
    get,
    path = "/packages",
    responses(
        (status = 200, description = "List of registered packages"),
        (status = 500, description = "Failure", body = String),
    ),
    security(
        ("Access Token" = [])
    ),
    tag = PACKAGES_TAG
)]
#[tracing::instrument]
pub async fn get_packages(
    Extension(state): Extension<State>,
) -> Result<Json<Vec<schema::Package>>, StatusCode> {
    let packages = sqlx::query_as!(
        schema::Package,
        "SELECT * FROM package ORDER BY package.created_at DESC"
    )
    .fetch_all(&state.pg_pool)
    .await
    .map_err(|err| {
        error!("Failed to get packages {err}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(packages))
}

#[derive(Debug, TryFromMultipart, utoipa::ToSchema)]
pub struct ReleasePackageRequest {
    #[schema(format = Binary, value_type = String)]
    #[form_data(limit = "1Gib")]
    file: FieldData<NamedTempFile>,
}

#[utoipa::path(
    put,
    path = "/packages",
    request_body(content = ReleasePackageRequest, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Sucess releasing package"),
        (status = 500, description = "Failure", body = String),
    ),
    security(
        ("Access Token" = [])
    ),
    tag = PACKAGES_TAG
)]
#[tracing::instrument]
pub async fn release_package(
    Extension(state): Extension<State>,
    TypedMultipart(ReleasePackageRequest { mut file }): TypedMultipart<ReleasePackageRequest>,
) -> Result<StatusCode, StatusCode> {
    let file_name = file.metadata.file_name.unwrap_or(String::from("data.bin"));

    let mut buf = Vec::new();
    file.contents.read_to_end(&mut buf).map_err(|err| {
        error!("error: failed to release package {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let mut cursor = Cursor::new(&buf);
    let mut pkg = debpkg::DebPkg::parse(&mut cursor).unwrap();

    let control_tar = pkg.control().unwrap();
    let control = debpkg::Control::extract(control_tar).unwrap();
    let arch = control.get("Architecture").unwrap();
    debug!("File Name: {}", file_name);
    debug!("Package Name: {}", control.name());
    debug!("Package Version: {}", control.version());
    debug!("Package Architecture: {}", arch);

    Package::new(
        control.name(),
        control.version(),
        arch,
        &file_name,
        &buf,
        state.config,
        &state.pg_pool,
    )
    .await
    .map_err(|err| {
        error!("error: Failed to save package: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(StatusCode::OK)
}

#[tracing::instrument]
pub async fn get_package_by_id(
    Path(package_id): Path<i32>,
    Extension(state): Extension<State>,
) -> Result<Response, Response> {
    get_package_info_by_id(Path(package_id), Extension(state))
        .await
        .map(|json| json.into_response())
}

#[tracing::instrument]
async fn get_package_info_by_id(
    Path(package_id): Path<i32>,
    Extension(state): Extension<State>,
) -> Result<Json<Package>, Response> {
    let package = sqlx::query_as!(
        Package,
        "SELECT * FROM package WHERE package.id = $1",
        package_id
    )
    .fetch_one(&state.pg_pool)
    .await
    .map_err(|err| {
        error!("Failed to get package {err}");
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    })?;

    Ok(Json(package))
}

#[tracing::instrument]
pub async fn delete_package_by_id(
    Path(package_id): Path<i32>,
    Extension(state): Extension<State>,
) -> Result<StatusCode, StatusCode> {
    Package::delete(&package_id, state.config, &state.pg_pool)
        .await
        .map_err(|err| {
            error!("Failed to delete the package {err}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(StatusCode::NO_CONTENT)
}
