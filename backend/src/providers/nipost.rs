use crate::postcode::{
    Autocomplete, PostcodeError, PostcodeProvider, ResolvedAddress, normalize_postcode,
    validate_query,
};
use async_trait::async_trait;
use reqwest::{
    Client, Url,
    header::{HeaderMap, HeaderValue},
};
use serde::Deserialize;
use serde_json::json;
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub struct NipostProvider {
    client: Client,
    base_url: Url,
    environment: &'static str,
    budget: Mutex<(Instant, u32)>,
}
#[derive(Deserialize)]
struct Envelope<T> {
    data: T,
}
#[derive(Deserialize)]
struct Validity {
    postcode: String,
    valid: bool,
}
#[derive(Deserialize)]
struct ErrorEnvelope {
    error: ErrorData,
}
#[derive(Deserialize)]
struct ErrorData {
    code: String,
}

// Never record URLs, query values, credentials, messages, or response bodies.
struct RequestDiagnostic {
    started: Instant,
    request_id: uuid::Uuid,
    upstream_request_id: Option<uuid::Uuid>,
    status: Option<u16>,
    error_code: &'static str,
    endpoint: &'static str,
}
impl Drop for RequestDiagnostic {
    fn drop(&mut self) {
        tracing::info!(
            target: "backend::nipost",
            endpoint = self.endpoint,
            upstream_http_status = ?self.status,
            nipost_error_code = self.error_code,
            duration_ms = self.started.elapsed().as_millis() as u64,
            request_id = %self.request_id,
            upstream_request_id = ?self.upstream_request_id,
            "NIPOST request completed"
        );
    }
}
fn diagnostic_error_code(code: &str) -> &'static str {
    // An arbitrary upstream string could contain customer information or secrets.
    match code {
        "auth_required" => "auth_required",
        "invalid_api_key" => "invalid_api_key",
        "insufficient_credits" => "insufficient_credits",
        "level_not_granted" => "level_not_granted",
        "scope_not_granted" => "scope_not_granted",
        "insufficient_scope" => "insufficient_scope",
        "rate_limited" => "rate_limited",
        "not_found" => "not_found",
        _ => "unrecognized_upstream_error",
    }
}

impl NipostProvider {
    pub fn new(base_url: String, api_key: String) -> Result<Self, PostcodeError> {
        Self::build(base_url, api_key, Duration::from_secs(10))
    }
    fn build(base_url: String, api_key: String, timeout: Duration) -> Result<Self, PostcodeError> {
        let url = Url::parse(&base_url)
            .map_err(|_| PostcodeError::Unavailable("invalid NIPOST base URL".into()))?;
        let official = url.as_str().trim_end_matches('/') == "https://api.postcode.gov.ng";
        // Debug-only loopback fixture support; release builds only send secrets to NIPOST.
        let fixture = (cfg!(debug_assertions) || cfg!(test))
            && url.scheme() == "http"
            && matches!(url.host_str(), Some("127.0.0.1" | "localhost" | "[::1]"));
        if (!official && !fixture)
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
            || url.path() != "/"
        {
            return Err(PostcodeError::Unavailable(
                "NIPOST base URL must be https://api.postcode.gov.ng".into(),
            ));
        }
        if api_key.trim().is_empty() {
            return Err(PostcodeError::AuthenticationFailed);
        }
        let environment = if api_key.starts_with("nipost_test_") {
            "sandbox"
        } else if api_key.starts_with("nipost_live_") {
            "production"
        } else {
            "unknown"
        };
        let mut key =
            HeaderValue::from_str(&api_key).map_err(|_| PostcodeError::AuthenticationFailed)?;
        key.set_sensitive(true);
        let mut headers = HeaderMap::new();
        headers.insert("X-API-Key", key);
        headers.insert("Accept", HeaderValue::from_static("application/json"));
        let client = Client::builder()
            .user_agent("SmartRide/0.3")
            .default_headers(headers)
            .timeout(timeout)
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|_| PostcodeError::Unavailable("could not initialize NIPOST client".into()))?;
        Ok(Self {
            client,
            base_url: url,
            environment,
            budget: Mutex::new((Instant::now(), 0)),
        })
    }
    async fn request<T: serde::de::DeserializeOwned>(
        &self,
        path: &'static str,
        query: &[(&str, &str)],
    ) -> Result<T, PostcodeError> {
        {
            let mut budget = self
                .budget
                .lock()
                .map_err(|_| PostcodeError::Unavailable("request limiter unavailable".into()))?;
            if budget.0.elapsed() >= Duration::from_secs(60) {
                *budget = (Instant::now(), 0);
            }
            if budget.1 >= 60 {
                return Err(PostcodeError::RateLimited);
            }
            budget.1 += 1;
        }
        let url = self
            .base_url
            .join(path)
            .map_err(|_| PostcodeError::Unavailable("invalid NIPOST route".into()))?;
        let transport = |e: reqwest::Error| {
            if e.is_timeout() {
                PostcodeError::Timeout
            } else {
                PostcodeError::Unavailable("NIPOST connection failed".into())
            }
        };
        let mut diagnostic = RequestDiagnostic {
            started: Instant::now(),
            request_id: uuid::Uuid::new_v4(),
            upstream_request_id: None,
            status: None,
            error_code: "transport_or_body_failure",
            endpoint: path,
        };
        let mut response = self
            .client
            .get(url)
            .query(query)
            .header("X-Request-ID", diagnostic.request_id.to_string())
            .send()
            .await
            .map_err(transport)?;
        let status = response.status().as_u16();
        diagnostic.status = Some(status);
        diagnostic.upstream_request_id = ["x-request-id", "x-correlation-id"]
            .iter()
            .filter_map(|name| response.headers().get(*name)?.to_str().ok())
            .find_map(|value| uuid::Uuid::parse_str(value).ok());
        diagnostic.error_code = "malformed_response";
        if response.content_length().is_some_and(|n| n > 65536) {
            return Err(PostcodeError::MalformedResponse);
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(transport)? {
            if bytes.len() + chunk.len() > 65536 {
                return Err(PostcodeError::MalformedResponse);
            }
            bytes.extend_from_slice(&chunk);
        }
        if !(200..300).contains(&status) {
            // Parse the official error envelope but never relay upstream text or credentials.
            if let Ok(error) = serde_json::from_slice::<ErrorEnvelope>(&bytes) {
                diagnostic.error_code = diagnostic_error_code(&error.error.code);
            } else {
                diagnostic.error_code = "http_error_without_envelope";
            }
            return Err(match status {
                401 => PostcodeError::AuthenticationFailed,
                402 => PostcodeError::InsufficientCredits,
                403 => PostcodeError::AccessDenied,
                // An HTTP 404 may mean a missing API route, not an invalid postcode.
                // Only a successful validity envelope can establish invalidity.
                429 => PostcodeError::RateLimited,
                _ => PostcodeError::Unavailable("NIPOST request failed".into()),
            });
        }
        let result = serde_json::from_slice(&bytes).map_err(|_| PostcodeError::MalformedResponse);
        if result.is_ok() {
            diagnostic.error_code = "none";
        }
        result
    }
}
#[async_trait]
impl PostcodeProvider for NipostProvider {
    async fn resolve(&self, postcode: &str) -> Result<ResolvedAddress, PostcodeError> {
        let code = normalize_postcode(postcode)?;
        let response: Envelope<Validity> = self
            .request("/v1/lookup", &[("code", &code), ("level", "1")])
            .await?;
        let canonical = normalize_postcode(&response.data.postcode)
            .map_err(|_| PostcodeError::MalformedResponse)?;
        if canonical != code {
            return Err(PostcodeError::MalformedResponse);
        }
        if !response.data.valid {
            return Err(PostcodeError::NotFound);
        }
        Ok(ResolvedAddress {
            postcode: canonical,
            formatted_address: None,
            provider: "nipost".into(),
            provider_data: json!({"valid":true,"verified":true,"lookup_level":1,"environment":self.environment}),
        })
    }
    async fn autocomplete(&self, query: &str) -> Result<Autocomplete, PostcodeError> {
        let query = validate_query(query)?;
        let response: Envelope<Autocomplete> = self
            .request("/v1/search/autocomplete", &[("q", &query)])
            .await?;
        let mut data = response.data;
        if !matches!(
            data.segment.as_str(),
            "state" | "lga" | "district" | "area" | "unit"
        ) || data.suggestions.iter().any(|s| {
            s.code.len() > 30
                || s.label.len() > 200
                || !s
                    .code
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b' ')
        }) {
            return Err(PostcodeError::MalformedResponse);
        }
        data.suggestions.truncate(10);
        Ok(data)
    }
    fn name(&self) -> &'static str {
        "nipost"
    }
    fn environment(&self) -> &'static str {
        self.environment
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        Router,
        extract::Query,
        http::{HeaderMap, StatusCode},
        response::IntoResponse,
        routing::get,
    };
    use std::collections::HashMap;
    async fn fixture(
        status: u16,
        body: &str,
        delay: Duration,
    ) -> (String, tokio::task::JoinHandle<()>) {
        let content = body.to_string();
        let app = Router::new().route(
            "/v1/lookup",
            get(
                move |headers: HeaderMap, Query(q): Query<HashMap<String, String>>| {
                    let content = content.clone();
                    async move {
                        assert_eq!(headers["x-api-key"], "nipost_test_fixture");
                        assert_eq!(headers["user-agent"], "SmartRide/0.3");
                        assert!(
                            uuid::Uuid::parse_str(headers["x-request-id"].to_str().unwrap())
                                .is_ok()
                        );
                        assert_eq!(q.get("level").unwrap(), "1");
                        tokio::time::sleep(delay).await;
                        (
                            StatusCode::from_u16(status).unwrap(),
                            [("x-request-id", "12345678-1234-4234-8234-123456789abc")],
                            content,
                        )
                            .into_response()
                    }
                },
            ),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        (url, task)
    }
    #[tokio::test]
    async fn l1_validity_only_and_no_address_assumptions() {
        let (url,task)=fixture(200,r#"{"data":{"postcode":"FC-01-A01-KP-27","valid":true,"recent_house_address":{"recent":"MUST NOT BE USED"}}}"#,Duration::ZERO).await;
        let p = NipostProvider::new(url, "nipost_test_fixture".into()).unwrap();
        let a = p.resolve("fc01a01kp27").await.unwrap();
        assert!(a.formatted_address.is_none());
        assert_eq!(a.provider_data["lookup_level"], 1);
        assert_eq!(a.provider_data["verified"], true);
        assert!(!a.provider_data.to_string().contains("MUST NOT BE USED"));
        task.abort();
    }
    #[tokio::test]
    async fn invalid_and_malformed_responses_fail_closed() {
        for body in [
            r#"{"data":{"postcode":"FC-01-A01-KP-27","valid":false}}"#,
            r#"{"data":{"valid":true}}"#,
            r#"{"data":{"postcode":"FC-01-A01-KP-27","valid":"true"}}"#,
            r#"{"data":{"postcode":"FC-01-A01-LR-01","valid":true}}"#,
            "not json",
        ] {
            let (url, task) = fixture(200, body, Duration::ZERO).await;
            let p = NipostProvider::new(url, "nipost_test_fixture".into()).unwrap();
            let result = p.resolve("FC-01-A01-KP-27").await;
            assert!(result.is_err());
            task.abort();
        }
    }
    #[tokio::test]
    async fn upstream_statuses_and_timeouts() {
        for (status, expected) in [
            (401, "credentials"),
            (402, "credits"),
            (403, "access denied"),
            (429, "limit"),
            (404, "unavailable"),
            (500, "unavailable"),
        ] {
            let (url, task) = fixture(
                status,
                r#"{"error":{"code":"upstream_code","message":"DO NOT RELAY"}}"#,
                Duration::ZERO,
            )
            .await;
            let p = NipostProvider::new(url, "nipost_test_fixture".into()).unwrap();
            let e = p.resolve("FC-01-A01-KP-27").await.unwrap_err().to_string();
            assert!(e.contains(expected), "{e}");
            assert!(!e.contains("DO NOT RELAY"));
            task.abort();
        }
        let (url, task) = fixture(200, "{}", Duration::from_secs(1)).await;
        let p = NipostProvider::build(url, "nipost_test_fixture".into(), Duration::from_millis(30))
            .unwrap();
        assert!(matches!(
            p.resolve("FC-01-A01-KP-27").await,
            Err(PostcodeError::Timeout)
        ));
        task.abort();
    }
    #[tokio::test]
    async fn negative_validity_is_distinct_from_http_errors() {
        let (url, task) = fixture(
            200,
            r#"{"data":{"postcode":"FC-01-A01-KP-27","valid":false,"status":"not_found"}}"#,
            Duration::ZERO,
        )
        .await;
        let p = NipostProvider::new(url, "nipost_test_fixture".into()).unwrap();
        assert!(matches!(
            p.resolve("FC-01-A01-KP-27").await,
            Err(PostcodeError::NotFound)
        ));
        assert!(matches!(
            p.resolve("malformed").await,
            Err(PostcodeError::InvalidPostcode)
        ));
        assert_eq!(p.budget.lock().unwrap().1, 1);
        task.abort();
        for body in [
            r#"{"error":{"code":"not_found","message":"DO NOT LOG"}}"#,
            "not json",
        ] {
            let (url, task) = fixture(404, body, Duration::ZERO).await;
            let p = NipostProvider::new(url, "nipost_test_fixture".into()).unwrap();
            assert!(matches!(
                p.resolve("FC-01-A01-KP-27").await,
                Err(PostcodeError::Unavailable(_))
            ));
            task.abort();
        }
    }
    #[test]
    fn diagnostic_codes_never_echo_arbitrary_upstream_content() {
        assert_eq!(diagnostic_error_code("auth_required"), "auth_required");
        assert_eq!(
            diagnostic_error_code("nipost_test_secret customer address"),
            "unrecognized_upstream_error"
        );
    }
    #[tokio::test]
    async fn diagnostics_include_metadata_without_sensitive_data() {
        use std::io::Write;
        use std::sync::{Arc, Mutex};
        #[derive(Clone)]
        struct Capture(Arc<Mutex<Vec<u8>>>);
        impl Write for Capture {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                self.0.lock().unwrap().extend_from_slice(bytes);
                Ok(bytes.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let capture = Capture(Arc::new(Mutex::new(Vec::new())));
        let writer = capture.clone();
        let subscriber = tracing_subscriber::fmt()
            .without_time()
            .with_ansi(false)
            .with_writer(move || writer.clone())
            .finish();
        // A global subscriber observes events reliably across parallel Tokio tests.
        tracing::subscriber::set_global_default(subscriber).unwrap();
        let (url, task) = fixture(
            403,
            r#"{"error":{"code":"insufficient_scope","message":"PRIVATE CUSTOMER ADDRESS"}}"#,
            Duration::ZERO,
        )
        .await;
        let p = NipostProvider::new(url, "nipost_test_fixture".into()).unwrap();
        assert!(matches!(
            p.resolve("FC-01-A01-KP-27").await,
            Err(PostcodeError::AccessDenied)
        ));
        let log = String::from_utf8(capture.0.lock().unwrap().clone()).unwrap();
        for expected in [
            "upstream_http_status=Some(403)",
            "insufficient_scope",
            "duration_ms=",
            "12345678-1234-4234-8234-123456789abc",
            "request_id=",
            "/v1/lookup",
        ] {
            assert!(
                log.contains(expected),
                "missing diagnostic field: {expected}; log={log:?}"
            );
        }
        for secret in [
            "nipost_test_fixture",
            "PRIVATE CUSTOMER ADDRESS",
            "FC-01-A01-KP-27",
            "level=1",
        ] {
            assert!(!log.contains(secret), "diagnostic leaked sensitive input");
        }
        task.abort();
    }
    #[tokio::test]
    async fn missing_key_network_failure_and_local_rate_limit() {
        assert!(matches!(
            NipostProvider::new("https://api.postcode.gov.ng".into(), "".into()),
            Err(PostcodeError::AuthenticationFailed)
        ));
        assert!(
            NipostProvider::new("https://evil.example".into(), "nipost_test_fixture".into())
                .is_err()
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        drop(listener);
        let p = NipostProvider::new(url, "nipost_test_fixture".into()).unwrap();
        assert!(matches!(
            p.resolve("FC-01-A01-KP-27").await,
            Err(PostcodeError::Unavailable(_))
        ));
        p.budget.lock().unwrap().1 = 60;
        assert!(matches!(
            p.resolve("FC-01-A01-KP-27").await,
            Err(PostcodeError::RateLimited)
        ));
    }
}

#[cfg(test)]
mod search_tests {
    use super::*;
    use axum::{Router, extract::Query, http::HeaderMap, routing::get};
    use std::collections::HashMap;
    #[tokio::test]
    async fn official_search_envelope_and_bounds() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let app=Router::new().route("/v1/search/autocomplete",get(|headers:HeaderMap,Query(q):Query<HashMap<String,String>>|async move{
            assert_eq!(headers["x-api-key"],"nipost_test_fixture");assert_eq!(q.get("q").unwrap(),"FC 01 A01");
            axum::Json(json!({"data":{"segment":"area","suggestions":[{"code":"KP","label":"KP"}]}}))
        }));
        let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let p = NipostProvider::new(url, "nipost_test_fixture".into()).unwrap();
        let result = p.autocomplete("fc 01 a01").await.unwrap();
        assert_eq!(result.suggestions[0].code, "KP");
        assert!(matches!(
            p.autocomplete("a").await,
            Err(PostcodeError::InvalidQuery)
        ));
        assert_eq!(p.budget.lock().unwrap().1, 1);
        task.abort();
    }
}
