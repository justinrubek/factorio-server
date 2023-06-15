use serde::{Deserialize, Serialize};

/// The response returned by the Factorio Mod Portal API when logging in using the Web
/// Authentication API.
/// The format of the request is taken from https://wiki.factorio.com/Web_authentication_API
#[derive(Debug, Deserialize, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub username: String,
}

/// An error returned when calling the Factorio Mod Portal API.
#[derive(Debug, Deserialize, Serialize)]
pub struct ErrorResponse {
    error: String,
    message: String,
}

/// The response returned by the Factorio Mod Portal API when requesting a mod's info.
#[derive(Debug, Deserialize, Serialize)]
pub struct ModResponse {
    /// Number of downloads.
    pub downloads_count: u64,
    /// The latest version of the mod available for download.
    pub latest_release: Option<ModRelease>,
    /// The mod's machine-readable ID string.
    pub name: String,
    /// The Factorio username of the mod's author.
    pub owner: String,
    /// A list of different versions of the mod available for download.
    pub releases: Vec<ModRelease>,
    /// A shorter mod description.
    pub summary: String,
    /// The mod's human-readable name.
    pub title: String,
    /// A simple tag describing the mod.
    pub category: Option<String>,
    /// The relative path to the thumbnail of the mod. For mods that have no thumbnail it may be
    /// absent or default to "/assets/.thumb.png". Prepend "assets-mod.factorio.com".
    pub thumbnail: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ModRelease {
    /// Path to download for a mod. Starts with "/download" and does not include a full URL.
    /// Use "https://mods.factorio.com" as a prefix.
    /// The route requires authentication.
    pub download_url: String,
    /// The file name of the release. Always seems to follow the pattern "{name}_{version}.zip".
    pub file_name: String,
    /// A copy of the mod's info.json file. Only contains factorio_version in the short version.
    /// The full version also contains an array of dependencies.
    // pub info_json: ModInfo,

    /// ISO 6501 date string of the release.
    pub released_at: String,
    /// The version of the release.
    pub version: String,
    /// The sha1 key for the release's file.
    pub sha1: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ModInfo {
    /// The machine-readable name of the mod
    pub name: String,
    /// The version of the mod
    pub version: String,
    /// The human-readable name of the mod
    pub title: String,
    /// The author of the mod
    pub author: String,
    /// The version of Factorio the mod is compatible with
    pub factorio_version: String,
    /// A list of mods that this mod depends on
    pub dependencies: Option<Vec<String>>,
}
