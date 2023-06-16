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
pub struct LoginError {
    error: String,
    message: String,
}
