use crate::downloader::DownloaderHandle;
use crate::filemanager::FileManagerHandle;
use crate::utils::schema::{SafeCommandResponse, SafeCommandRx};

// pub(super) async fn open_port(
//     id: i32,
//     tunnel_handle: &TunnelHandle,
//     port: Option<u16>,
// ) -> SafeCommandResponse {
//     let remote_port = tunnel_handle.start_tunnel(port).await;
//     let status = if remote_port > 0 { 0 } else { -1 };

//     SafeCommandResponse {
//         id,
//         command: SafeCommandRx::OpenTunnel {
//             port_server: remote_port,
//         },
//         status,
//     }
// }

// pub(super) async fn close_ssh(id: i32, tunnel_handle: &TunnelHandle) -> SafeCommandResponse {
//     tunnel_handle.stop_ssh_tunnel().await;

//     SafeCommandResponse {
//         id,
//         command: SafeCommandRx::TunnelClosed,
//         status: 0,
//     }
// }

pub(super) async fn download_ota(
    id: i32,
    download_handle: &DownloaderHandle,
    file_handle: &FileManagerHandle,
    tools_file: &str,
    package_file: &str,
    rate: f64,
) -> SafeCommandResponse {
    // Create the storage folders
    let arguments: Vec<String> = vec!["-p".to_owned(), "/ota".to_owned()];
    let result = file_handle
        .execute_system_command("mkdir", arguments, None)
        .await;
    if let Err(_) = result {
        return SafeCommandResponse {
            id,
            command: SafeCommandRx::DownloadOTA,
            status: -1,
        };
    }

    let arguments: Vec<String> = vec!["-p".to_owned(), "/otatool".to_owned()];
    let result = file_handle
        .execute_system_command("mkdir", arguments, None)
        .await;
    if let Err(_) = result {
        return SafeCommandResponse {
            id,
            command: SafeCommandRx::DownloadOTA,
            status: -1,
        };
    }

    // Download the OTA tools
    let remote_file = format!("ota/{}", tools_file);
    let result = download_handle
        .download(remote_file.as_str(), "/otatool/ota_tools.tbz2", rate)
        .await;
    if let Err(_) = result {
        return SafeCommandResponse {
            id,
            command: SafeCommandRx::DownloadOTA,
            status: -1,
        };
    }

    let remote_file = format!("ota/{}", package_file);
    let result = download_handle
        .download(
            remote_file.as_str(),
            "/ota/ota_payload_package.tar.gz",
            rate,
        )
        .await;
    if let Err(_) = result {
        return SafeCommandResponse {
            id,
            command: SafeCommandRx::DownloadOTA,
            status: -1,
        };
    }

    SafeCommandResponse {
        id,
        command: SafeCommandRx::DownloadOTA,
        status: 0,
    }
}

pub(super) async fn start_ota(id: i32, download_handle: &DownloaderHandle) -> SafeCommandResponse {
    // TODO: Think about adding its own response here?
    // The function will auto restart the device so I don't think we will ever see this
    SafeCommandResponse {
        id,
        command: SafeCommandRx::DownloadOTA,
        status: 0,
    }
}

pub(super) async fn check_ota(id: i32, download_handle: &DownloaderHandle) -> SafeCommandResponse {
    SafeCommandResponse {
        id,
        command: SafeCommandRx::CheckOTAStatus,
        status: 0,
    }
}
