#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    RonSpanned(#[from] ron::de::SpannedError),

    #[error("Error from factorio_api: {0:?}")]
    FactorioApi(factorio_api::ErrorResponse),
    #[error("No matching release found for mod {0} with version {1}")]
    NoMatchingRelease(String, String),
}

pub type Result<T> = std::result::Result<T, Error>;
