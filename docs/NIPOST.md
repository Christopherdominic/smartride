# NIPOST Level 1 integration

SmartRide makes authenticated server-side requests to the official NIPOST API. Lookup is fixed at **Level 1**. No route, configuration variable, or frontend control requests L2–L5.

## Contract and data provenance

- Base URL: `https://api.postcode.gov.ng`
- Authentication: `X-API-Key`, populated exclusively from backend `NIPOST_API_KEY`
- Verification: `GET /v1/lookup?code=...&level=1`
- Autocomplete: `GET /v1/search/autocomplete?q=...`
- Responses: `data` envelope; lookup requires a boolean `valid` and matching postcode.
- Errors: official `error.code` / `error.message` envelopes are parsed, but raw upstream messages are not forwarded or logged.

L1 verifies **postcode validity only**. It supplies no street address, administrative address, building information, or coordinates. SmartRide discards higher-level fields even if an upstream response happens to include them.

Delivery creation requires `customer_address` in NIPOST mode, with optional `landmark` and existing delivery instructions/contact details. The backend repeats the L1 lookup before creating a delivery; frontend claims such as `postcode_verified: true` cannot bypass verification. The address and landmark are stored in separate columns, labeled customer-provided, and never presented as NIPOST-verified. Verification time and dataset environment are recorded separately. Existing offline addresses and delivery histories are preserved.

Authoritative references: [postcode format](https://docs.postcode.gov.ng/concepts/postcode-format), [authentication and sandbox datasets](https://docs.postcode.gov.ng/authentication), [L1 capabilities](https://docs.postcode.gov.ng/concepts/lookup-levels), [official OpenAPI schema](https://docs.postcode.gov.ng/api-reference/openapi.yaml).

## Input and proxy controls

Canonical display format is `AA-99-H77-BB-55`. Compact and spaced forms are accepted and normalized; one-digit numeric segments are zero-filled. Numeric segments must be `01`–`99`. The segment lengths total 11 characters before separators; the documentation's prose describing 12 characters is inconsistent with its segment table, so validation follows the explicit segment specification.

The backend autocomplete route is `/api/v1/addresses/autocomplete?q=...`. Queries contain 2–30 ASCII letters/digits/spaces/hyphens. Responses expose at most ten validated suggestions for the active segment. The frontend offers a search action, supports segment selection, and requires separate verification of the complete postcode. Search-scope denial is shown without preventing manual entry and lookup.

Lookup, autocomplete, and creation-time re-verification share a **60 upstream requests/minute per-instance** budget, in addition to the existing general API limiter. There are no automatic upstream retries, higher-level requests, or silent mock fallbacks. Requests identify the client as `SmartRide/0.3`, time out after ten seconds, reject redirects, and cap response bodies at 64 KiB. Actual gateway testing showed that requests without a User-Agent receive HTTP 403/non-JSON; the explicit application User-Agent restores HTTP 200 without changing key permissions or lookup level. Release builds only send credentials to the exact official HTTPS origin. Debug builds additionally support loopback HTTP fixtures for automated integration tests; this exception is absent from the release binary.

| Condition | SmartRide HTTP status / machine code |
| --- | --- |
| Malformed postcode | `400` / `invalid_postcode_format` |
| Invalid/not-found postcode | `404` / `postcode_not_found` |
| Upstream `401` | `503` / `nipost_authentication_failed` |
| Upstream `402` | `503` / `nipost_insufficient_credits` |
| Upstream `403` | `403` / `nipost_access_denied` |
| Local or upstream rate limit | `429` / `postcode_rate_limited` |
| Network/upstream failure | `502` / `postcode_unavailable` |
| Malformed/mismatched upstream response | `502` / `invalid_upstream_response` |
| Timeout | `504` / `postcode_timeout` |

Credential failures are server configuration errors, not a request for the browser to supply a key. Error responses preserve `error` as a human-readable string and add a machine-readable `code`.

## Environment configuration

For the dedicated public demo backend:

```text
APP_MODE=demo
POSTCODE_PROVIDER=nipost
NIPOST_API_BASE_URL=https://api.postcode.gov.ng
NIPOST_API_KEY=<configured privately in the Render environment>
FRONTEND_ORIGIN=https://smartride-kappa.vercel.app
```

Preserve the existing `DATABASE_URL`, `PORT`, and other hosting settings. A local `.env` does not configure Render. The key must never enter frontend environment variables, source code, Docker build arguments, or logs. Runtime startup applies the additive customer-address migration automatically.

`APP_MODE` still defaults to `production`, which disables business endpoints pending production authentication. Choosing the NIPOST provider does not remove demo merchant/rider isolation. A sandbox key is labeled as **NIPOST sandbox** in the UI; it is not represented as production data. Offline development remains available with `POSTCODE_PROVIDER=mock`.

Safe rollout order: deploy the additive backend changes while retaining the existing provider, deploy the updated frontend, then configure the private NIPOST key and switch the backend provider. Verify `/api/v1/demo` reports `provider: nipost`, the correct dataset environment and L1, then test a known postcode. No credentials should appear in this endpoint.

## Actual official API results — 25 October 2026

The existing local key was detected as a **sandbox secret key**. No production key was available, and no production-key test or lookup above L1 was performed.

| Real request | Observed result |
| --- | --- |
| `FC-01-A01-KP-27`, L1 | HTTP 200; `valid: false`, `status: not_found`, `verified: false` |
| Compact `FC01A01KP27`, L1 | HTTP 200; `valid: false`, `status: not_found` |
| `FC-01-A01-LR-01`, L1 | HTTP 200; `valid: false`, `status: not_found` |
| `FC-01-A01-MH-01`, L1 | HTTP 200; `valid: false`, `status: not_found` |
| User-provided `KT-14-B62-EB-16`, L1 | HTTP 200; `valid: false`, `status: not_found` |
| Compact and spaced forms of the same KT postcode | Both HTTP 200; `valid: false`, `status: not_found`, including requests with `Cache-Control: no-cache` |
| `ZZ-99-Z99-ZZ-99`, L1 | HTTP 200; `valid: false`, `status: not_found` |
| `not-a-postcode`, L1 | HTTP 200; `valid: false`, `status: invalid`; SmartRide rejects this format locally with 400 |
| Missing API key, L1 | HTTP 401; `error.code: auth_required` |
| Autocomplete prefixes `FC-01`, `FC`, `FC01A01KP`, `KT14B62EB` | HTTP 200; empty `data.suggestions` |

The user reports that the same Test key shows **Valid** for `KT-14-B62-EB-16` in the dashboard. That result could not be reproduced against the official public API. This is an unresolved dashboard/API discrepancy, not proof of a valid API lookup. SmartRide respects the API's negative response and does not create a delivery or show a verified badge from it.

**A successful real sandbox lookup and real NIPOST-backed delivery creation have not passed.** A known postcode that returns `data.valid: true` through this API/key is required to complete those checks. No key rotation, access upgrade, or L2–L5 request is part of this implementation.

## Executed automated checks

- Nine Rust tests passed: canonical formats, numeric ranges, query bounds, authenticated L1-only HTTP requests, validity-only parsing, malformed/mismatched payloads, credential/scope/credit/rate errors, network failures, timeouts, and official autocomplete schema.
- `scripts/nipost-integration.py` passed with a **local HTTP fixture**, dummy key, and fresh disposable PostgreSQL. It verifies customer-address requirements, server re-verification, invalid/unavailable/rate-limited responses, address provenance, and the full rider/OTP workflow in API and Chrome tests.
- The rebuilt release Docker image passed the existing workflow in offline mock mode, fresh migrations and seed idempotence, OTP expiry/replay/attempt/concurrency tests, and production-mode/CORS/rate-limit checks.
- Frontend TypeScript, lint, and production build passed. These results do not turn a fixture response into a successful official API test.

Reproduce the fixture integration after building the debug backend and production frontend for the local API:

```sh
cargo build --locked --manifest-path backend/Cargo.toml
NEXT_PUBLIC_API_BASE_URL=http://localhost:8081 npm run build --prefix frontend
npm run start --prefix frontend
# In another terminal, with ports 8081/8082 free:
VERIFY_BROWSER=1 python3 scripts/nipost-integration.py
```

For a real positive workflow, use a known valid code appropriate to the **configured key's dataset**:

```sh
API_BASE_URL=https://smartride-api-bfrg.onrender.com \
TEST_POSTCODE='<known valid postcode for this key>' \
python3 scripts/smoke.py

FRONTEND_URL=https://smartride-kappa.vercel.app \
TEST_POSTCODE='<known valid postcode for this key>' \
node scripts/browser-smoke.mjs
```

The scripts require explicit `TEST_POSTCODE` in NIPOST mode and never substitute a production sample for a sandbox key. They create sample records only. Current negative real API results must be resolved before claiming these real positive tests have passed.

## Hosting rollout

The frontend and additive backend changes are deployed. Render has `POSTCODE_PROVIDER=nipost`, the official base URL, and the existing sandbox secret configured privately; database/CORS/PORT/APP_MODE settings are preserved. The public configuration endpoint reports `nipost`, `sandbox`, and lookup level 1. The initial deployed default-Reqwest requests hit the gateway's missing-User-Agent 403; the client compatibility fix adds an explicit application identifier. Final proxy/browser checks are recorded in [VALIDATION.md](VALIDATION.md).

The final hosted proxy receives the expected negative validity result after the User-Agent fix (SmartRide maps it to `404/postcode_not_found`). The public Chrome negative-result check passed: no false verified badge, no fictional lookup address, a separate required customer address, and disabled creation. The hosting fix and UI behavior do not resolve the upstream dataset discrepancy.

## Follow-up investigation — 27 October 2026

See the [redacted support report](NIPOST-SUPPORT.md) and [request evidence](nipost-support-evidence.json). Fresh direct and deployed comparisons reproduce the sandbox negative results. Safe upstream diagnostics were added; upstream HTTP 404 now maps to service-unavailable rather than postcode-not-found. No real positive validity result or upstream fix has been verified.
