use crate::{
    commands::{AuthCommands, Commands, DownloadCommands, LoginRequest, ModList},
    error::Result,
    mods::{retrieve_factorio_auth, retrieve_mod_file, retrieve_mod_release, ModDetails},
};
use clap::Parser;
use tracing::info;

mod commands;
mod error;
mod mods;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = commands::Args::parse();
    match args.command {
        Commands::Auth(auth) => {
            let cmd = auth.command;
            match cmd {
                AuthCommands::Login(LoginRequest { username, password }) => {
                    let client = reqwest::Client::new();
                    let res = client
                        .post(format!("https://auth.factorio.com/api-login?username={username}&password={password}"))
                        .send()
                        .await?;

                    match res.status() {
                        reqwest::StatusCode::OK => (),
                        _ => {
                            let body = res.json::<factorio_api::ErrorResponse>().await?;
                            tracing::error!("{:#?}", body);
                            return Ok(());
                        }
                    }

                    let body = res.json::<factorio_api::LoginResponse>().await?;
                    println!("{}", body.token);
                }
            }
        }
        Commands::Download(download) => {
            let cmd = download.command;
            match cmd {
                DownloadCommands::SingleMod(details) => {
                    info!(
                        "Downloading mod {} version {}",
                        details.name, details.version
                    );
                    // Retrieve the mod's info
                    let client = reqwest::Client::new();

                    let release_info = retrieve_mod_release(&client, &details).await?;

                    // determine the download_url using the version
                    let download_url = release_info.download_url;
                    let file_name = release_info.file_name;

                    tracing::debug!("Downloading from {}", download_url);

                    let (factorio_username, factorio_token) = retrieve_factorio_auth();

                    retrieve_mod_file(
                        &client,
                        &download_url,
                        std::path::Path::new(&file_name),
                        &factorio_username,
                        &factorio_token,
                    )
                    .await?;
                }
                DownloadCommands::ModList(ModList { file, directory }) => {
                    let file = tokio::fs::read_to_string(file).await?;
                    let mod_list = ron::from_str::<Vec<ModDetails>>(&file)?;

                    // Retrieve info for all of the mods
                    let client = reqwest::Client::new();

                    let mut releases = Vec::new();
                    for mod_item in mod_list {
                        println!(
                            "Retrieving mod {} version {}",
                            mod_item.name, mod_item.version
                        );
                        let release_info = retrieve_mod_release(&client, &mod_item).await?;
                        releases.push(release_info);
                    }

                    info!(?releases, "Downloading {} mods", releases.len());

                    // create the directory if it doesn't exist
                    tokio::fs::create_dir_all(&directory).await?;

                    let download_tasks = releases
                        .into_iter()
                        .map(|release| async {
                            let download_url = release.download_url;
                            let file_name = release.file_name;

                            tracing::debug!("Downloading from {}", download_url);

                            let (factorio_username, factorio_token) = retrieve_factorio_auth();

                            retrieve_mod_file(
                                &client,
                                &download_url,
                                &std::path::Path::new(&directory).join(&file_name),
                                &factorio_username,
                                &factorio_token,
                            )
                            .await
                        })
                        .collect::<Vec<_>>();

                    futures::future::try_join_all(download_tasks).await?;
                }
            }
        }
    }

    Ok(())
}
