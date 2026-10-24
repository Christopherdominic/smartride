use super::*;
use axum::{
    extract::Request,
    middleware::Next,
    response::{Html, Response},
};
use hmac::{Hmac, Mac};
use rand::{Rng, rngs::OsRng};
use sha2::Sha256;
use sqlx::Row;
pub const MERCHANT: Uuid = Uuid::from_u128(0x11111111111141118111111111111111);
fn bad(message: &str) -> ApiError {
    ApiError::new(StatusCode::CONFLICT, message)
}
pub async fn guard(
    State(s): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    if !s.demo {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "Public operations disabled: production authentication is not configured",
        ));
    }
    {
        let mut limit = s.limiter.lock().map_err(ApiError::internal)?;
        if limit.0.elapsed().as_secs() >= 60 {
            *limit = (std::time::Instant::now(), 0);
        }
        limit.1 += 1;
        if limit.1 > 240 {
            return Err(ApiError::new(
                StatusCode::TOO_MANY_REQUESTS,
                "Demo request limit reached; retry in one minute",
            ));
        }
    }
    let response = next.run(req).await;
    if matches!(
        response.status().as_u16(),
        400 | 404 | 405 | 413 | 415 | 422
    ) && !response
        .headers()
        .get(axum::http::header::CONTENT_TYPE)
        .is_some_and(|v| v == "application/json")
    {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "invalid request: check path, query, JSON fields and content type",
        ));
    }
    Ok(response)
}
pub async fn seed(db: &PgPool) -> Result<(), sqlx::Error> {
    let mut tx = db.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(734923)")
        .execute(&mut *tx)
        .await?;
    sqlx::query("INSERT INTO merchants(id,name,is_demo) VALUES($1,'Lagos Market • Demo',true) ON CONFLICT(id) DO NOTHING").bind(MERCHANT).execute(&mut *tx).await?;
    let dedicated: bool = sqlx::query_scalar("SELECT is_demo FROM merchants WHERE id=$1")
        .bind(MERCHANT)
        .fetch_one(&mut *tx)
        .await?;
    if !dedicated {
        return Err(sqlx::Error::Protocol(
            "reserved demo merchant ID belongs to non-demo data".into(),
        ));
    }
    for (n, name) in ["Ada Okafor", "Tunde Bello", "Zainab Musa"]
        .iter()
        .enumerate()
    {
        sqlx::query("INSERT INTO riders(id,merchant_id,name,phone) VALUES($1,$2,$3,'+2340000000000') ON CONFLICT(id) DO NOTHING").bind(Uuid::from_u128(0x22222222222242228222222222222220+n as u128)).bind(MERCHANT).bind(name).execute(&mut *tx).await?;
    }
    for (n, status) in [
        "created",
        "assigned",
        "picked_up",
        "out_for_delivery",
        "delivered",
    ]
    .iter()
    .enumerate()
    {
        let id = Uuid::from_u128(0x33333333333343338333333333333330 + n as u128);
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM deliveries WHERE id=$1)")
                .bind(id)
                .fetch_one(&mut *tx)
                .await?;
        if exists {
            continue;
        }
        let address:Uuid=sqlx::query_scalar("INSERT INTO addresses(postcode,formatted_address,provider,provider_data) VALUES('EK01A03FK01','Demo Building, Test District, Nigeria','mock','{\"mock\":true,\"verified\":false}') RETURNING id").fetch_one(&mut *tx).await?;
        sqlx::query("INSERT INTO deliveries(id,merchant_id,address_id,recipient_name,recipient_phone,parcel_description,status,rider_id) VALUES($1,$2,$3,'Demo Recipient','+2340000000000','Sample parcel',$4,$5)").bind(id).bind(MERCHANT).bind(address).bind(status).bind(if n>0 {Some(Uuid::from_u128(0x22222222222242228222222222222220))}else{None}).execute(&mut *tx).await?;
        for event in [
            "created",
            "assigned",
            "picked_up",
            "out_for_delivery",
            "delivered",
        ]
        .iter()
        .take(n + 1)
        {
            sqlx::query("INSERT INTO delivery_events(delivery_id,event_type,metadata,created_at) VALUES($1,$2,'{\"seeded\":true}',clock_timestamp())").bind(id).bind(event).execute(&mut *tx).await?;
        }
    }
    tx.commit().await
}
pub async fn config() -> Json<Value> {
    Json(
        json!({"mode":"demo","provider":"mock","merchant":{"id":MERCHANT,"name":"Lagos Market • Demo"},"postcodes":["EK01A03FK01","LA01A03IK01","AB01A03FC01"]}),
    )
}
const SELECT: &str = "SELECT to_jsonb(d) || jsonb_build_object('postcode',a.postcode,'formatted_address',a.formatted_address,'rider_name',r.name,'proof_confirmed',EXISTS(SELECT 1 FROM delivery_otps o WHERE o.delivery_id=d.id AND o.consumed_at IS NOT NULL)) FROM deliveries d JOIN addresses a ON a.id=d.address_id LEFT JOIN riders r ON r.id=d.rider_id";
pub async fn list(State(s): State<AppState>) -> Result<Json<Value>, ApiError> {
    let rows: Vec<Value> = sqlx::query_scalar(&format!(
        "{SELECT} WHERE d.merchant_id=$1 ORDER BY d.created_at DESC LIMIT 200"
    ))
    .bind(MERCHANT)
    .fetch_all(&s.db)
    .await
    .map_err(ApiError::internal)?;
    Ok(Json(json!(rows)))
}
pub async fn detail(
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, ApiError> {
    let mut d: Value = sqlx::query_scalar(&format!("{SELECT} WHERE d.id=$1 AND d.merchant_id=$2"))
        .bind(id)
        .bind(MERCHANT)
        .fetch_optional(&s.db)
        .await
        .map_err(ApiError::internal)?
        .ok_or_else(|| ApiError::new(StatusCode::NOT_FOUND, "delivery not found"))?;
    let events: Vec<Value> = sqlx::query_scalar(
        "SELECT to_jsonb(e) FROM delivery_events e WHERE delivery_id=$1 ORDER BY created_at,CASE event_type WHEN 'created' THEN 0 WHEN 'assigned' THEN 1 WHEN 'picked_up' THEN 2 WHEN 'out_for_delivery' THEN 3 ELSE 4 END,id",
    )
    .bind(id)
    .fetch_all(&s.db)
    .await
    .map_err(ApiError::internal)?;
    d["events"] = json!(events);
    Ok(Json(d))
}
pub async fn riders(State(s): State<AppState>) -> Result<Json<Value>, ApiError> {
    let rows:Vec<Value>=sqlx::query_scalar("SELECT to_jsonb(r) || jsonb_build_object('active_deliveries',(SELECT count(*) FROM deliveries d WHERE d.rider_id=r.id AND d.status IN ('assigned','picked_up','out_for_delivery'))) FROM riders r WHERE merchant_id=$1 ORDER BY name").bind(MERCHANT).fetch_all(&s.db).await.map_err(ApiError::internal)?;
    Ok(Json(json!(rows)))
}
#[derive(Deserialize)]
pub struct Register {
    name: String,
    phone: String,
}
pub async fn register(
    State(s): State<AppState>,
    Json(i): Json<Register>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    if i.name.trim().is_empty() || i.name.len() > 120 || i.phone.len() < 7 || i.phone.len() > 30 {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "valid name and phone required",
        ));
    }
    let v: Value = sqlx::query_scalar(
        "INSERT INTO riders(merchant_id,name,phone) VALUES($1,$2,$3) RETURNING to_jsonb(riders)",
    )
    .bind(MERCHANT)
    .bind(i.name)
    .bind(i.phone)
    .fetch_one(&s.db)
    .await
    .map_err(ApiError::internal)?;
    Ok((StatusCode::CREATED, Json(v)))
}
pub async fn rider_deliveries(
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, ApiError> {
    let v: Vec<Value> = sqlx::query_scalar(&format!(
        "{SELECT} WHERE d.merchant_id=$1 AND d.rider_id=$2 ORDER BY d.created_at DESC LIMIT 200"
    ))
    .bind(MERCHANT)
    .bind(id)
    .fetch_all(&s.db)
    .await
    .map_err(ApiError::internal)?;
    Ok(Json(json!(v)))
}
pub async fn stats(State(s): State<AppState>) -> Result<Json<Value>, ApiError> {
    let v:Value=sqlx::query_scalar("SELECT jsonb_build_object('total',count(*),'pending',count(*) FILTER(WHERE status='created'),'active',count(*) FILTER(WHERE status IN ('assigned','picked_up','out_for_delivery')),'completed',count(*) FILTER(WHERE status='delivered'),'failed',count(*) FILTER(WHERE status='failed')) FROM deliveries WHERE merchant_id=$1").bind(MERCHANT).fetch_one(&s.db).await.map_err(ApiError::internal)?;
    Ok(Json(v))
}
#[derive(Deserialize)]
pub struct Assign {
    rider_id: Uuid,
}
pub async fn assign(
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
    Json(i): Json<Assign>,
) -> Result<Json<Value>, ApiError> {
    let mut tx = s.db.begin().await.map_err(ApiError::internal)?;
    let rider: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM riders WHERE id=$1 AND merchant_id=$2)")
            .bind(i.rider_id)
            .bind(MERCHANT)
            .fetch_one(&mut *tx)
            .await
            .map_err(ApiError::internal)?;
    if !rider {
        return Err(bad("rider does not belong to demo merchant"));
    }
    let result=sqlx::query("UPDATE deliveries SET rider_id=$1,status='assigned',updated_at=NOW() WHERE id=$2 AND merchant_id=$3 AND status='created'").bind(i.rider_id).bind(id).bind(MERCHANT).execute(&mut *tx).await.map_err(ApiError::internal)?;
    if result.rows_affected() != 1 {
        return Err(bad("only created demo deliveries can be assigned"));
    }
    event(&mut tx, id, "assigned").await?;
    tx.commit().await.map_err(ApiError::internal)?;
    detail(State(s), Path(id)).await
}
async fn event(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    id: Uuid,
    status: &str,
) -> Result<(), ApiError> {
    sqlx::query("INSERT INTO delivery_events(delivery_id,event_type,created_at) VALUES($1,$2,clock_timestamp())")
        .bind(id)
        .bind(status)
        .execute(&mut **tx)
        .await
        .map_err(ApiError::internal)?;
    Ok(())
}
fn valid(from: &str, to: &str) -> bool {
    matches!(
        (from, to),
        ("assigned", "picked_up")
            | ("picked_up", "out_for_delivery")
            | (
                "created" | "assigned" | "picked_up" | "out_for_delivery",
                "failed"
            )
    )
}
#[derive(Deserialize)]
pub struct Progress {
    rider_id: Uuid,
    status: String,
}
pub async fn progress(
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
    Json(i): Json<Progress>,
) -> Result<Json<Value>, ApiError> {
    let mut tx = s.db.begin().await.map_err(ApiError::internal)?;
    let from: Option<String> = sqlx::query_scalar(
        "SELECT status FROM deliveries WHERE id=$1 AND merchant_id=$2 AND rider_id=$3 FOR UPDATE",
    )
    .bind(id)
    .bind(MERCHANT)
    .bind(i.rider_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(ApiError::internal)?;
    if !from.as_deref().is_some_and(|f| valid(f, &i.status)) {
        return Err(bad(
            "invalid transition or unassigned rider; delivery requires OTP confirmation",
        ));
    }
    sqlx::query("UPDATE deliveries SET status=$1,updated_at=NOW() WHERE id=$2")
        .bind(&i.status)
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(ApiError::internal)?;
    event(&mut tx, id, &i.status).await?;
    tx.commit().await.map_err(ApiError::internal)?;
    detail(State(s), Path(id)).await
}
fn otp_hash(id: Uuid, code: &str) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(id.as_bytes()).expect("HMAC accepts UUID");
    mac.update(code.as_bytes());
    format!("{:x}", mac.finalize().into_bytes())
}
fn otp_allowed(expires: DateTime<Utc>, attempts: i32, consumed: bool) -> bool {
    expires > Utc::now() && attempts < 5 && !consumed
}
pub async fn issue(
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, ApiError> {
    let mut tx = s.db.begin().await.map_err(ApiError::internal)?;
    let status: Option<String> = sqlx::query_scalar(
        "SELECT status FROM deliveries WHERE id=$1 AND merchant_id=$2 FOR UPDATE",
    )
    .bind(id)
    .bind(MERCHANT)
    .fetch_optional(&mut *tx)
    .await
    .map_err(ApiError::internal)?;
    if status.as_deref() != Some("out_for_delivery") {
        return Err(bad("parcel must be out for delivery"));
    }
    let code = format!("{:06}", OsRng.gen_range(0..1_000_000));
    // pgcrypto bcrypt gives each low-entropy OTP a random salt and a deliberately slow hash.
    let result=sqlx::query("INSERT INTO delivery_otps(delivery_id,code_hash,expires_at) VALUES($1,crypt($2,gen_salt('bf',10)),NOW()+INTERVAL '5 minutes') ON CONFLICT(delivery_id) DO UPDATE SET code_hash=EXCLUDED.code_hash,expires_at=EXCLUDED.expires_at,attempts=0 WHERE delivery_otps.expires_at<NOW() AND delivery_otps.consumed_at IS NULL AND delivery_otps.attempts<5").bind(id).bind(otp_hash(id,&code)).execute(&mut *tx).await.map_err(ApiError::internal)?;
    if result.rows_affected() != 1 {
        return Err(bad("OTP already issued or attempt limit reached"));
    }
    tx.commit().await.map_err(ApiError::internal)?;
    Ok(Json(
        json!({"demo_otp":code,"expires_in_seconds":300,"warning":"Demo only: shared fictional data. Never use real recipient information."}),
    ))
}
#[derive(Deserialize)]
pub struct Confirm {
    rider_id: Uuid,
    code: String,
}
pub async fn confirm(
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
    Json(i): Json<Confirm>,
) -> Result<Json<Value>, ApiError> {
    if i.code.len() != 6 || !i.code.bytes().all(|b| b.is_ascii_digit()) {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "OTP must contain six digits",
        ));
    }
    let mut tx = s.db.begin().await.map_err(ApiError::internal)?;
    let status: Option<String> = sqlx::query_scalar(
        "SELECT status FROM deliveries WHERE id=$1 AND merchant_id=$2 AND rider_id=$3 FOR UPDATE",
    )
    .bind(id)
    .bind(MERCHANT)
    .bind(i.rider_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(ApiError::internal)?;
    if status.as_deref() != Some("out_for_delivery") {
        return Err(bad("delivery cannot be confirmed by this rider"));
    }
    let row=sqlx::query("SELECT expires_at,attempts,consumed_at,code_hash=crypt($2,code_hash) AS matches FROM delivery_otps WHERE delivery_id=$1 FOR UPDATE").bind(id).bind(otp_hash(id,&i.code)).fetch_optional(&mut *tx).await.map_err(ApiError::internal)?.ok_or_else(||bad("request an OTP first"))?;
    if !otp_allowed(
        row.get("expires_at"),
        row.get("attempts"),
        row.get::<Option<DateTime<Utc>>, _>("consumed_at").is_some(),
    ) {
        return Err(bad("OTP expired, consumed, or attempt limit reached"));
    }
    sqlx::query("UPDATE delivery_otps SET attempts=attempts+1 WHERE delivery_id=$1")
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(ApiError::internal)?;
    if !row.get::<bool, _>("matches") {
        tx.commit().await.map_err(ApiError::internal)?;
        return Err(ApiError::new(StatusCode::BAD_REQUEST, "incorrect OTP"));
    }
    sqlx::query("UPDATE delivery_otps SET consumed_at=NOW() WHERE delivery_id=$1")
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(ApiError::internal)?;
    sqlx::query("UPDATE deliveries SET status='delivered',updated_at=NOW() WHERE id=$1")
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(ApiError::internal)?;
    event(&mut tx, id, "delivered").await?;
    tx.commit().await.map_err(ApiError::internal)?;
    detail(State(s), Path(id)).await
}
pub async fn openapi() -> Json<Value> {
    Json(serde_json::from_str(include_str!("../../docs/openapi.json")).expect("valid OpenAPI"))
}
pub async fn docs() -> Html<&'static str> {
    Html(
        r#"<!doctype html><html><head><title>SmartRide API</title><link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css"></head><body><div id="swagger-ui"></div><script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js"></script><script>SwaggerUIBundle({url:'/openapi.json',dom_id:'#swagger-ui'})</script></body></html>"#,
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn transitions() {
        assert!(valid("assigned", "picked_up"));
        assert!(valid("picked_up", "out_for_delivery"));
        for from in [
            "created",
            "assigned",
            "picked_up",
            "out_for_delivery",
            "delivered",
            "failed",
        ] {
            assert!(!valid(from, "delivered"));
            assert!(!valid(from, from));
        }
        assert!(!valid("delivered", "failed"));
        assert!(valid("out_for_delivery", "failed"));
    }
    #[test]
    fn otp_policy() {
        assert!(otp_allowed(
            Utc::now() + chrono::Duration::minutes(1),
            0,
            false
        ));
        assert!(!otp_allowed(
            Utc::now() - chrono::Duration::seconds(1),
            0,
            false
        ));
        assert!(!otp_allowed(
            Utc::now() + chrono::Duration::minutes(1),
            5,
            false
        ));
        assert!(!otp_allowed(
            Utc::now() + chrono::Duration::minutes(1),
            0,
            true
        ));
        assert_ne!(otp_hash(MERCHANT, "123456"), otp_hash(MERCHANT, "123457"));
    }
}
