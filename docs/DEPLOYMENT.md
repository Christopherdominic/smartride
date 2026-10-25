# Free deployment: Render + Neon + Vercel

SmartRide uses **Render Free** for the Rust API, **Neon Free** for PostgreSQL, and **Vercel Hobby** for the Next.js frontend. Railway is not required. No paid resources should be selected.

## Verified live deployment — 25 October 2026

| Component | Live resource |
| --- | --- |
| Frontend / guided demo | https://smartride-kappa.vercel.app/demo |
| API health | https://smartride-api-bfrg.onrender.com/health |
| Swagger UI | https://smartride-api-bfrg.onrender.com/docs |
| Render service | `srv-dbejnr8ae00c73dcgjsg`, `smartride-api`, Frankfurt, **Free** |
| Neon project | `floral-band-71506813`, `smartride-demo`, AWS Frankfurt, PostgreSQL 16, **Free** organization, 0.25 CU |
| Vercel project | `smartride`, `christopherdominics-projects`, **Hobby**, Node 22 |
| Verified application source | GitHub commit `33e42884bf241fc70353eeb14b7687b5b18c682c` on `main` |

The backend reports Live, the frontend reports Ready, and the public health and anonymous Chrome end-to-end workflow passed. No paid plan was selected. Database credentials exist only in the backend provider environment and private CLI state, not in Git or the frontend.

### Redeploy these existing resources

Backend (after installing/signing in to the [Render CLI](https://render.com/docs/cli)):

```sh
render workspace set tea-ct55sml6l47c73fd15v0 --confirm
render deploys create srv-dbejnr8ae00c73dcgjsg --wait --confirm
```

Render also watches backend/image changes on GitHub `main`; documentation-only commits are excluded by its build filter. Runtime environment settings are configured in Render's dashboard, including `FRONTEND_ORIGIN=https://smartride-kappa.vercel.app`. Preserve the existing Neon direct TLS secret when changing other variables.

Frontend (the initial verified deployment uploaded **only the frontend directory**, not the repository root):

```sh
npx vercel login
npx vercel link --cwd frontend --project smartride --scope christopherdominics-projects --yes
npx vercel deploy --cwd frontend --project smartride --scope christopherdominics-projects --prod --yes --archive=tgz
```

Its existing Production/Preview `NEXT_PUBLIC_API_BASE_URL` is `https://smartride-api-bfrg.onrender.com`. The current Vercel project has no Git integration and its project Root Directory is unset because the CLI uploads `frontend/` as the source root. **Do not upload the full repository with these settings.** To switch this existing project to Git-based builds, connect the repository and set Root Directory to `frontend` first. The dashboard instructions below describe that alternative.

## Why this stack

Checked against official documentation on 24 October 2026:

| Service | Choice | Constraints |
| --- | --- | --- |
| Render | Free Docker web service | 750 instance hours/workspace/month, sleeps after 15 minutes idle, approximately one-minute cold start. Bandwidth/build quotas apply. Without a payment method, exhaustion suspends service/builds rather than creating overage charges. Account-specific anti-abuse verification may still apply. |
| Neon | Free PostgreSQL | No card required. Current published allowance: 100 CU-hours/month/project and 1 GB storage/project; idle compute scales to zero. Use a dedicated sample-data project. |
| Vercel | Hobby, personal demo | Free for personal, non-commercial use within its quotas. Do not choose a paid team plan. |
| Koyeb | Not selected | Although a free instance exists, current onboarding requires card validation with a $29 authorization hold and can charge the selected paid plan. Render avoids that onboarding risk. |

Sources: [Render Free](https://render.com/docs/free), [Render billing FAQ](https://render.com/docs/faq), [Neon pricing](https://neon.com/pricing), [Vercel Hobby](https://vercel.com/docs/plans/hobby), [Koyeb pricing FAQ](https://www.koyeb.com/docs/faqs/pricing).

## 1. Make the tested source available

Render and Vercel must deploy the commit containing the working MVP, not the original API-only repository. Verify the intended branch includes `render.yaml`, `backend/Dockerfile`, and `frontend/package-lock.json`.

```sh
git status --short
git log -1 --oneline
```

Publish the reviewed source normally to GitHub; never force-push or upload `.env` files. The local `.gitignore` excludes secrets and provider CLI state. Render's Blueprint uses the repository root as Docker context. `backend/` alone is insufficient because the binary embeds `docs/openapi.json`.

## 2. Neon Free database

1. Sign in at https://console.neon.tech and create a project named `smartride-demo` on the **Free** plan. Choose PostgreSQL 16 if offered and a region near Render Frankfurt (for example AWS Frankfurt). Any supported newer PostgreSQL version may also work, but local verification uses 16.
2. Use a dedicated database containing only fictional demo data. The default `neondb` database and project owner role are sufficient.
3. Open **Connect** and turn **Connection pooling off**. Copy the **direct PostgreSQL connection string** into Render's secret `DATABASE_URL` field, not into Git or chat.
4. Keep TLS enabled. Use `sslmode=verify-full` for certificate/hostname verification with SQLx. Preserve the actual generated hostname, database, username, and password. Never use a `-pooler` hostname for this deployment: startup SQLx migrations use session advisory locks.
5. Neon supports [pgcrypto](https://neon.com/docs/extensions/pgcrypto). The unchanged initial migration creates it using the owner connection, then creates tables; migration 2 adds the demo-data isolation marker and OTP constraints. **Do not run ad-hoc schema changes.**

The application runs both migrations before listening, and seeds the dedicated demo records idempotently. No separate paid pre-deploy job or manual SQL insert is needed.

## 3. Render Free backend

### Blueprint setup (recommended)

1. Sign in at https://dashboard.render.com and choose **New → Blueprint**.
2. Connect `Christopherdominic/smartride`, select the branch containing the tested deployment commit, and use **render.yaml** from the repository root.
3. Confirm the service is **smartride-api**, **Docker**, **Free**, region **Frankfurt**. No database, paid disk, background worker, or paid instance is declared.
4. Supply the two fields marked `sync: false`: the Neon direct `DATABASE_URL` and `FRONTEND_ORIGIN`.
5. If the Vercel production domain is not allocated yet, use `https://example.invalid` temporarily for `FRONTEND_ORIGIN`. This denies real browser origins until it is replaced; server-side health/API verification still works. Do not use a wildcard.
6. Click **Deploy Blueprint** and inspect build/runtime logs. Wait until the specific deployment is **Live**.

### Equivalent manual Web Service settings

| Field | Value |
| --- | --- |
| Repository | `https://github.com/Christopherdominic/smartride` |
| Branch | The branch with the verified MVP/deployment commit |
| Runtime | Docker |
| Root Directory | Leave blank |
| Dockerfile Path | `./backend/Dockerfile` |
| Docker Build Context | `.` (repository root) |
| Instance Type | **Free** |
| Region | Frankfurt |
| Docker Command | Leave blank; use the image CMD |
| Health Check Path | `/health` |

Set these environment variables:

```text
APP_MODE=demo
POSTCODE_PROVIDER=nipost
NIPOST_API_BASE_URL=https://api.postcode.gov.ng
NIPOST_API_KEY=<server-side secret configured privately>
PORT=10000
RUST_LOG=info
DATABASE_URL=<Neon direct TLS connection string; secret>
FRONTEND_ORIGIN=https://<actual-production-domain>.vercel.app
```

`PORT` is read at runtime and the binary binds `0.0.0.0`; `10000` matches Render's usual web-service port. A root `Dockerfile` symlink points to `backend/Dockerfile` so Render CLI defaults and dashboard builds share one implementation. The image uses its own `smartride --healthcheck` probe and copied CA certificates; no runtime package download is required. The Docker image defaults to `APP_MODE=production`; only this isolated public demo explicitly enables demo mode. Business endpoints remain denied in production mode.

Copy the actual `https://...onrender.com` URL from Render. Do not assume the service name is globally available. Warm and check the API, allowing for the initial cold start:

```sh
export API_BASE_URL='https://ACTUAL-API.onrender.com'
curl --fail --show-error --max-time 120 "$API_BASE_URL/health"
curl --fail --show-error --max-time 120 "$API_BASE_URL/api/v1/demo"
TEST_POSTCODE='<known valid postcode for the configured key>' python3 scripts/smoke.py
```

The remote smoke test creates fictional records. Leave `TEST_DB_CONTAINER` unset: it is only for local database expiry inspection. Once the service is Live, check `/docs` and the complete API smoke result; a build/upload acknowledgment alone is insufficient.

## 4. Vercel Hobby frontend

1. Sign in at https://vercel.com/new and import the same repository and deployment branch into your personal Hobby account.
2. Choose **Next.js** with **Root Directory = frontend**. The tracked `frontend/vercel.json` specifies `npm ci` and `npm run build`.
3. Add `NEXT_PUBLIC_API_BASE_URL` with the actual Render HTTPS origin, without `/api/v1` or credentials. Set it for Production (and Preview if you want preview builds to succeed). This public URL is compiled into the JavaScript bundle. The Next.js configuration rejects Vercel builds missing a valid HTTPS API origin.
4. Deploy and wait for **Ready**. Record the stable Production domain, not a one-off preview URL.
5. Set Render's `FRONTEND_ORIGIN` to that exact HTTPS production origin, without a trailing slash; save and redeploy/restart the API.
6. For judges to access it, ensure the **Production** deployment is public. If Vercel Authentication/Deployment Protection is enabled for Production, turn it off for this dedicated demo. Previews can remain protected.

Equivalent CLI deployment from the repository root (after authenticating):

```sh
npx vercel login
cd frontend
npx vercel link
npx vercel env add NEXT_PUBLIC_API_BASE_URL production
# Enter the public Render URL at the prompt.
npx vercel --prod
```

When prompted to set up the project, choose the personal Hobby scope and the Next.js defaults in `frontend/`. Use `vercel link` to select an existing project if one was already created in the dashboard; do not create a duplicate. If the Vercel project root is configured as `frontend`, run the CLI from the repository root linked to that project instead of nesting the root twice.

## 5. Public acceptance test

Set the **actual** URLs, not placeholders:

```sh
export API_BASE_URL='https://ACTUAL-API.onrender.com'
export FRONTEND_URL='https://ACTUAL-FRONTEND.vercel.app'
curl --fail --show-error --max-time 120 "$API_BASE_URL/health"
TEST_POSTCODE='<known valid postcode for the configured key>' python3 scripts/smoke.py
TEST_POSTCODE='<known valid postcode for the configured key>' node scripts/browser-smoke.mjs
```

The browser script uses `FRONTEND_URL`, operates the actual deployed UI, and persists deliveries through the public API. It needs Node 22+ and Google Chrome. Complete postcode → create → assign → pickup → transit → OTP → delivered, then confirm the final timeline and proof. Check browser CORS/network errors if the API works directly but not through the UI. Render may return a temporary HTML loading page while waking; warm `/health` first and retry once it is ready. No synthetic success state or local API fallback counts as public validation.

## Local image verification

```sh
docker build --progress=plain -f backend/Dockerfile -t smartride-api:free-hosting .
python3 scripts/verify-container.py
```

The test starts disposable PostgreSQL 16 and API containers with random temporary credentials, checks the image's non-root user and custom `PORT=10000`, runs fresh migrations, verifies seed idempotence on restart, then runs the API/security smoke tests. It removes only its own containers/network and never touches the existing application database. Ports 8081 and 8082 must be free.

## Secrets and security

- `DATABASE_URL` belongs only in Render's environment. Never put it in `NEXT_PUBLIC_*`, Git, screenshots, or browser code.
- No production authentication is introduced by hosting. `APP_MODE=production` still disables business operations. Demo access remains limited to dedicated fictional merchant/rider data; do not enable it against a sensitive database.
- Use `POSTCODE_PROVIDER=nipost` for official L1 verification and configure the key privately in Render. Use `mock` only for explicit offline development; failures never fall back. See [NIPOST.md](NIPOST.md) for the current sandbox API discrepancy and actual tests.
- OTP display remains a demo-only operation with expiry, attempt limits, hashing, and replay prevention. Do not expose real recipient data or reuse these demo access rules in production.
- Free Render services sleep and can be suspended by quota/excess outbound traffic (including database traffic); Neon/Vercel quotas also apply. Warm the demo manually shortly before presenting. Do not add artificial keep-alive traffic.

[Validation record](VALIDATION.md) distinguishes tested local/container behavior from unverified public URLs. Account sign-in/authorization is required before resources can be created. If an account specifically insists on payment, stop and report it; never silently upgrade.
