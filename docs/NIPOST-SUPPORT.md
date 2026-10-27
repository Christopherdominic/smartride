# NIPOST sandbox support report — 27 October 2026

The official public API returns a negative validity result for a documented sandbox sample. The failure is reproducible without SmartRide. No real `valid:true` result has been obtained.

## Request and evidence

- Observed at: `2026-10-27T10:26:53.907244+00:00`.
- Environment: sandbox secret key (prefix classified privately; secret omitted).
- Origin: `https://api.postcode.gov.ng`.
- Authentication: `X-API-Key: <REDACTED>`; `Accept: application/json`; `User-Agent: SmartRide/0.3`.
- Request: `GET /v1/lookup?code=FC-01-A01-KP-27&level=1`.
- HTTP 200, 774 ms; client-supplied `X-Request-ID: 197e5d03-1d89-42b2-b2af-4dfe3d0c069a`.
- No upstream request/correlation ID was returned on this direct request. The client ID is not confirmed to have been retained by NIPOST.

Observed lookup JSON:

```json
{"data":{"postcode":"FC-01-A01-KP-27","valid":false,"status":"not_found","verified":false}}
```

The [official authentication documentation](https://docs.postcode.gov.ng/authentication) lists this sample as resolving under a Test key. Direct `KT-14-B62-EB-16` lookup also returns HTTP 200 with the same false/not_found outcome; the account holder reports a Valid dashboard result with the same Test key.

Direct autocomplete requests for broad prefixes `FC` and `KT` both return HTTP 200:

```json
{"data":{"segment":"state","suggestions":[]}}
```

Removing authentication returns HTTP 401 / `auth_required`. Authenticated search returns no 403 or scope error. This establishes no observed search-scope denial; the key's actual granted scopes cannot be independently enumerated with the documented endpoints tested.

The [redacted machine-readable evidence](nipost-support-evidence.json) contains status, duration, generated request IDs, safe response fields, and the local/deployed comparison. It contains no key, fingerprint, customer data, or account credentials.

## SmartRide comparison and failure layer

Render privately confirms `POSTCODE_PROVIDER=nipost`, the official base URL, and the same key as the ignored local backend `.env`. `/api/v1/demo` publicly confirms `nipost`, `sandbox`, L1, and a separately required customer address. The local `.env` deliberately retains `POSTCODE_PROVIDER=mock` for offline development; direct requests and deployed tests bypass that setting.

SmartRide's Rust provider sends `X-API-Key`, requests only `level=1`, canonicalizes the postcode, and requires a matching `data.postcode` and boolean `data.valid`. The upstream false result maps to SmartRide HTTP 404 / `postcode_not_found`; direct and proxied autocomplete results both have zero suggestions. The current observed failure is in NIPOST's public API result, before SmartRide parsing. Whether this is sandbox data, key-to-organization association, routing, or another upstream problem is unconfirmed.

One separate application defect was found and corrected: **upstream HTTP 404** previously became postcode-not-found. It now becomes HTTP 502 / `postcode_unavailable`; only a successful matching validity envelope establishes invalidity. Existing 401/402/403/429/5xx handling remains distinct. This correction does not fix the real HTTP 200 / false result.

## Request to NIPOST support

Please check why this organization's Test key returns negative L1 validity for the documented sandbox sample and empty broad-prefix autocomplete, and reconcile the dashboard result for `KT-14-B62-EB-16` with the public API. Confirm the sandbox dataset/key association and search scope. No higher lookup level is requested. Please provide a Test-key sample and request trace that returns `data.valid:true` through these same public endpoints.

This report is prepared for the account holder; it has not been sent to third parties.

## Diagnostics and regression validation

Provider diagnostics record only the fixed endpoint path, upstream HTTP status, allowlisted error code, elapsed milliseconds, generated UUID request ID, and a UUID-shaped upstream request/correlation ID if supplied. Unrecognized codes are replaced by `unrecognized_upstream_error`; raw messages, arbitrary header values, URLs/query values, postcodes, bodies, keys, and customer information are excluded. Diagnostics use the existing `info` log level; no new environment variables are needed.

Rust regressions cover valid L1 fixtures without address assumptions, false validity, malformed/mismatched bodies and inputs, missing keys, 401/402/403/404/429/500, non-JSON HTTP errors, timeout/network failure, request headers/L1/correlation, autocomplete bounds, and captured diagnostic redaction. The full local-fixture delivery/rider/OTP integration passed. Positive fixture tests are not real NIPOST verification. Live-key testing was skipped because the configured key is a Test key. Delivery verification rules and mock-fallback behavior are unchanged.

## Verified rollout

Commit `81c43e3` is live on Render as deployment `dep-dbg7pcbbc2fs739uv310`. Post-deployment public API checks passed for configuration, invalid/malformed postcode handling, empty autocomplete, and forged-verification rejection without persisting a delivery. Five filtered hosted diagnostic events were inspected: upstream HTTP 200, durations 29–209 ms, generated UUID request IDs, and no secret or tested postcode in those events. This verifies diagnostics and rejection behavior, not a positive NIPOST lookup.

All 12 Rust tests passed. The final production Docker image built successfully and passed fresh migrations, mock delivery/rider/OTP regression, production read/write denial, CORS, and rate-limit checks. The full L1 HTTP-fixture integration also passed. No frontend or verification-rule changes were made during this investigation.
