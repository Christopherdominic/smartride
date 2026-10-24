# Validation record — updated 25 October 2026

Executed against the existing local PostgreSQL 16 Docker service and the implemented application. No public deployment is claimed.

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
- **Vercel:** configuration/instructions prepared; not deployed because no public API is available yet. No live end-to-end URLs have been tested.
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
- Neon/Vercel account authorization and public deployment verification are still pending. **No public success is claimed by these local/container results.**
