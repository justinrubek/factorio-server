use crate::{
    commands::{AuthCommands, Commands, DownloadCommands, LoginRequest, ModList},
    error::{Error, Result},
    mods::{retrieve_factorio_auth, retrieve_mod_file, retrieve_mod_release, ModDetails},
};
use clap::Parser;
use commands::ServerCommands;
use mods::download_mod_list;
use nix::{
    sys::signal::{kill, Signal::SIGTERM},
    unistd::Pid,
};
use tokio::{
    process::Command,
    select,
    signal::unix::{signal, SignalKind},
    sync::watch,
};
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
                            let body = res.json::<factorio_api::auth::LoginError>().await?;
                            tracing::error!("{:#?}", body);
                            return Err(Error::FactorioLogin(body));
                        }
                    }

                    let body = res.json::<factorio_api::auth::LoginResponse>().await?;
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

                    download_mod_list(mod_list, &directory).await?;
                }
            }
        }
        Commands::Server(server) => {
            let cmd = server.command;
            match cmd {
                ServerCommands::Start(opts) => {
                    if let Some(mod_list) = &opts.mod_list {
                        // download and install the mods
                        let file = tokio::fs::read_to_string(mod_list).await?;
                        let mod_list = ron::from_str::<Vec<ModDetails>>(&file)?;

                        download_mod_list(mod_list, &opts.mod_directory).await?;
                    }

                    // Prepare signal handlers for graceful shutdown
                    let (stop_tx, mut stop_rx) = watch::channel(());
                    tokio::spawn(async move {
                        let mut sigterm = signal(SignalKind::terminate()).unwrap();
                        let mut sigint = signal(SignalKind::interrupt()).unwrap();

                        loop {
                            select! {
                                _ = sigterm.recv() => println!("Received SIGTERM, shutting down"),
                                _ = sigint.recv() => println!("Received SIGINT, shutting down"),
                            };
                            stop_tx.send(()).unwrap();
                        }
                    });

                    // Now, launch the server
                    // TODO: We need to kill the child process when we receive a signal
                    let mut child = run_factorio_server(&opts).await?;

                    loop {
                        select! {
                            biased;

                            _ = stop_rx.changed() => {
                                println!("Stopping server");
                                if let Some(pid) = child.id() {
                                    if let Err(e) = kill(Pid::from_raw(pid.try_into().expect("Invalid PID")), SIGTERM) {
                                        tracing::error!("Failed to send SIGTERM to child process: {}", e);
                                    }
                                }
                            }
                            status = child.wait() => {
                                match status {
                                    Ok(status) => {
                                        if !status.success() {
                                            println!("Factorio server exited with {}", status);
                                            std::process::exit(status.code().unwrap_or(1));
                                        }
                                        break;
                                    }
                                    Err(e) => {
                                        println!("Factorio server exited with error {}", e);
                                        std::process::exit(1);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

async fn run_factorio_server(opts: &commands::Start) -> Result<tokio::process::Child> {
    let mod_directory = std::fs::canonicalize(&opts.mod_directory)?;

    info!(?mod_directory, ?opts.args, "Starting factorio server");

    let mut child = Command::new(&opts.executable)
        .args([
            "--mod-directory",
            mod_directory.to_str().expect("Invalid mod directory"),
        ])
        .args(opts.args.iter())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    // pass stdout and stderr to the parent process
    let stdout = child.stdout.take().expect("Failed to get stdout");
    let stderr = child.stderr.take().expect("Failed to get stderr");
    let mut stdout = tokio::io::BufReader::new(stdout);
    let mut stderr = tokio::io::BufReader::new(stderr);
    tokio::spawn(async move {
        tokio::io::copy(&mut stdout, &mut tokio::io::stdout())
            .await
            .unwrap();
    });
    tokio::spawn(async move {
        tokio::io::copy(&mut stderr, &mut tokio::io::stderr())
            .await
            .unwrap();
    });

    Ok(child)
}
