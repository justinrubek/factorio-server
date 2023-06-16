use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;

#[derive(Serialize, Deserialize, Debug, clap::Args)]
pub(crate) struct ModDetails {
    pub name: String,
    pub version: String,
}

/// Retrieves the download_url and file_name for a mod.
pub(crate) async fn retrieve_mod_release(
    client: &reqwest::Client,
    ModDetails { name, version }: &ModDetails,
) -> Result<factorio_api::ModRelease> {
    let res = client
        .get(format!("https://mods.factorio.com/api/mods/{name}"))
        .send()
        .await?;

    match res.status() {
        reqwest::StatusCode::OK => (),
        _ => {
            let body = res.text().await?;
            tracing::error!("{:#?}", body);
            return Err(Error::FactorioApi(body));
        }
    }

    let body = res.json::<factorio_api::ModResponse>().await?;

    // Find a release that matches the version
    let release = body
        .releases
        .iter()
        .find(|release| release.version.eq(version))
        .cloned()
        .ok_or_else(|| Error::NoMatchingRelease(name.clone(), version.clone()))?;

    Ok(release)
}

// Downloads the mod to a given path.
pub async fn retrieve_mod_file(
    client: &reqwest::Client,
    download_url: &str,
    file_path: &std::path::Path,
    username: &str,
    token: &str,
) -> Result<()> {
    // Retrieve the mod's file
    let res = client
        .get(format!(
            "https://mods.factorio.com{download_url}?username={username}&token={token}"
        ))
        .send()
        .await?;

    match res.status() {
        reqwest::StatusCode::OK => (),
        _ => {
            let body = res.text().await?;
            tracing::error!("{:#?}", body);
            return Ok(());
        }
    }

    let body = res.bytes().await?;
    // Write the file to disk
    let mut file = tokio::fs::File::create(file_path).await?;
    file.write_all(&body).await?;

    Ok(())
}

/// Retrieves Factorio authentication details from the environment.
pub fn retrieve_factorio_auth() -> (String, String) {
    let factorio_username = std::env::var("FACTORIO_USERNAME").expect("FACTORIO_USERNAME not set");
    let factorio_token = std::env::var("FACTORIO_TOKEN").expect("FACTORIO_TOKEN not set");

    (factorio_username, factorio_token)
}
