use async_trait::async_trait;
use reqwest::Client;

use crate::postcode::{PostcodeError, PostcodeProvider, ResolvedAddress, normalize_postcode};

pub struct NipostProvider {
    client: Client,
    base_url: String,
    api_key: String,
}

impl NipostProvider {
    pub fn new(base_url: String, api_key: String) -> Result<Self, PostcodeError> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| PostcodeError::Unavailable(e.to_string()))?;

        Ok(Self {
            client,
            base_url,
            api_key,
        })
    }
}

#[async_trait]
impl PostcodeProvider for NipostProvider {
    async fn resolve(&self, postcode: &str) -> Result<ResolvedAddress, PostcodeError> {
        let _code = normalize_postcode(postcode)?;

        // TODO:
        // Confirm the exact query parameter, authentication
        // header, response schema and lookup level using
        // the official API documentation and staging key.
        //
        // Then implement the HTTP request and map the
        // returned response to ResolvedAddress.

        let _ = (&self.client, &self.base_url, &self.api_key);

        Err(PostcodeError::Unavailable(
            "Live NIPOST adapter not configured yet".into(),
        ))
    }
}
