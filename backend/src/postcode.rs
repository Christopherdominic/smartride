use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedAddress {
    pub postcode: String,
    pub formatted_address: Option<String>,
    pub provider: String,
    pub provider_data: Value,
}

#[derive(Debug, thiserror::Error)]
pub enum PostcodeError {
    #[error("invalid postcode")]
    InvalidPostcode,

    #[error("postcode not found")]
    NotFound,

    #[error("provider unavailable: {0}")]
    Unavailable(String),
}

#[async_trait]
pub trait PostcodeProvider: Send + Sync {
    async fn resolve(&self, postcode: &str) -> Result<ResolvedAddress, PostcodeError>;
}

pub fn normalize_postcode(input: &str) -> Result<String, PostcodeError> {
    let code: String = input
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-')
        .collect::<String>()
        .to_ascii_uppercase();

    let bytes = code.as_bytes();

    let valid = bytes.len() == 11
        && bytes[0..2].iter().all(u8::is_ascii_alphabetic)
        && bytes[2..4].iter().all(u8::is_ascii_digit)
        && bytes[4..7].iter().all(u8::is_ascii_alphanumeric)
        && bytes[7..9].iter().all(u8::is_ascii_alphabetic)
        && bytes[9..11].iter().all(u8::is_ascii_digit);

    if valid {
        Ok(code)
    } else {
        Err(PostcodeError::InvalidPostcode)
    }
}
