use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use tracing::debug;

#[derive(Serialize, Deserialize, Debug, clap::Args)]
pub(crate) struct ModDetails {
    pub name: String,
    pub version: String,
}

#[derive(Deserialize, Debug)]
struct VintageStoryModResponse {
    r#mod: VintageStoryMod,
}

#[derive(Deserialize, Debug)]
struct VintageStoryMod {
    releases: Vec<VintageStoryModRelease>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct VintageStoryModRelease {
    pub mainfile: String,
    pub filename: String,
    pub modversion: String,
}

/// Retrieves the download URL and file name for a mod
pub(crate) async fn retrieve_mod_release(
    client: &reqwest::Client,
    ModDetails { name, version }: &ModDetails,
) -> Result<VintageStoryModRelease> {
    debug!("Requesting mod info for {} version {}", name, version);

    let url = format!("https://mods.vintagestory.at/api/mod/{}", name);
    debug!("API URL: {}", url);

    let res =
        client.get(&url).send().await.map_err(|e| {
            Error::VintageStoryApi(format!("Request failed for mod {}: {}", name, e))
        })?;

    if !res.status().is_success() {
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        debug!(
            "API response for mod {}: Status={}, Body={}",
            name, status, body
        );
        return Err(Error::VintageStoryApi(format!(
            "API error for mod {} ({}): {}",
            name, status, body
        )));
    }

    let body_text = res.text().await.map_err(|e| {
        Error::VintageStoryApi(format!("Failed to read response for mod {}: {}", name, e))
    })?;

    debug!("Raw API response for mod {}: {}", name, body_text);

    let response = serde_json::from_str::<VintageStoryModResponse>(&body_text).map_err(|e| {
        Error::VintageStoryApi(format!(
            "Failed to parse API response for mod {}: {}. Response was: {}",
            name, e, body_text
        ))
    })?;

    let release = response
        .r#mod
        .releases
        .iter()
        .find(|release| release.modversion == *version)
        .cloned() // Clone the found release instead of moving it
        .ok_or_else(|| {
            let available_versions: Vec<_> = response
                .r#mod
                .releases
                .iter()
                .map(|r| r.modversion.clone())
                .collect();
            Error::NoMatchingRelease(
                name.clone(),
                format!("{} (available versions: {:?})", version, available_versions),
            )
        })?;

    debug!(
        "Found release for {} {}: URL={}",
        name, version, release.mainfile
    );
    Ok(release)
}

/// Downloads the mod to a given path
pub async fn retrieve_mod_file(
    client: &reqwest::Client,
    download_url: &str,
    file_path: &std::path::Path,
) -> Result<()> {
    debug!("Downloading from {}", download_url);

    let res = client
        .get(download_url)
        .send()
        .await
        .map_err(|e| Error::VintageStoryDownload(format!("Download request failed: {}", e)))?;

    if !res.status().is_success() {
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        return Err(Error::VintageStoryDownload(format!(
            "Download failed ({}): {}",
            status, body
        )));
    }

    let body = res.bytes().await.map_err(|e| {
        Error::VintageStoryDownload(format!("Failed to read download content: {}", e))
    })?;

    debug!("Writing mod file to {}", file_path.display());
    let mut file = tokio::fs::File::create(file_path)
        .await
        .map_err(Error::Io)?;
    file.write_all(&body).await.map_err(Error::Io)?;

    Ok(())
}

/// Retrieves all specified mods and downloads them to the specified directory
pub(crate) async fn download_mod_list(
    mod_list: Vec<ModDetails>,
    directory: &std::path::Path,
) -> Result<()> {
    let client = reqwest::Client::new();
    let mut releases = Vec::new();

    for mod_item in &mod_list {
        debug!(
            "Processing mod {} version {}",
            mod_item.name, mod_item.version
        );
        match retrieve_mod_release(&client, mod_item).await {
            Ok(release) => releases.push(release),
            Err(e) => {
                tracing::error!("Failed to process mod {}: {}", mod_item.name, e);
                return Err(e);
            }
        }
    }

    debug!(?releases, "Preparing {} mods", releases.len());
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(Error::Io)?;

    for release in releases {
        let download_url = release.mainfile;
        let file_name = release.filename.clone();
        let file_path = directory.join(&file_name);

        if file_path.exists() {
            tracing::info!("File {} already exists, skipping", file_name);
            continue;
        }

        debug!("Downloading {} from {}", file_name, download_url);

        match retrieve_mod_file(&client, &download_url, &file_path).await {
            Ok(_) => {
                tracing::info!("Successfully downloaded {}", file_name);
            }
            Err(e) => {
                tracing::error!("Failed to download {}: {}", file_name, e);
                return Err(e);
            }
        }
    }

    Ok(())
}
