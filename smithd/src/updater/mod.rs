mod actor;
mod handler;

pub use handler::Handler as UpdaterHandle;

// use crate::magic::MagicHandle;
// use crate::shutdown::{ShutdownHandler, ShutdownSignals};
// use crate::utils::network::NetworkClient;
// use anyhow::{Context, Result};
// use tokio::process::Command;
// use tokio::{
//     sync::{mpsc, oneshot},
//     time,
// };
// use tracing::{error, info, warn};

// enum UpdaterStatus {
//     Idle,
//     Updating,
//     Upgrading,
// }

// struct Updater {
//     shutdown: ShutdownSignals,
//     receiver: mpsc::Receiver<UpdaterMessage>,
//     magic: MagicHandle,
//     status: UpdaterStatus,
//     network: NetworkClient,
//     last_update: Option<Result<time::Instant, anyhow::Error>>,
//     last_upgrade: Option<Result<time::Instant, anyhow::Error>>,
// }

// enum UpdaterMessage {
//     Update,
//     Upgrade,
//     StatusReport { rpc: oneshot::Sender<String> },
// }

// impl Updater {
//     fn new(
//         shutdown: ShutdownSignals,
//         receiver: mpsc::Receiver<UpdaterMessage>,
//         magic: MagicHandle,
//     ) -> Self {
//         let network = NetworkClient::new();
//         Self {
//             shutdown,
//             receiver,
//             magic,
//             network,
//             status: UpdaterStatus::Idle,
//             last_update: None,
//             last_upgrade: None,
//         }
//     }

//     async fn handle_message(&mut self, msg: UpdaterMessage) {
//         match msg {
//             UpdaterMessage::Update => {
//                 info!("Checking for updates");
//                 self.status = UpdaterStatus::Updating;
//                 let res = self.check_for_updates().await.map(|_| time::Instant::now());
//                 info!("Check for updates result: {:?}", res);
//                 self.last_update = Some(res);
//                 self.status = UpdaterStatus::Idle;
//             }
//             UpdaterMessage::Upgrade => {
//                 info!("Upgrading device");
//                 self.status = UpdaterStatus::Upgrading;
//                 let res = self.upgrade_device().await.map(|_| time::Instant::now());
//                 info!("Upgrading result: {:?}, changing to app mode", res);
//                 self.last_upgrade = Some(res);
//                 self.status = UpdaterStatus::Idle;
//             }
//             UpdaterMessage::StatusReport { rpc } => {
//                 // take the instants and format them nicely with X seconds ago
//                 let interval = |time: time::Instant| {
//                     let duration = time.elapsed();
//                     let seconds = duration.as_secs();
//                     let minutes = seconds / 60;
//                     let hours = minutes / 60;
//                     let days = hours / 24;

//                     if days > 0 {
//                         format!("{} days ago", days)
//                     } else if hours > 0 {
//                         format!("{} hours ago", hours)
//                     } else if minutes > 0 {
//                         format!("{} minutes ago", minutes)
//                     } else {
//                         format!("{} seconds ago", seconds)
//                     }
//                 };

//                 let last_update_string = match &self.last_update {
//                     Some(Ok(time)) => interval(*time),
//                     Some(Err(err)) => format!("Error: {}", err),
//                     None => "Never".to_string(),
//                 };

//                 let last_upgrade_string = match &self.last_upgrade {
//                     Some(Ok(time)) => interval(*time),
//                     Some(Err(err)) => format!("Error: {}", err),
//                     None => "Never".to_string(),
//                 };

//                 let _rpc = rpc.send(format!(
//                     "Last Update: {} | Last Upgrade: {}",
//                     last_update_string, last_upgrade_string
//                 ));
//             }
//         }
//     }

//     async fn run(&mut self) {
//         info!("Updater Starting");
//         let hostname = self.magic.get_server().await;
//         self.network.set_hostname(hostname);

//         let mut update_check_interval = tokio::time::interval(tokio::time::Duration::from_secs(60));

//         let mut can_exit = false;

//         loop {
//             tokio::select! {
//                 Some(msg) = self.receiver.recv() => {
//                     self.handle_message(msg).await;
//                     can_exit = true;
//                 }
//                 _ = update_check_interval.tick() => {
//                     // Compare release id and target release every 60s.
//                     let release_id = self.magic.get_release_id().await;
//                     let target_release_id = self.magic.get_target_release_id().await;

//                     // Update and upgrade in case the ids differ.
//                     if release_id != target_release_id {
//                         info!("upgrading from release_id {release_id:?} to target_release_id {target_release_id:?}");

//                         if let Err(err) = self.check_for_updates().await {
//                             error!("error: failed to check for updates {err}");
//                             continue;
//                         }

//                         if let Err(err) = self.upgrade_device().await {
//                             error!("error: failed to upgrade device {err}");
//                             continue;
//                         }

//                         // Signal that the updated has been run by setting the release id to the target release id.
//                         self.magic.set_release_id(target_release_id).await;
//                     }
//                 }
//                 _ = self.shutdown.token.cancelled() => {
//                     loop {
//                         if can_exit {
//                             break;
//                         } else {
//                             info!("Updater waiting for tasks to finish");
//                             time::sleep(time::Duration::from_secs(1)).await;
//                         }
//                     }
//                     break;
//                 }
//             }
//         }
//         info!("Updater shutting down");
//     }

//     /// Checks whether packages are up to date.
//     ///
//     /// Returns `Ok` if all packages are, `Err` otherwise.
//     async fn are_packages_up_to_date(&self) -> Result<()> {
//         let shutdown = ShutdownHandler::new();
//         let configuration = MagicHandle::new(shutdown.signals());
//         configuration.load(None).await;

//         let magic_packages = configuration.get_packages().await;

//         // check the system version of the packages in the magic file
//         for package in magic_packages {
//             let installed_version = package.get_system_version().await?;
//             let magic_toml_version = package.version;

//             if magic_toml_version != installed_version {
//                 return Err(anyhow::anyhow!(
//                     "Package {} is not up to date",
//                     package.name
//                 ));
//             }
//         }

//         Ok(())
//     }

//     async fn check_for_updates(&self) -> Result<()> {
//         // apt update on check for updates
//         Command::new("sh")
//             .arg("-c")
//             .arg("apt update -y")
//             .output()
//             .await
//             .with_context(|| "Failed to run apt update")?;

//         let target_release_id = self.magic.get_target_release_id().await;
//         info!("Checking for updates");
//         info!("Target release id: {:?}", target_release_id);

//         // get current configured packages
//         let local_packages = self.magic.get_packages().await;

//         // ask postman for the packages of the target release
//         let target_packages = self.network.get_release_packages(target_release_id).await;

//         info!("== Current packages ==");
//         for package in local_packages.iter() {
//             info!(
//                 "Local: {} {} {}",
//                 package.name, package.version, package.file
//             );
//         }
//         info!("++ Release packages ++");
//         for package in target_packages.iter() {
//             info!(
//                 "Remote: {} {} {}",
//                 package.name, package.version, package.file
//             );
//         }

//         let mut up_to_date = true;
//         // compare the packages and check if we need to update
//         for target_package in target_packages.iter() {
//             let package_not_on_magic_file = !local_packages.contains(target_package);
//             let package_not_installed = tokio::process::Command::new("dpkg")
//                 .arg("-l")
//                 .arg(&target_package.name)
//                 .output()
//                 .await
//                 .map(|output| !output.status.success())
//                 .unwrap_or(true);

//             if package_not_on_magic_file || package_not_installed {
//                 info!("Package {} is not installed", target_package.name);
//                 up_to_date = false;
//                 // we need to install the package
//                 self.network.get_package(&target_package.file).await?;
//             }
//         }

//         if !up_to_date {
//             self.magic.set_packages(target_packages).await;
//         }

//         Ok(())
//     }

//     async fn upgrade_device(&self) -> Result<()> {
//         // TODO: not sure if we should update by default
//         //self.check_for_updates().await?;

//         // Check if previous update was successful
//         match self.last_update {
//             Some(Ok(time)) => {
//                 let time_since_last_update = time.elapsed();
//                 info!(
//                     "Previous update was successful {:?}",
//                     time_since_last_update
//                 );
//             }
//             Some(_) => {
//                 warn!("Previous update was not successful");
//                 return Ok(());
//             }
//             None => {
//                 info!("No previous update, continuing anyway");
//             }
//         }

//         let packages_from_magic = self.magic.get_packages().await;

//         // check if all packages are available locally
//         for package in packages_from_magic.iter() {
//             info!("Checking package: {}", package.name);
//             let package_name = &package.name;
//             let package_file = &package.file;

//             // check if package is available locally
//             let path = std::env::current_dir()?;
//             let packages_folder = path.join("packages");
//             let package_file = packages_folder.join(package_file);

//             if package_file.exists() {
//                 info!("Package {} exists locally", package_name);
//                 continue;
//             } else {
//                 info!("Package {} does not exist locally", package_name);
//                 return Err(anyhow::anyhow!(
//                     "Package {} does not exist locally",
//                     package_name
//                 ));
//             }
//         }

//         // now install packages
//         let mut update_smith = false;
//         for package in packages_from_magic.into_iter() {
//             let package_name = package.name;
//             let package_file = package.file;
//             let package_version = package.version;

//             let path = std::env::current_dir()?;
//             let packages_folder = path.join("packages");
//             let package_file = packages_folder.join(&package_file);

//             // check if version on system is the one we should be running
//             let output = match Command::new("dpkg")
//                 .arg("-l")
//                 .arg(&package_name)
//                 .output()
//                 .await
//             {
//                 Ok(output) => output,
//                 Err(e) => {
//                     error!("Failed to execute dpkg command for {}: {}", package_name, e);
//                     continue;
//                 }
//             };

//             let mut package_installed = false;
//             if output.status.success() {
//                 if let Ok(stdout) = String::from_utf8(output.stdout) {
//                     let lines: Vec<&str> = stdout.lines().collect();
//                     if let Some(package_info) = lines.get(5) {
//                         let fields: Vec<&str> = package_info.split_whitespace().collect();
//                         if let Some(version) = fields.get(2) {
//                             info!("> {} | {} => {}", package_name, version, &package_version);
//                             package_installed = version == &package_version;
//                         } else {
//                             error!("Failed to get version for package {}", package_name);
//                         }
//                     } else {
//                         error!("Failed to get package info for {}", package_name);
//                     }
//                 } else {
//                     error!("Failed to parse dpkg output for {}", package_name);
//                 }
//             }

//             if !package_installed {
//                 if package_name == "smith" || package_name == "smith_amd64" {
//                     update_smith = true;
//                     continue;
//                 }
//                 let install_command = format!(
//                     "sudo apt install {} -y --allow-downgrades",
//                     package_file.display()
//                 );
//                 match Command::new("sh")
//                     .arg("-c")
//                     .arg(&install_command)
//                     .output()
//                     .await
//                 {
//                     Ok(status) => {
//                         if status.status.success() {
//                             info!("Successfully installed package {}", package_name);
//                         } else {
//                             let stderr = String::from_utf8_lossy(&status.stderr);
//                             error!("Failed to install package {}: {}", package_name, stderr);
//                         }
//                     }
//                     Err(e) => {
//                         error!(
//                             "Failed to execute install command for {}: {}",
//                             package_name, e
//                         );
//                     }
//                 }
//             }
//         }
//         if update_smith {
//             let status = Command::new("sh")
//                 .arg("-c")
//                 .arg("sudo systemctl start smith-updater")
//                 .output()
//                 .await
//                 .with_context(|| "Failed to stop smith service")?;

//             if !status.status.success() {
//                 error!("Failed to start smith updater {:?}", status);
//             }
//         }

//         self.are_packages_up_to_date().await
//     }
// }

// #[derive(Clone)]
// pub struct UpdaterHandle {
//     sender: mpsc::Sender<UpdaterMessage>,
// }

// impl UpdaterHandle {
//     pub fn new(shutdown: ShutdownSignals, magic: MagicHandle) -> Self {
//         let (sender, receiver) = mpsc::channel(8);
//         let mut actor = Updater::new(shutdown, receiver, magic);
//         tokio::spawn(async move { actor.run().await });

//         Self { sender }
//     }

//     pub async fn check_for_updates(&self) -> bool {
//         // unwrap because if this fails then we are in a bad state
//         self.sender.send(UpdaterMessage::Update).await.unwrap();
//         true
//     }

//     pub async fn upgrade_device(&self) {
//         // unwrap because if this fails then we are in a bad state
//         self.sender.send(UpdaterMessage::Upgrade).await.unwrap();
//     }

//     pub async fn status(&self) -> String {
//         let (rpc, receiver) = oneshot::channel();
//         // unwrap because if this fails then we are in a bad state
//         self.sender
//             .send(UpdaterMessage::StatusReport { rpc })
//             .await
//             .unwrap();
//         receiver.await.unwrap()
//     }
// }
