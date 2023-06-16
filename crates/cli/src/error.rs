#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    RonSpanned(#[from] ron::de::SpannedError),

    #[error("Error logging in: {0:?}")]
    FactorioLogin(factorio_api::auth::LoginError),
    #[error("Error retrieving mod release info: {0:?}")]
    FactorioApi(String),

    #[error("No matching release found for mod {0} with version {1}")]
    NoMatchingRelease(String, String),
}

pub type Result<T> = std::result::Result<T, Error>;
