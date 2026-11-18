use crate::GatewayState;
use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use std::sync::Arc;

pub struct McpGateway;

impl McpGateway {
    pub async fn health() -> StatusCode {
        StatusCode::NO_CONTENT
    }

    pub async fn post(
        State(state): State<Arc<GatewayState>>,
        headers: HeaderMap,
        body: Bytes,
    ) -> Response {
        if headers.contains_key(header::ORIGIN) {
            return StatusCode::FORBIDDEN.into_response();
        }
        let Some(authorization) = headers.get(header::AUTHORIZATION) else {
            return StatusCode::UNAUTHORIZED.into_response();
        };
        let accept = headers
            .get(header::ACCEPT)
            .cloned()
            .unwrap_or_else(|| HeaderValue::from_static("application/json, text/event-stream"));
        let content_type = headers
            .get(header::CONTENT_TYPE)
            .cloned()
            .unwrap_or_else(|| HeaderValue::from_static("application/json"));
        let protocol = headers.get("mcp-protocol-version").cloned();

        let Ok(_permit) = state.permits.clone().try_acquire_owned() else {
            return StatusCode::TOO_MANY_REQUESTS.into_response();
        };

        let mut request = state
            .client
            .post(state.upstream.clone())
            .header(header::AUTHORIZATION, authorization)
            .header(header::ACCEPT, accept)
            .header(header::CONTENT_TYPE, content_type)
            .body(body);
        if let Some(protocol) = protocol {
            request = request.header("mcp-protocol-version", protocol);
        }

        let response = match request.send().await {
            Ok(response) => response,
            Err(_) => return StatusCode::BAD_GATEWAY.into_response(),
        };
        let status =
            StatusCode::from_u16(response.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .cloned()
            .unwrap_or_else(|| HeaderValue::from_static("application/json"));
        let body = match response.bytes().await {
            Ok(body) if body.len() <= 60 * 1024 * 1024 => body,
            _ => return StatusCode::BAD_GATEWAY.into_response(),
        };
        let mut result = (status, body).into_response();
        result
            .headers_mut()
            .insert(header::CONTENT_TYPE, content_type);
        result
    }
}
