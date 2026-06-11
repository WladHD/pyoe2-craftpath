//! Content negotiation between `application/json` (canonical proto3 JSON via
//! pbjson) and `application/x-protobuf` (prost binary). One generated type
//! serves both encodings.

use axum::body::Bytes;
use axum::extract::{FromRequest, FromRequestParts, Request};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use prost::Message;
use serde::Serialize;
use serde::de::DeserializeOwned;

pub const CONTENT_TYPE_PROTOBUF: &str = "application/x-protobuf";
const CONTENT_TYPE_PROTOBUF_ALT: &str = "application/protobuf";

fn is_protobuf(value: &str) -> bool {
    let essence = value.split(';').next().unwrap_or("").trim();
    essence.eq_ignore_ascii_case(CONTENT_TYPE_PROTOBUF)
        || essence.eq_ignore_ascii_case(CONTENT_TYPE_PROTOBUF_ALT)
}

fn is_json(value: &str) -> bool {
    let essence = value.split(';').next().unwrap_or("").trim();
    essence.eq_ignore_ascii_case("application/json")
}

/// Request-body extractor accepting both encodings based on `Content-Type`
/// (absent Content-Type is treated as JSON).
pub struct ProtoOrJson<T>(pub T);

impl<S, T> FromRequest<S> for ProtoOrJson<T>
where
    S: Send + Sync,
    T: Message + Default + DeserializeOwned,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let content_type = req
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/json")
            .to_string();

        let bytes = Bytes::from_request(req, state).await.map_err(|e| {
            ApiError::new(StatusCode::BAD_REQUEST, "INVALID_REQUEST", &e.to_string())
        })?;

        if is_protobuf(&content_type) {
            let value = T::decode(bytes.as_ref()).map_err(|e| {
                ApiError::new(
                    StatusCode::BAD_REQUEST,
                    "INVALID_REQUEST",
                    &format!("invalid protobuf body: {e}"),
                )
            })?;
            Ok(ProtoOrJson(value))
        } else if is_json(&content_type) {
            let value = serde_json::from_slice::<T>(&bytes).map_err(|e| {
                ApiError::new(
                    StatusCode::BAD_REQUEST,
                    "INVALID_REQUEST",
                    &format!("invalid JSON body: {e}"),
                )
            })?;
            Ok(ProtoOrJson(value))
        } else {
            Err(ApiError::new(
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "INVALID_REQUEST",
                &format!(
                    "unsupported Content-Type '{content_type}'; use application/json or {CONTENT_TYPE_PROTOBUF}"
                ),
            ))
        }
    }
}

/// Whether the client asked for protobuf responses via `Accept`.
#[derive(Clone, Copy, Debug)]
pub struct WantsProto(pub bool);

impl<S: Send + Sync> FromRequestParts<S> for WantsProto {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        Ok(WantsProto(accepts_proto(&parts.headers)))
    }
}

pub fn accepts_proto(headers: &HeaderMap) -> bool {
    headers
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .map(|accept| accept.split(',').any(is_protobuf))
        .unwrap_or(false)
}

/// Encode a response message in the encoding the client asked for.
pub fn respond<T: Message + Serialize>(
    wants_proto: WantsProto,
    status: StatusCode,
    value: &T,
) -> Response {
    if wants_proto.0 {
        (
            status,
            [(header::CONTENT_TYPE, CONTENT_TYPE_PROTOBUF)],
            value.encode_to_vec(),
        )
            .into_response()
    } else {
        match serde_json::to_vec(value) {
            Ok(body) => (
                status,
                [(header::CONTENT_TYPE, "application/json")],
                body,
            )
                .into_response(),
            Err(e) => ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL",
                &format!("response serialization failed: {e}"),
            )
            .into_response(),
        }
    }
}

/// Error shape mirroring `craftpath.v1.Error`; rendered as JSON.
#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub code: &'static str,
    pub message: String,
}

impl ApiError {
    pub fn new(status: StatusCode, code: &'static str, message: &str) -> Self {
        Self {
            status,
            code,
            message: message.to_string(),
        }
    }

    pub fn internal(err: impl std::fmt::Display) -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "INTERNAL",
            &err.to_string(),
        )
    }

    pub fn not_found(job_id: &str) -> Self {
        Self::new(
            StatusCode::NOT_FOUND,
            "JOB_NOT_FOUND",
            &format!("no job with id '{job_id}' (it may have expired)"),
        )
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = craftpath_proto::v1::Error {
            code: self.code.to_string(),
            message: self.message,
            details: Default::default(),
        };
        (
            self.status,
            [(header::CONTENT_TYPE, "application/json")],
            serde_json::to_vec(&body).unwrap_or_default(),
        )
            .into_response()
    }
}
