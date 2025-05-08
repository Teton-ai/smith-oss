use anyhow;
use std::path::Path;
use tracing::info;

pub async fn extract_file_here(file: &str) -> anyhow::Result<String> {
    // Set the target to the file location minus the file name

    let target = Path::new(file).parent().unwrap().to_str().unwrap();

    extract_file(file, target).await
}

pub async fn extract_file(file: &str, target: &str) -> anyhow::Result<String> {
    let output = std::process::Command::new("tar")
        .arg("-xpf")
        .arg(file)
        .arg("-C")
        .arg(target)
        .output();

    if let Err(e) = output {
        return Err(anyhow::anyhow!("Failed to extract file: {}", e));
    }

    let output = output.unwrap();

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Failed to extract tar.gz file: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    info!("File extracted successfully: {}", file);

    Ok(format!("Successfully extracted {}", file))
}

pub async fn execute_script(
    file: &str,

    arguments: Vec<String>,

    folder: Option<&str>,
) -> anyhow::Result<String> {
    let mut new_args = Vec::new();

    new_args.push(file.to_owned());

    new_args.extend(arguments);

    execute_system_command("bash", new_args, folder).await
}

pub async fn execute_system_command(
    command: &str,

    arguments: Vec<String>,

    folder: Option<&str>,
) -> anyhow::Result<String> {
    let mut cmd = std::process::Command::new(command);

    for arg in arguments {
        cmd.arg(arg);
    }

    // Check if a folder string exists, if so, push that folder as workingdir

    if let Some(folder) = folder {
        cmd.current_dir(folder);
    }

    let output = cmd.output();

    if let Err(e) = output {
        return Err(anyhow::anyhow!("{}", e));
    }

    let output = output.unwrap();

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "stderr: {}stdout: {}",
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    info!("System command executed successfully: {}", command);

    Ok(format!("Successfully executed {} - {}", command, stdout))
}
