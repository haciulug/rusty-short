use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Redirect, Response},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::{
    domain::{CreateLinkRequest, ErrorResponse, LinkResponse, LinkStats, AnalyticsSummary, LinkAnalytics},
    services::{LinkService, QrService, AnalyticsService},
};

#[derive(Clone)]
pub struct AppState {
    pub link_service: Arc<LinkService>,
    pub repository: crate::repository::LinkRepository,
}

pub async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

pub async fn create_short_link(
    State(state): State<AppState>,
    Json(request): Json<CreateLinkRequest>,
) -> Result<Json<LinkResponse>, AppError> {
    let response = state.link_service.create_link(request).await?;
    Ok(Json(response))
}

pub async fn redirect_to_original(
    State(state): State<AppState>,
    Path(key): Path<String>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let _link = state
        .link_service
        .get_link(&key)
        .await?
        .ok_or_else(|| AppError::NotFound("Link not found".to_string()))?;

    let referrer = headers
        .get(header::REFERER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    
    let ip_hash = if let Some(x_forwarded_for) = headers.get("x-forwarded-for") {
        x_forwarded_for
            .to_str()
            .ok()
            .and_then(|s| s.split(',').next())
            .map(|ip| AnalyticsService::hash_ip(ip.trim()))
    } else if let Some(x_real_ip) = headers.get("x-real-ip") {
        x_real_ip
            .to_str()
            .ok()
            .map(|ip| AnalyticsService::hash_ip(ip))
    } else {
        None
    };

    tokio::spawn({
        let service = state.link_service.clone();
        let key = key.clone();
        let link_id = _link.id;
        async move {
            let _ = service.increment_click(&key).await;
            let _ = service
                .record_analytics(link_id, referrer, user_agent, ip_hash)
                .await;
        }
    });

    Ok(Redirect::permanent(&_link.original_url))
}

pub async fn get_link_stats(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> Result<Json<LinkStats>, AppError> {
    let stats = state
        .link_service
        .get_stats(&key)
        .await?
        .ok_or_else(|| AppError::NotFound("Link not found".to_string()))?;

    Ok(Json(stats))
}

pub async fn generate_qr_code(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let _link = state
        .link_service
        .get_link(&key)
        .await?
        .ok_or_else(|| AppError::NotFound("Link not found".to_string()))?;

    let short_url = format!("{}/{}", state.link_service.base_url, key);
    let qr_data = QrService::generate_qr_code(&short_url)?;

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "image/png")],
        qr_data,
    ))
}

pub async fn delete_link(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> Result<StatusCode, AppError> {
    let deleted = state.link_service.delete_link(&key).await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound("Link not found".to_string()))
    }
}

#[derive(Deserialize)]
pub struct ListQuery {
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default)]
    offset: i64,
}

fn default_limit() -> i64 {
    50
}

pub async fn list_links(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<LinkResponse>>, AppError> {
    let limit = query.limit.min(100);
    let links = state.link_service.list_links(limit, query.offset).await?;
    Ok(Json(links))
}

#[derive(Deserialize)]
pub struct AnalyticsQuery {
    #[serde(default = "default_days")]
    days: i32,
}

fn default_days() -> i32 {
    30
}

pub async fn get_analytics_summary(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Query(query): Query<AnalyticsQuery>,
) -> Result<Json<AnalyticsSummary>, AppError> {
    let days = query.days.min(365);
    let summary = state
        .link_service
        .get_analytics_summary(&key, days)
        .await?
        .ok_or_else(|| AppError::NotFound("Link not found".to_string()))?;

    Ok(Json(summary))
}

pub async fn get_detailed_analytics(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<LinkAnalytics>>, AppError> {
    let limit = query.limit.min(1000);
    
    if !state.link_service.get_link(&key).await?.is_some() {
        return Err(AppError::NotFound("Link not found".to_string()));
    }
    
    let analytics = state.repository.get_analytics(&key, limit).await?;
    Ok(Json(analytics))
}

pub enum AppError {
    Internal(anyhow::Error),
    NotFound(String),
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Internal(err)
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Internal(err.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Internal(err) => {
                tracing::error!("Internal error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorResponse {
                        error: "Internal server error".to_string(),
                        details: None,
                    },
                )
            }
            AppError::NotFound(msg) => (
                StatusCode::NOT_FOUND,
                ErrorResponse {
                    error: msg,
                    details: None,
                },
            ),
        };

        (status, Json(error_message)).into_response()
    }
}

