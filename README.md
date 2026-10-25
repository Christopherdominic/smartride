# SmartRide

### One postcode. Every delivery.

**Postcode-powered delivery coordination for Nigeria’s last mile.**

SmartRide helps merchants create deliveries from digital postcodes, assign riders, track parcel progress, and confirm the final handoff with a one-time password. Built as a working hackathon MVP, it connects a Next.js dashboard to a Rust API and PostgreSQL persistence.

[**Launch the demo**](https://smartride-kappa.vercel.app/demo) · [Merchant dashboard](https://smartride-kappa.vercel.app/dashboard) · [Rider workspace](https://smartride-kappa.vercel.app/rider) · [API documentation](https://smartride-api-bfrg.onrender.com/docs)

> **Shared demo:** Use sample recipient details only. NIPOST L1 verifies postcode validity; street addresses and directions are customer-provided and are not NIPOST-verified. Sandbox keys use the sandbox dataset. Offline mock mode remains available for development. See [NIPOST integration and actual API results](docs/NIPOST.md).

## Overview

Last-mile delivery depends on identifying the destination and coordinating each handoff. SmartRide demonstrates how digital postcode resolution can become the starting point for a connected delivery workflow:

```text
Digital postcode → Resolved address → Delivery order → Rider assignment
                                                     ↓
Proof of delivery ← OTP confirmation ← Delivery tracking
```

The project explores the application of Nigeria’s NIPOST National Digital Alphanumeric Postcode System (NDAPS) concept to merchant and logistics operations. Its official API adapter requests only Lookup Level 1. The postcode, customer-provided street address, and landmark are recorded separately with explicit provenance; no street address or coordinates are inferred from L1.

## Implemented capabilities

| Area | Capabilities |
| --- | --- |
| Merchant operations | Delivery creation, recent deliveries, status breakdowns, dashboard statistics, and delivery details |
| Postcode verification | Official NIPOST L1 lookup, bounded server-side autocomplete, five-segment validation, and an offline mock provider |
| Rider coordination | Rider registration, workload visibility, assignment, and assigned-delivery views |
| Delivery tracking | Enforced lifecycle transitions, terminal failure handling, and persisted event timelines |
| Proof of delivery | Secure six-digit OTP generation, hashed storage, expiry, attempt limits, and replay prevention |
| Interactive demo | Guided delivery creation followed by assignment, tracking, and OTP confirmation through real API calls |
| Developer tooling | OpenAPI specification, Swagger UI, SQLx migrations, Docker image, and integration test scripts |

### Delivery lifecycle

```text
created → assigned → picked_up → out_for_delivery → delivered
```

Active deliveries can be marked as `failed`. Invalid transitions are rejected, and a delivery can reach `delivered` only after successful OTP verification. Status changes and their events are committed atomically.

## Live deployment

| Component | Platform | Public URL |
| --- | --- | --- |
| Frontend | Vercel Hobby | [smartride-kappa.vercel.app](https://smartride-kappa.vercel.app) |
| Rust API | Render Free | [Health endpoint](https://smartride-api-bfrg.onrender.com/health) |
| API explorer | Swagger UI | [Interactive documentation](https://smartride-api-bfrg.onrender.com/docs) |
| Database | Neon Free | Dedicated PostgreSQL 16 demo project; accessed only by the backend |

The original offline demo’s public workflow was verified on **25 October 2026**. The NIPOST update passes fixture-based L1 workflow tests, but its real API currently reports the documented and user-provided sandbox codes as not found; a positive official lookup remains unverified. See [actual NIPOST results](docs/NIPOST.md). See the [validation record](docs/VALIDATION.md) for executed checks and deployment evidence.

**Free-tier availability:** Render sleeps after idle periods. Allow approximately one minute for the first API request to wake the service before presenting the demo. Hosting quotas also apply.

## Three-minute demo

1. Open the [guided demo](https://smartride-kappa.vercel.app/demo). The dedicated **Lagos Market • Demo** merchant is selected.
2. Enter a postcode appropriate to the configured key’s dataset and click **Verify postcode**. Autocomplete is available if the key has search scope.
3. Supply the customer’s full delivery address and optional landmark; L1 does not verify these details.
4. Use the supplied sample recipient details, describe the parcel, and create the delivery.
5. Select a rider and click **Assign rider**.
6. Click **Mark parcel picked up**, then **Start delivery**.
7. Click **Generate demo OTP** and enter the displayed six-digit code.
8. Click **Confirm delivery**. Review the completed timeline and confirmed proof of delivery.
9. Visit the merchant dashboard for updated statistics, or the rider workspace to view assigned deliveries.

The following fixtures apply **only to offline mock mode**, where **Resolve postcode** previews the fictional destination. They are not NIPOST sandbox or production samples.

| Offline demo postcode | Fictional destination |
| --- | --- |
| `EK01A03FK01` | Demo Building, Test District, Nigeria |
| `LA01A03IK01` | Fictional Market House, Demo Lagos District, Nigeria |
| `AB01A03FC01` | Fictional Unity House, Demo Abuja District, Nigeria |

Seeded delivered examples are labeled as having no actual OTP confirmation. Deliveries completed through the workflow display confirmed OTP proof.

## Technology

| Layer | Stack |
| --- | --- |
| Backend | Rust, Axum, Tokio, SQLx, Reqwest |
| Frontend | Next.js App Router, TypeScript, React, Tailwind CSS, Lucide icons |
| Database | PostgreSQL 16 with pgcrypto |
| Infrastructure | Docker, Render, Neon, Vercel |
| API documentation | OpenAPI 3.0 and Swagger UI |

## Repository structure

```text
backend/
  src/                  API handlers, postcode providers, and delivery workflow
  migrations/           Versioned PostgreSQL migrations
  Dockerfile            Production image and built-in health probe
frontend/
  app/                  Landing page, merchant dashboard, rider workspace, demo
  components/           Shared interface components and workflow controls
  lib/                  Typed API client and response models
docs/
  DEPLOYMENT.md         Hosting configuration and exact redeployment commands
  VALIDATION.md         Local, container, and public verification results
  NIPOST.md             L1 contract, environment configuration, and real API results
  openapi.json          API specification
scripts/                API, browser, security, and container verification
render.yaml             Free Render service configuration
Dockerfile              Symlink to backend/Dockerfile for root-context builds
```

## Local development

### Prerequisites

- Rust **1.97.1 or newer**; the container uses a digest-pinned Rust 1.98.0 toolchain.
- Node.js **22** and npm.
- Docker with Docker Compose, or PostgreSQL 16 with pgcrypto support.

### 1. Start PostgreSQL and configure the environment

Run from the repository root:

```sh
docker compose up -d
cp -n backend/.env.example backend/.env
cp -n frontend/.env.example frontend/.env.local
```

The copy commands preserve existing environment files. Review those files if they already exist. Docker Compose exposes the development database on port **5434**.

### 2. Start the API

```sh
cd backend
APP_MODE=demo POSTCODE_PROVIDER=mock cargo run
```

Startup applies the SQLx migrations and idempotently seeds one dedicated demo merchant, three riders, and five example deliveries. Existing records are preserved.

### 3. Start the frontend

In a second terminal:

```sh
cd frontend
npm ci
npm run dev
```

| Local service | URL |
| --- | --- |
| Frontend | http://localhost:3000 |
| API health | http://localhost:8080/health |
| Swagger UI | http://localhost:8080/docs |
| OpenAPI specification | http://localhost:8080/openapi.json |

### Environment variables

| Variable | Location | Purpose |
| --- | --- | --- |
| `DATABASE_URL` | Backend only | PostgreSQL connection string; use a direct TLS connection for Neon |
| `APP_MODE` | Backend | Set to `demo` for the isolated workflow; defaults to `production` |
| `POSTCODE_PROVIDER` | Backend | `nipost` for official L1 verification; `mock` for offline development |
| `NIPOST_API_BASE_URL` | Backend | `https://api.postcode.gov.ng` |
| `NIPOST_API_KEY` | Backend only | Private NIPOST key; never expose it through frontend configuration |
| `FRONTEND_ORIGIN` | Backend | Exact permitted browser origin, including scheme and without a trailing slash |
| `PORT` | Backend | Runtime listening port; defaults to `8080` |
| `RUST_LOG` | Backend | Logging level, typically `info` |
| `NEXT_PUBLIC_API_BASE_URL` | Frontend | Public API origin, without `/api/v1`; defaults locally to `http://localhost:8080` |

`NEXT_PUBLIC_API_BASE_URL` is compiled into browser code. Rebuild the frontend after changing it. Database credentials and provider secrets must never use a `NEXT_PUBLIC_` prefix.

## API reference

See the [OpenAPI specification](docs/openapi.json) for request and response schemas, or use the [deployed Swagger UI](https://smartride-api-bfrg.onrender.com/docs).

| Method | Endpoint | Purpose |
| --- | --- | --- |
| GET | `/health` | Database readiness |
| GET | `/api/v1/demo` | Demo merchant, provider, dataset environment, and address requirements |
| GET | `/api/v1/addresses/resolve?postcode=...` | Verify postcode validity (or resolve an offline fixture) |
| GET | `/api/v1/addresses/autocomplete?q=...` | Bounded server-side NIPOST autocomplete |
| GET, POST | `/api/v1/deliveries` | List or create demo merchant deliveries |
| GET | `/api/v1/deliveries/{id}` | Delivery details, address, rider, proof, and timeline |
| GET | `/api/v1/stats` | Merchant dashboard statistics |
| GET, POST | `/api/v1/riders` | List or register demo riders |
| GET | `/api/v1/riders/{id}/deliveries` | Retrieve assigned deliveries |
| POST | `/api/v1/deliveries/{id}/assign` | Assign a rider |
| POST | `/api/v1/deliveries/{id}/status` | Record pickup, transit, or failure |
| POST | `/api/v1/deliveries/{id}/otp` | Generate an OTP with demo-only display |
| POST | `/api/v1/deliveries/{id}/confirm` | Verify the OTP and complete delivery atomically |

Errors use `{"error":"message","code":"category"}`. Business endpoints return `403` outside demo mode. Delivery lists show the latest 200 records; statistics include all records.

## Security and MVP boundaries

The deployment is a shared, fictional-data demonstration. Production merchant/rider authentication and customer OTP messaging are not implemented. Setting `APP_MODE=production` disables business endpoints; it does not enable a production login system.

The implemented safeguards include:

- **Data isolation:** Public operations are restricted to the dedicated demo merchant and its riders. Arbitrary merchant identities do not grant access to existing merchants’ records.
- **OTP protection:** OS-backed random generation, salted bcrypt storage at cost 10, five-minute expiry, five verification attempts, and consumed-state checks. Plaintext codes are displayed only by the guarded demo issuance endpoint and are never logged or persisted.
- **Transactional integrity:** Row locks serialize OTP verification; failed attempts persist, and successful confirmation commits the delivery state and event together.
- **Request controls:** Exact-origin CORS, input length checks, JSON body limits, foreign-key constraints, and an aggregate limit of 240 API requests per minute per instance.

The rate limiter is intended for a single-instance demo. Real deployments require authenticated merchant/rider access, distributed abuse prevention, customer messaging, and operational controls appropriate for customer data. Payments, dispatch optimization, and cancellation workflows are outside this MVP’s scope.

**OTP recovery:** If a displayed code is lost, wait for its five-minute expiry before requesting another. Five incorrect attempts lock that delivery’s OTP flow; create a new sample delivery to continue the demonstration.

## Build and verification

### Backend

```sh
cd backend
cargo fmt --check
cargo check --locked
cargo test --locked
```

### Frontend

```sh
cd frontend
npm run typecheck
npm run lint
npm run build
```

### API integration

With the local API and development database running, execute from the repository root:

```sh
API_BASE_URL=http://localhost:8080 \
TEST_DB_CONTAINER=smartride-postgres \
python3 scripts/smoke.py
```

The smoke test creates sample deliveries. In NIPOST mode, supply `TEST_POSTCODE` with a code known to be valid for the configured key; no real positive NIPOST test is claimed unless that API returns valid. `TEST_DB_CONTAINER` enables database-level migration, hash-storage, and OTP-expiry checks against newly created test records. Omit it when testing a remote API; use only a dedicated demo database.

### Production container

```sh
docker build -f backend/Dockerfile -t smartride-api:free-hosting .
python3 scripts/verify-container.py
```

The container test starts disposable PostgreSQL and API containers, verifies fresh migrations, seeding, non-root execution, and custom port binding, then runs API and security checks. It removes its own resources without touching the existing database. Ports **8081** and **8082** must be available.

### Public browser workflow

```sh
FRONTEND_URL=https://smartride-kappa.vercel.app \
node scripts/browser-smoke.mjs
```

In NIPOST mode, also set `TEST_POSTCODE` to a known valid code for the configured dataset. The browser test requires Node.js 22+ and Google Chrome. Set `CHROME_PATH` to override `/usr/bin/google-chrome`. It verifies the actual UI workflow and saves desktop/mobile screenshots under `/tmp/smartride-*.png`.

For local browser testing, run the API on port 8081 and build the frontend with `NEXT_PUBLIC_API_BASE_URL=http://localhost:8081` before starting it. The security script additionally expects a production-mode API on port 8082 and intentionally exhausts the demo rate limit; run it last.

See [VALIDATION.md](docs/VALIDATION.md) for the checks that were actually executed, including successful public API and browser tests.

## Deployment and operations

The verified stack is **Render Free + Neon Free + Vercel Hobby**. The API binds to `0.0.0.0:$PORT`, applies migrations before serving, and exposes a database-aware health endpoint. Its Docker image runs as a non-root user and includes a built-in health probe.

[DEPLOYMENT.md](docs/DEPLOYMENT.md) contains the live resource identifiers, exact redeployment commands, environment settings, dashboard instructions, and free-tier constraints. Backend builds require the repository root as context because the binary embeds `docs/openapi.json`.

Keep `.env` files and provider credentials out of Git. The public demonstration must remain isolated from databases containing sensitive information.

## License

Distributed under the [MIT License](LICENSE).
