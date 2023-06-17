use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::mods::ModDetails;

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Args {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(clap::Subcommand, Debug)]
pub(crate) enum Commands {
    Auth(Auth),
    Download(Download),
    Server(Server),
}

#[derive(clap::Args, Debug)]
pub(crate) struct Auth {
    #[clap(subcommand)]
    pub command: AuthCommands,
}

#[derive(clap::Subcommand, Debug)]
pub(crate) enum AuthCommands {
    Login(LoginRequest),
}

#[derive(Serialize, Deserialize, Debug, clap::Args)]
pub(crate) struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(clap::Args, Debug)]
pub(crate) struct Download {
    #[clap(subcommand)]
    pub command: DownloadCommands,
}

#[derive(clap::Subcommand, Debug)]
pub(crate) enum DownloadCommands {
    /// Downloads a single mod to the current directory
    SingleMod(ModDetails),
    /// Downloads mods from a mod list file
    ModList(ModList),
}

#[derive(clap::Args, Debug)]
pub(crate) struct ModList {
    /// The path to the mod list file
    pub file: PathBuf,
    /// The path to the directory to download the mods to
    #[clap(long, short, default_value = ".")]
    pub directory: PathBuf,
}

#[derive(clap::Args, Debug)]
pub(crate) struct Server {
    #[clap(subcommand)]
    pub command: ServerCommands,
}

#[derive(clap::Subcommand, Debug)]
pub(crate) enum ServerCommands {
    /// Starts the game server
    Start(Start),
}

#[derive(clap::Args, Debug)]
pub(crate) struct Start {
    /// The executable to run
    #[clap(long, short = 'e', default_value = "factorio")]
    pub executable: String,
    /// The path to the directory that the mods will be downloaded to
    #[clap(long, short = 'd')]
    pub mod_directory: PathBuf,
    /// The path to the mod list file
    #[clap(long, short = 'm')]
    pub mod_list: Option<PathBuf>,
    /// Pass-through arguments to the server
    #[arg()]
    pub args: Vec<String>,
}
