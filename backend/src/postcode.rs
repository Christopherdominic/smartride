use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedAddress {
    pub postcode: String,
    // Only offline fixtures provide a resolved address. L1 never supplies one.
    pub formatted_address: Option<String>,
    pub provider: String,
    pub provider_data: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub code: String,
    pub label: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Autocomplete {
    pub segment: String,
    pub suggestions: Vec<Suggestion>,
}

#[derive(Debug, thiserror::Error)]
pub enum PostcodeError {
    #[error("invalid postcode format; use AA-99-H77-BB-55 with numeric segments 01–99")]
    InvalidPostcode,
    #[error("postcode was not found or is not valid in the configured NIPOST dataset")]
    NotFound,
    #[error("NIPOST rejected the configured API credentials")]
    AuthenticationFailed,
    #[error("NIPOST access denied; check configured key permissions and request restrictions")]
    AccessDenied,
    #[error("NIPOST reports insufficient credits")]
    InsufficientCredits,
    #[error("postcode request limit reached; please try again later")]
    RateLimited,
    #[error("postcode verification timed out; please try again")]
    Timeout,
    #[error("postcode service returned an invalid response")]
    MalformedResponse,
    #[error("invalid search query; enter 2–30 letters, digits, spaces or hyphens")]
    InvalidQuery,
    #[error("provider unavailable: {0}")]
    Unavailable(String),
}

#[async_trait]
pub trait PostcodeProvider: Send + Sync {
    async fn resolve(&self, postcode: &str) -> Result<ResolvedAddress, PostcodeError>;
    async fn autocomplete(&self, _query: &str) -> Result<Autocomplete, PostcodeError> {
        Err(PostcodeError::AccessDenied)
    }
    fn name(&self) -> &'static str {
        "mock"
    }
    fn environment(&self) -> &'static str {
        "offline"
    }
}

// Five explicitly separated segments, or the official compact 11-character form.
// Do not remove arbitrary punctuation or accept misplaced/repeated separators.
pub fn normalize_postcode(input: &str) -> Result<String, PostcodeError> {
    let input = input.trim();
    if input.len() > 30 || !input.is_ascii() {
        return Err(PostcodeError::InvalidPostcode);
    }
    let raw = input.to_ascii_uppercase();
    let parts: Vec<&str> = if raw.contains('-') {
        raw.split('-').collect()
    } else if raw.contains(char::is_whitespace) {
        raw.split_whitespace().collect()
    } else if raw.len() == 11 {
        vec![&raw[..2], &raw[2..4], &raw[4..7], &raw[7..9], &raw[9..]]
    } else {
        return Err(PostcodeError::InvalidPostcode);
    };
    if parts.len() != 5 {
        return Err(PostcodeError::InvalidPostcode);
    }
    let alpha = |s: &str, n: usize| s.len() == n && s.bytes().all(|c| c.is_ascii_uppercase());
    let numeric = |s: &str| {
        (s.len() == 1 || s.len() == 2)
            && s.bytes().all(|c| c.is_ascii_digit())
            && s.parse::<u8>().is_ok_and(|n| (1..=99).contains(&n))
    };
    if !alpha(parts[0], 2)
        || !numeric(parts[1])
        || parts[2].len() != 3
        || !parts[2].bytes().all(|c| c.is_ascii_alphanumeric())
        || !alpha(parts[3], 2)
        || !numeric(parts[4])
    {
        return Err(PostcodeError::InvalidPostcode);
    }
    Ok(format!(
        "{}-{:02}-{}-{}-{:02}",
        parts[0],
        parts[1].parse::<u8>().unwrap(),
        parts[2],
        parts[3],
        parts[4].parse::<u8>().unwrap()
    ))
}
pub fn validate_query(input: &str) -> Result<String, PostcodeError> {
    let query = input.trim();
    if !(2..=30).contains(&query.len())
        || !query
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b' ' || b == b'-')
        || !query.bytes().any(|b| b.is_ascii_alphanumeric())
    {
        return Err(PostcodeError::InvalidQuery);
    }
    Ok(query.to_ascii_uppercase())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn official_formats() {
        for code in [
            "fc-01-a01-kp-27",
            "FC 01 A01 KP 27",
            "FC01A01KP27",
            "FC-1-A01-KP-27",
        ] {
            assert_eq!(normalize_postcode(code).unwrap(), "FC-01-A01-KP-27");
        }
        for code in [
            "FC-00-A01-KP-27",
            "FC-01-A01-KP-00",
            "FC--01-A01-KP-27",
            "F-C01A01KP27",
            "FC-01-AA-KP-27",
            "FC-001-A01-KP-27",
            "éC-01-A01-KP-27",
        ] {
            assert!(normalize_postcode(code).is_err(), "{code}");
        }
    }
    #[test]
    fn bounded_search() {
        assert!(validate_query("FC-01").is_ok());
        for q in ["F", "--", "FC?level=5", "FC\n01", &"A".repeat(31)] {
            assert!(validate_query(q).is_err());
        }
    }
}
