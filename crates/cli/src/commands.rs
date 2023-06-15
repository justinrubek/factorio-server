use serde::{Deserialize, Serialize};

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Args {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(clap::Subcommand, Debug)]
pub(crate) enum Commands {
    Auth(Auth),
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
