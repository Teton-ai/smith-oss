use crate::magic::{MagicHandle, structure};
use crate::shutdown::ShutdownHandler;
use reqwest::{Client, multipart};
use tokio::fs;
use tracing::error;
use walkdir::WalkDir;

pub async fn files_upload(path: &str) -> anyhow::Result<()> {
    let shutdown = ShutdownHandler::new();

    let configuration = MagicHandle::new(shutdown.signals());

    configuration.load(None).await;

    let client = Client::new();
    let server_api_url = configuration.get_server().await;
    let metadata = fs::metadata(path).await?;

    if metadata.is_file() {
        upload_file(path, &client, &server_api_url).await?;
    } else {
        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
        {
            let file_path = entry.path();
            upload_file(file_path.to_str().unwrap(), &client, &server_api_url).await?;
        }
    }
    Ok(())
}

async fn upload_file(file_path: &str, client: &Client, server_api_url: &str) -> anyhow::Result<()> {
    let content = fs::read(file_path).await?;
    let file_name = std::path::Path::new(file_path)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap();
    let form = multipart::Form::new().part(
        "file",
        multipart::Part::bytes(content).file_name(file_name.to_string()),
    );

    let conf = match structure::MagicFile::load(None) {
        Ok((conf, _path)) => Some(conf),
        Err(err) => {
            error!("Failed to load magic file: {}", err);
            None
        }
    };

    if conf.is_none() {
        error!("Failed to load magic file");
        return Ok(());
    }

    let token = conf.unwrap().get_token().unwrap_or_default();

    let response = client
        .post(format!("{}/upload", &server_api_url))
        .header("Authorization", format!("Bearer {}", token))
        .multipart(form)
        .send()
        .await?;

    if !response.status().is_success() {
        error!("Failed to upload file {}: {}", file_name, response.status());
    }
    Ok(())
}
