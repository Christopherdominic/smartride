# SmartRide

**One postcode. Every delivery.** A Rust + Next.js hackathon prototype for postcode-powered delivery coordination in Nigeria.

**Live demo:** https://smartride-kappa.vercel.app/demo

**API / Swagger:** https://smartride-api-bfrg.onrender.com/docs

Deployed on Vercel Hobby, Render Free, and Neon Free; the complete public browser workflow was verified on 25 October 2026. Render sleeps after idle periods, so allow about a minute for the first API request to wake it.

SmartRide connects fictional digital postcode resolution, merchant delivery creation, rider dispatch, tracking, and OTP proof of delivery. All demo actions call the Rust API and persist in PostgreSQL.

## Run locally

Requirements: Rust 1.97.1 or newer, Node 22, npm, Docker Compose (or PostgreSQL 16 with pgcrypto).

```sh
docker compose up -d
cp -n backend/.env.example backend/.env
cp -n frontend/.env.example frontend/.env.local
cd backend
APP_MODE=demo POSTCODE_PROVIDER=mock cargo run
```

In a second terminal:

```sh
cd frontend
npm ci
npm run dev
```

Open http://localhost:3000. API health: http://localhost:8080/health. Swagger UI: http://localhost:8080/docs. Machine-readable specification: `/openapi.json`.

The existing `.env` is preserved. Set `APP_MODE=demo`, `POSTCODE_PROVIDER=mock`, and the correct `DATABASE_URL`. Migrations run automatically before the server starts. Demo seeding is idempotent and protected by an advisory transaction lock. It creates one dedicated merchant, three riders, and five example deliveries with lifecycle histories. Existing records and the initial migration are preserved.

## Three-minute walkthrough

1. Open `/demo`. The dedicated **Lagos Market • Demo** merchant is selected.
2. Resolve `EK01A03FK01` (also try `LA01A03IK01` or `AB01A03FC01`). Every address is explicitly fictional.
3. Use the supplied sample recipient details, add a parcel description, and create the delivery.
4. On the delivery page, choose a rider and click **Assign rider**.
5. Click **Mark parcel picked up**, then **Start delivery**.
6. Click **Generate demo OTP**. Copy the displayed six-digit code into the confirmation field.
7. Click **Confirm delivery**. Review the completed timeline and proof confirmation; visit `/dashboard` for updated statistics.
8. `/rider` provides the same progress and confirmation actions scoped to the selected demo rider’s assigned deliveries.

Seeded delivered examples are clearly identified as having no actual OTP confirmation. Newly completed deliveries carry confirmed OTP proof. If an issued code is lost on refresh, wait five minutes for expiry and request another. A delivery locked after five incorrect attempts requires a new demo delivery; issuance cannot reset its attempt limit.

## API

See [OpenAPI](docs/openapi.json) for request contracts. Errors use `{"error":"message"}`. Business endpoints return 403 outside demo mode.

| Method | Endpoint | Purpose |
| --- | --- | --- |
| GET | `/health` | Database readiness |
| GET | `/api/v1/demo` | Demo merchant and fictional postcodes |
| GET | `/api/v1/addresses/resolve?postcode=...` | Resolve fictional address |
| GET, POST | `/api/v1/deliveries` | List / create demo merchant deliveries |
| GET | `/api/v1/deliveries/{id}` | Delivery, address, rider, proof, timeline |
| GET | `/api/v1/stats` | Merchant dashboard counts |
| GET, POST | `/api/v1/riders` | List / register demo riders |
| GET | `/api/v1/riders/{id}/deliveries` | Assigned deliveries |
| POST | `/api/v1/deliveries/{id}/assign` | Assign rider |
| POST | `/api/v1/deliveries/{id}/status` | Pickup, transit, or failure |
| POST | `/api/v1/deliveries/{id}/otp` | Generate demo-only OTP |
| POST | `/api/v1/deliveries/{id}/confirm` | Verify OTP and deliver atomically |

## Security and integration boundaries

- **Public shared demo, not production authentication.** Anyone can operate the dedicated demo merchant/riders. Never enter real personal or commercial data. Arbitrary merchant IDs are rejected; existing merchants’ deliveries cannot be read or changed through demo endpoints.
- `APP_MODE` defaults to `production`, which disables every business endpoint. Production merchant/rider authentication and recipient OTP messaging are not implemented. Do not enable demo mode on a database with sensitive information. Prefer a dedicated deployment/database.
- OTPs use OS-backed random generation, salted bcrypt cost 10 storage, five-minute expiry, five attempts, and consumed-state checks. Plaintext is returned only by the guarded demo-only issuance endpoint and is never logged or persisted. Row locks serialize issuance/verification; failed attempts persist; delivery and event changes commit atomically.
- A single-instance aggregate rate limit allows 240 API calls per minute. OTP attempts also have persistent per-delivery limits. This intentionally simple public demo limiter is not a distributed production abuse-prevention system. Add an ingress rate limit and authentication before production use.
- CORS permits only `FRONTEND_ORIGIN`; it is not an authorization mechanism. Default Axum JSON body limits apply; text fields have length bounds. UUID foreign keys enforce data integrity.
- The existing `PostcodeProvider` abstraction remains. Live NIPOST is deliberately unavailable: no endpoint, authentication convention, response schema, or lookup level is invented. The demo postcode format is a local fixture convention, not a claim about official NDAPS validation. Verified official documentation and credentials are required to implement the adapter.
- Delivery lists show the latest 200 rows; statistics count all rows. No payments, dispatch optimization, production messaging, or cancellation workflow is included.

## Deployment

The free deployment target is **Render Free + Neon Free + Vercel Hobby**; Railway is optional and is not used. Follow [deployment instructions](docs/DEPLOYMENT.md). Backend Docker builds use the repository root as context and include the existing migrations. The server binds `0.0.0.0:$PORT`, runs migrations at startup, and exposes database-aware `/health`.

```sh
docker build -f backend/Dockerfile -t smartride-api .
docker run --rm --env-file backend/.env -p 8080:8080 smartride-api
```

For container access to a host database, adjust `DATABASE_URL` to a reachable host; `localhost` inside Docker refers to the container.

## Validation

```sh
cd backend
cargo fmt --check
cargo check --locked
cargo test --locked
cd ../frontend
npm run typecheck
npm run lint
npm run build
cd ..
API_BASE_URL=http://localhost:8080 TEST_DB_CONTAINER=smartride-postgres python3 scripts/smoke.py
```

The smoke test creates **new sample records** in the selected demo database. With `TEST_DB_CONTAINER`, it verifies migrations/hash storage and expires one newly created OTP to test actual database expiry. Omit that variable for remote API testing. Use only a dedicated demo database. See [validation record](docs/VALIDATION.md) for what was actually executed.

For browser integration tests, start the backend in demo mode on port 8081 and build the frontend with `NEXT_PUBLIC_API_BASE_URL=http://localhost:8081`, then run `npm start` from `frontend/` and `node scripts/browser-smoke.mjs` from the repository root. It requires Node 22+ and local Google Chrome (`CHROME_PATH` overrides `/usr/bin/google-chrome`). Screenshots are saved under `/tmp/smartride-*.png`.

To run the access/CORS/rate-limit checks, also start a production-mode API on port 8082 and execute `python3 scripts/security-smoke.py` **after** the other tests. It exhausts the demo instance's rate limit; allow a minute or restart the demo API before continuing.
