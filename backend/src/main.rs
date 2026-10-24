mod postcode;
mod providers;

use std::{env, sync::Arc};

use axum::extract::Path;
use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use postcode::{PostcodeError, PostcodeProvider, ResolvedAddress};
use providers::{mock::MockPostcodeProvider, nipost::NipostProvider};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    db: PgPool,
    postcode: Arc<dyn PostcodeProvider>,
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }

    fn internal(error: impl std::fmt::Display) -> Self {
        tracing::error!(error = %error, "internal error");
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, "internal server error")
    }
}

impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (self.status, Json(json!({ "error": self.message }))).into_response()
    }
}

impl From<PostcodeError> for ApiError {
    fn from(error: PostcodeError) -> Self {
        match error {
            PostcodeError::InvalidPostcode => Self::new(StatusCode::BAD_REQUEST, error.to_string()),
            PostcodeError::NotFound => Self::new(StatusCode::NOT_FOUND, error.to_string()),
            PostcodeError::Unavailable(_) => {
                Self::new(StatusCode::BAD_GATEWAY, "postcode service unavailable")
            }
        }
    }
}

#[derive(Deserialize)]
struct ResolveQuery {
    postcode: String,
}

async fn health(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    sqlx::query("SELECT 1")
        .execute(&state.db)
        .await
        .map_err(ApiError::internal)?;

    Ok(Json(json!({
        "status": "ok",
        "service": "smartride-api"
    })))
}

async fn resolve_address(
    State(state): State<AppState>,
    Query(query): Query<ResolveQuery>,
) -> Result<Json<ResolvedAddress>, ApiError> {
    let address = state.postcode.resolve(&query.postcode).await?;
    Ok(Json(address))
}

#[derive(Debug, Deserialize)]
struct CreateDelivery {
    merchant_id: Uuid,
    external_reference: Option<String>,
    recipient_name: String,
    recipient_phone: String,
    destination_postcode: String,
    delivery_instructions: Option<String>,
    parcel_description: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct DeliveryResponse {
    id: Uuid,
    merchant_id: Uuid,
    address_id: Uuid,
    status: String,
    created_at: DateTime<Utc>,
}

async fn create_delivery(
    State(state): State<AppState>,
    Json(input): Json<CreateDelivery>,
) -> Result<(StatusCode, Json<DeliveryResponse>), ApiError> {
    if input.recipient_name.trim().is_empty() || input.recipient_phone.trim().is_empty() {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "recipient name and phone are required",
        ));
    }

    let address = state.postcode.resolve(&input.destination_postcode).await?;

    let mut tx = state.db.begin().await.map_err(ApiError::internal)?;

    let address_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO addresses (
            postcode,
            formatted_address,
            provider,
            provider_data
        )
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(&address.postcode)
    .bind(&address.formatted_address)
    .bind(&address.provider)
    .bind(&address.provider_data)
    .fetch_one(&mut *tx)
    .await
    .map_err(ApiError::internal)?;

    let delivery: DeliveryResponse = sqlx::query_as(
        r#"
        INSERT INTO deliveries (
            merchant_id,
            address_id,
            external_reference,
            recipient_name,
            recipient_phone,
            delivery_instructions,
            parcel_description
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING
            id,
            merchant_id,
            address_id,
            status,
            created_at
        "#,
    )
    .bind(input.merchant_id)
    .bind(address_id)
    .bind(input.external_reference)
    .bind(input.recipient_name)
    .bind(input.recipient_phone)
    .bind(input.delivery_instructions)
    .bind(input.parcel_description)
    .fetch_one(&mut *tx)
    .await
    .map_err(ApiError::internal)?;

    sqlx::query(
        r#"
        INSERT INTO delivery_events (
            delivery_id,
            event_type,
            metadata
        )
        VALUES ($1, 'created', $2)
        "#,
    )
    .bind(delivery.id)
    .bind(json!({
        "postcode": address.postcode,
        "provider": address.provider
    }))
    .execute(&mut *tx)
    .await
    .map_err(ApiError::internal)?;

    tx.commit().await.map_err(ApiError::internal)?;

    Ok((StatusCode::CREATED, Json(delivery)))
}

async fn get_delivery(
    State(state): State<AppState>,
    Path(delivery_id): Path<Uuid>,
) -> Result<Json<DeliveryResponse>, ApiError> {
    let delivery = sqlx::query_as::<_, DeliveryResponse>(
        r#"
        SELECT
            id,
            merchant_id,
            address_id,
            status,
            created_at
        FROM deliveries
        WHERE id = $1
        "#,
    )
    .bind(delivery_id)
    .fetch_optional(&state.db)
    .await
    .map_err(ApiError::internal)?;

    match delivery {
        Some(delivery) => Ok(Json(delivery)),
        None => Err(ApiError::new(StatusCode::NOT_FOUND, "delivery not found")),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let database_url = env::var("DATABASE_URL")?;

    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&db).await?;

    let provider_name = env::var("POSTCODE_PROVIDER").unwrap_or_else(|_| "mock".into());

    let postcode: Arc<dyn PostcodeProvider> = match provider_name.as_str() {
        "mock" => Arc::new(MockPostcodeProvider),
        "nipost" => Arc::new(NipostProvider::new(
            env::var("NIPOST_API_BASE_URL")?,
            env::var("NIPOST_API_KEY")?,
        )?),
        other => {
            return Err(format!("Unknown POSTCODE_PROVIDER: {other}").into());
        }
    };

    let state = AppState { db, postcode };

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/addresses/resolve", get(resolve_address))
        .route("/api/v1/deliveries", post(create_delivery))
        .route("/api/v1/deliveries/{id}", get(get_delivery))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let port: u16 = env::var("PORT").unwrap_or_else(|_| "8080".into()).parse()?;

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}")).await?;

    tracing::info!(port, "SmartRide API listening");

    axum::serve(listener, app).await?;

    Ok(())
}
