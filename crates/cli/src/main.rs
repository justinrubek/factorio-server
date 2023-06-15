use crate::{
    commands::{AuthCommands, Commands, LoginRequest},
    error::Result,
};
use clap::Parser;
use commands::{DownloadCommands, ModDetails};
use tokio::io::AsyncWriteExt;
use tracing::info;

mod commands;
mod error;

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
                DownloadCommands::SingleMod(ModDetails { name, version }) => {
                    info!("Downloading mod {name} version {version}");
                    // Retrieve the mod's info
                    let client = reqwest::Client::new();

                    let res = client
                        .get(format!("https://mods.factorio.com/api/mods/{name}"))
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

                    let body = res.json::<factorio_api::ModResponse>().await?;
                    // determine the download_url using the version
                    let (download_url, file_name) = body
                        .releases
                        .iter()
                        .find(|release| release.version == version)
                        .map(|release| (release.download_url.clone(), release.file_name.clone()))
                        .unwrap_or_else(|| {
                            tracing::error!("Version {} not found for mod {}", version, name);
                            std::process::exit(1);
                        });

                    tracing::debug!("Downloading from {}", download_url);

                    // Retrieve auth token and username from environment variables
                    let factorio_username =
                        std::env::var("FACTORIO_USERNAME").expect("FACTORIO_USERNAME not set");
                    let factorio_token =
                        std::env::var("FACTORIO_TOKEN").expect("FACTORIO_TOKEN not set");

                    // Retrieve the mod's file
                    let res = client.get(format!("https://mods.factorio.com{download_url}?username={factorio_username}&token={factorio_token}"))
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

                    let body = res.bytes().await?;
                    // Write the file to disk
                    let mut file = tokio::fs::File::create(file_name).await?;
                    file.write_all(&body).await?;
                }
            }
        }
    }

    Ok(())
}
