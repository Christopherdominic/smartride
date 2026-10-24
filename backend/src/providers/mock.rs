use async_trait::async_trait;
use serde_json::json;

use crate::postcode::{PostcodeError, PostcodeProvider, ResolvedAddress, normalize_postcode};

pub struct MockPostcodeProvider;

#[async_trait]
impl PostcodeProvider for MockPostcodeProvider {
    async fn resolve(&self, postcode: &str) -> Result<ResolvedAddress, PostcodeError> {
        let code = normalize_postcode(postcode)?;

        if code != "EK01A03FK01" {
            return Err(PostcodeError::NotFound);
        }

        Ok(ResolvedAddress {
            postcode: code,
            formatted_address: Some("Demo Building, Test District, Nigeria".into()),
            provider: "mock".into(),
            provider_data: json!({
                "mock": true,
                "verified": false
            }),
        })
    }
}
