use async_trait::async_trait;
use serde_json::json;

use crate::postcode::{PostcodeError, PostcodeProvider, ResolvedAddress, normalize_postcode};

pub struct MockPostcodeProvider;

#[async_trait]
impl PostcodeProvider for MockPostcodeProvider {
    async fn resolve(&self, postcode: &str) -> Result<ResolvedAddress, PostcodeError> {
        let code = normalize_postcode(postcode)?;

        let address = match code.as_str() {
            "EK01A03FK01" => "Demo Building, Test District, Nigeria",
            "LA01A03IK01" => "Fictional Market House, Demo Lagos District, Nigeria",
            "AB01A03FC01" => "Fictional Unity House, Demo Abuja District, Nigeria",
            _ => return Err(PostcodeError::NotFound),
        };

        Ok(ResolvedAddress {
            postcode: code,
            formatted_address: Some(address.into()),
            provider: "mock".into(),
            provider_data: json!({
                "mock": true,
                "verified": false
            }),
        })
    }
}
