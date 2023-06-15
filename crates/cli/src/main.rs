use crate::{
    commands::{AuthCommands, Commands, LoginRequest},
    error::Result,
};
use clap::Parser;

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
    }

    Ok(())
}
