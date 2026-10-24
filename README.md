# SmartRide

**Postcode-powered delivery infrastructure for Nigeria.**

SmartRide is a delivery coordination API that connects digital postcode-based address resolution with merchant delivery requests, rider dispatch, tracking, and proof of delivery.

The goal is to make delivery operations easier to integrate into e-commerce platforms, logistics systems, and merchant applications.

## Tech Stack

- **Backend:** Rust, Axum, Tokio
- **Database:** PostgreSQL, SQLx
- **Address resolution:** Pluggable postcode providers
- **Frontend:** Next.js, TypeScript, Tailwind CSS (planned)
- **Development:** Docker Compose

## Current Features

- Health-check endpoint
- Mock digital postcode resolution
- Merchant records
- Delivery creation and retrieval
- PostgreSQL persistence
- Delivery event recording

## API Endpoints

| Method | Endpoint | Description |
|---|---|---|
| GET | `/health` | Check API and database health |
| GET | `/api/v1/addresses/resolve?postcode=...` | Resolve a postcode |
| POST | `/api/v1/deliveries` | Create a delivery |
| GET | `/api/v1/deliveries/{id}` | Retrieve a delivery |

## Local Development

Start PostgreSQL:

```bash
docker compose up -d
```

Configure `backend/.env` using the environment example, then start the backend:

```bash
cd backend
cargo run
```

The API runs locally at `http://127.0.0.1:8080`.

## Development Status

SmartRide is an early-stage MVP.

The current postcode provider uses fictional demonstration data. Live NIPOST integration is not yet implemented.

Merchant authentication, rider assignment, delivery status transitions, OTP confirmation, and the frontend are planned.

**Security:** Current API endpoints are intended for local development only and should not be exposed publicly without authentication and authorization.

## Vision

One postcode. Every delivery.
