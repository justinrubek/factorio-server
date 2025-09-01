// cli/src/errors.rs
use reqwest;
use ron::de::SpannedError;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    RonSpanned(#[from] SpannedError),

    #[error("Error retrieving mod release info from Vintage Story API: {0}")]
    VintageStoryApi(String),
    #[error("Error downloading mod from Vintage Story: {0}")]
    VintageStoryDownload(String),

    #[error("No matching release found for mod {0} with version {1}")]
    NoMatchingRelease(String, String),
}

pub type Result<T> = std::result::Result<T, Error>;
