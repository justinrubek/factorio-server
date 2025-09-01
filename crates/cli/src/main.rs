use crate::{
    commands::{Commands, DownloadCommands, ModList},
    error::Result,
    mods::{download_mod_list, retrieve_mod_file, retrieve_mod_release, ModDetails},
};
use clap::Parser;
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
        Commands::Download(download) => {
            let cmd = download.command;
            match cmd {
                DownloadCommands::SingleMod(details) => {
                    info!(
                        "Downloading mod {} version {}",
                        details.name, details.version
                    );
                    let client = reqwest::Client::new();
                    let release_info = retrieve_mod_release(&client, &details).await?;

                    retrieve_mod_file(
                        &client,
                        &release_info.mainfile,
                        std::path::Path::new(&release_info.filename),
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
                commands::ServerCommands::Start(opts) => {
                    if let Some(mod_list) = &opts.mod_list {
                        let file = tokio::fs::read_to_string(mod_list).await?;
                        let mod_list = ron::from_str::<Vec<ModDetails>>(&file)?;
                        download_mod_list(mod_list, &opts.mod_directory).await?;
                    }

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

                    let mut child = run_vintagestory_server(&opts).await?;

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
                                            println!("Vintage Story server exited with {}", status);
                                            std::process::exit(status.code().unwrap_or(1));
                                        }
                                        break;
                                    }
                                    Err(e) => {
                                        println!("Vintage Story server exited with error {}", e);
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

async fn run_vintagestory_server(opts: &commands::Start) -> Result<tokio::process::Child> {
    let mod_directory = std::fs::canonicalize(&opts.mod_directory)?;

    info!(?mod_directory, ?opts.args, "Starting Vintage Story server");

    let mut child = Command::new(&opts.executable)
        .args([
            "--mod-directory",
            mod_directory.to_str().expect("Invalid mod directory"),
        ])
        .args(opts.args.iter())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

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
