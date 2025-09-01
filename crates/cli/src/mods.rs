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
    statuscode: String,
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
    let res = client
        .get(format!("https://mods.vintagestory.at/api/mod/{}", name))
        .send()
        .await?;

    if !res.status().is_success() {
        let body = res.text().await?;
        tracing::error!("API error: {}", body);
        return Err(Error::VintageStoryApi(body));
    }

    let body = res.json::<VintageStoryModResponse>().await?;

    let release = body
        .r#mod
        .releases
        .into_iter()
        .find(|release| release.modversion == *version)
        .ok_or_else(|| Error::NoMatchingRelease(name.clone(), version.clone()))?;

    Ok(release)
}

/// Downloads the mod to a given path
pub async fn retrieve_mod_file(
    client: &reqwest::Client,
    download_url: &str,
    file_path: &std::path::Path,
) -> Result<()> {
    let res = client.get(download_url).send().await?;

    if !res.status().is_success() {
        let body = res.text().await?;
        tracing::error!("Download failed: {}", body);
        return Err(Error::VintageStoryDownload(body));
    }

    let body = res.bytes().await?;
    let mut file = tokio::fs::File::create(file_path).await?;
    file.write_all(&body).await?;

    Ok(())
}

/// Retrieves all specified mods and downloads them to the specified directory
pub(crate) async fn download_mod_list(
    mod_list: Vec<ModDetails>,
    directory: &std::path::Path,
) -> Result<()> {
    let client = reqwest::Client::new();
    let mut releases = Vec::new();

    for mod_item in mod_list {
        debug!(
            "Retrieving details for {} version {}",
            mod_item.name, mod_item.version
        );
        let release_info = retrieve_mod_release(&client, &mod_item).await?;
        releases.push(release_info);
    }

    debug!(?releases, "Preparing {} mods", releases.len());
    tokio::fs::create_dir_all(&directory).await?;

    let download_tasks = releases
        .into_iter()
        .map(|release| async {
            let download_url = release.mainfile;
            let file_name = release.filename;

            if std::path::Path::new(&directory).join(&file_name).exists() {
                tracing::info!("File {} already exists, skipping", file_name);
                return Ok(());
            }

            debug!("Downloading {file_name} from {download_url}");

            retrieve_mod_file(
                &client,
                &download_url,
                &std::path::Path::new(&directory).join(&file_name),
            )
            .await
        })
        .collect::<Vec<_>>();

    futures::future::try_join_all(download_tasks).await?;

    Ok(())
}
