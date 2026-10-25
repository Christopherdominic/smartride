# Validation record — updated 25 October 2026

Executed against the existing local PostgreSQL 16 Docker service and the implemented application. Local, container, and public deployment checks are distinguished below. The public deployment was verified on 25 October 2026.

| Check | Result |
| --- | --- |
| `cargo fmt --check` | Passed |
| `cargo check --locked` | Passed |
| `cargo test --locked` | Passed: 2 unit tests covering transition and OTP policies |
| `npm run typecheck` | Passed |
| `npm run lint` | Passed, no warnings after configuration cleanup |
| `npm run build` | Passed: Next.js 15.5.28 production build, all requested routes |
| `scripts/smoke.py` against PostgreSQL | Passed: full persisted workflow, all three fictional postcodes, history, invalid transitions, wrong rider, merchant scope, OTP replay and five-attempt lockout |
| Real OTP expiry/database checks | Passed: freshly created test OTP expired in SQL and rejected; bcrypt hash storage and both applied migrations verified |
| Concurrent OTP verification | Passed: two simultaneous confirmations produce exactly one success and one delivered event |
| Rider registration / failed state | Passed: registered rider starts with no deliveries; failed delivery rejects further progress |
| `scripts/security-smoke.py` | Passed: production denies business reads/writes, JSON error validation, CORS origin restriction, aggregate rate limit, health available after limit |
| `scripts/browser-smoke.mjs` | Passed against production frontend build and Rust API: create, assign, pickup, transit, demo OTP, confirmation, proof and complete timeline; dashboard and rider page data load |
| Responsive review | Chrome screenshots at 1440px desktop and 390px mobile; landing, delivery details, and dashboard checked for horizontal page overflow; screenshots visually inspected |
| Seed idempotence | API restarted repeatedly; deterministic demo merchant/rider/delivery IDs did not conflict or duplicate; existing migrations preserved |
| `git diff --check` | Passed |

The browser test creates real sample records in PostgreSQL. API smoke tests also leave their sample records available for inspection. They do not reset or delete the existing database.

## Incomplete external validation

- **Docker image build (resolved 25 October):** `smartride-api:free-hosting` built successfully using a digest-pinned Rust 1.98.0 builder and Debian Bookworm runtime. Runtime package installation was removed; the binary supplies its own health probe. The exact image passed fresh-database and browser integration tests as described below.
- **Railway:** source upload/deployment received explicit user approval. The CLI sign-in succeeded, but `railway up` then reported: **“Your trial has expired. Please select a plan to continue using Railway.”** No successful deployment or managed database was created/verified by this work. Railway is no longer the deployment target; no paid plan is needed for the selected Render/Neon/Vercel configuration.
- **Vercel (resolved 25 October):** deployed on Hobby; the stable public URL and complete public browser workflow passed. See the live deployment record below.
- **Live NIPOST:** intentionally unconfigured; verified official documentation and credentials are not available in this task. Fictional data remains clearly labeled.
- **Production authentication:** not implemented. Production mode disables all business endpoints; the public demo permits only dedicated sample data.

See [deployment instructions](DEPLOYMENT.md) for the exact Railway and Vercel settings and final rollout checks.

## Free-hosting preparation — 25 October 2026

- Production Docker image built successfully with `docker build --progress=plain -f backend/Dockerfile -t smartride-api:free-hosting .`.
- `VERIFY_BROWSER=1 python3 scripts/verify-container.py` **passed** against that image and a new disposable PostgreSQL 16 database. Existing PostgreSQL data was untouched.
- Verified both migrations on an empty database; one dedicated demo merchant, three seeded riders, and five seeded deliveries; restarting the API did not duplicate seeds.
- Verified UID 10001, a host-overridden port of 10000, the binary's container health probe, and absence of DATABASE_URL from image defaults.
- API, actual Chrome frontend/OTP workflow, concurrent OTP consumption, expiry/replay/attempt tests, CORS, rate limiting, and production read/write denial all passed against the container deployment.
- Rust formatting/check/unit tests, frontend TypeScript/lint/production build, source secret scan, and `git diff --check` passed again. Vercel builds now reject a missing or invalid public HTTPS API origin.
- Render CLI browser authorization succeeded. Render's API-backed Blueprint validator returned `valid: true` for `render.yaml`; its plan contains only the free `smartride-api` service.
- Neon and Vercel authorization subsequently completed; public checks are recorded separately below.

## Verified public deployment — 25 October 2026

| Check | Result |
| --- | --- |
| Hosting plans | Render service `plan=free`; Neon organization `plan=free`; Vercel scope `plan=hobby`; no paid upgrade selected |
| Render deployment | `dep-dbejtd7b9u1s739vkvgg` reported **live** after the final CORS update; application commit `33e42884bf241fc70353eeb14b7687b5b18c682c` |
| Neon | Dedicated `smartride-demo` PostgreSQL 16 project (`floral-band-71506813`), direct TLS with `sslmode=verify-full`, minimum/maximum 0.25 CU |
| Public API readiness | `https://smartride-api-bfrg.onrender.com/health` returned HTTP 200 and `status: ok` |
| Public API smoke | `API_BASE_URL=https://smartride-api-bfrg.onrender.com python3 scripts/smoke.py` passed persisted delivery lifecycle, timeline, OTP attempts/replay/concurrency, scope restrictions, rider registration, failure state, health, and docs |
| Vercel deployment | `dpl_Crnu6t7GaXyYzza3jtkqVB179nWK` reported **READY**, production alias `https://smartride-kappa.vercel.app` |
| Anonymous public frontend | HTTP 200 with SmartRide landing content; production alias requires no Vercel login |
| Deployed CORS | Exact origin `https://smartride-kappa.vercel.app`; untrusted browser origin not permitted |
| Public browser workflow | `FRONTEND_URL=https://smartride-kappa.vercel.app node scripts/browser-smoke.mjs` **passed** postcode → resolve → create → assign → pickup → transit → OTP → delivered, proof confirmation, exact event timeline, dashboard/rider data, and mobile overflow checks |
| Upload secrecy | Vercel dry-run manifest contained 20 frontend source files; all `.env*`, auth state, dependencies, and build caches excluded. Git environment/auth files remain ignored. Render database secret was never printed or committed. |

The public browser test used a fresh Chrome profile and no authentication bypass. The browser harness was updated to wait for loaded riders and enabled controls so it handles remote latency correctly; these were test synchronization fixes, not application workarounds.

Actual OTP expiry and production-mode denial were verified using the exact production image with disposable local PostgreSQL. The public service intentionally runs isolated demo mode; no production authentication or official NIPOST integration is claimed. Free-tier idle suspension and quotas remain applicable.

## NIPOST L1 update — 25 October 2026

- `cargo fmt --check`, `cargo check --locked --offline`, and `cargo test --locked --offline` passed; **9 Rust tests** cover normalization, bounded queries, authenticated L1-only requests, autocomplete parsing, invalid/malformed/mismatched responses, missing credentials, HTTP 401/402/403/429/500, timeout and network errors.
- Frontend TypeScript, lint and production build passed with the separate customer-address/landmark form and runtime provider indicator.
- Updated production Docker image built successfully.
- `VERIFY_BROWSER=1 python3 scripts/nipost-integration.py` passed against a **local HTTP fixture**, dummy key and fresh disposable PostgreSQL. It exercises the actual Reqwest adapter, creation-time re-verification, customer-address provenance, invalid/unavailable/throttled lookup rejection, and rider/OTP API and Chrome workflows.
- `VERIFY_BROWSER=1 python3 scripts/verify-container.py` passed against the release image in offline mock mode: all three migrations, idempotent seeding, merchant/rider/OTP/browser regression, CORS/rate limiting, production access denial, and non-root/custom-port/health checks. It additionally verified that the release binary rejects the debug-only loopback NIPOST base URL.
- The actual private key was checked against tracked/candidate source and was absent. No credential is embedded in frontend code or build inputs.
- **Real official API:** the configured key is a sandbox key. Official sandbox examples and user-provided `KT-14-B62-EB-16` returned HTTP 200 with `valid: false` / `not_found`; compact, spaced and hyphenated KT forms behaved identically. Autocomplete returned empty suggestions. A malformed postcode returned `status: invalid`; an unauthenticated request returned HTTP 401 / `auth_required`.
- The user confirms the same Test key shows KT as Valid in the dashboard. No positive public-API validity result was observed, so **valid sandbox lookup and real NIPOST-backed delivery creation are not marked as passed**. No production key or higher lookup level was tested.

See [NIPOST integration](NIPOST.md) for exact results, schema/provenance rules, configuration, error mapping and the remaining dashboard/API discrepancy. Earlier public positive workflow results in this document refer to the original offline demo, not a successful real NIPOST lookup.
