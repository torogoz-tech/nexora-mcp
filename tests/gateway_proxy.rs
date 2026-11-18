use axum::{
    Router,
    body::Bytes,
    http::{HeaderMap, StatusCode},
    routing::post,
};
use nexora_mcp::{GatewayState, McpGateway};
use std::sync::Arc;

async fn capture_request(headers: HeaderMap, body: Bytes) -> (StatusCode, String) {
    let authorization = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    let protocol = headers
        .get("mcp-protocol-version")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    let content = String::from_utf8_lossy(&body);
    (
        StatusCode::ACCEPTED,
        format!("{authorization}|{protocol}|{content}"),
    )
}

#[tokio::test]
async fn rejects_browser_or_unauthenticated_requests_and_forwards_mcp_calls() {
    let upstream_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let upstream_address = upstream_listener.local_addr().unwrap();
    let upstream = Router::new().route("/mcp", post(capture_request));
    let upstream_task = tokio::spawn(async move {
        axum::serve(upstream_listener, upstream).await.unwrap();
    });

    let gateway_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let gateway_address = gateway_listener.local_addr().unwrap();
    let upstream_url = format!("http://{upstream_address}/mcp").parse().unwrap();
    let state = Arc::new(GatewayState::new(upstream_url).unwrap());
    let gateway = Router::new()
        .route("/mcp", post(McpGateway::post))
        .with_state(state);
    let gateway_task = tokio::spawn(async move {
        axum::serve(gateway_listener, gateway).await.unwrap();
    });
    let client = reqwest::Client::new();
    let endpoint = format!("http://{gateway_address}/mcp");

    let unauthenticated = client.post(&endpoint).body("{}").send().await.unwrap();
    assert_eq!(unauthenticated.status(), StatusCode::UNAUTHORIZED);

    let browser_request = client
        .post(&endpoint)
        .header("authorization", "Bearer test-key")
        .header("origin", "https://web.example.invalid")
        .body("{}")
        .send()
        .await
        .unwrap();
    assert_eq!(browser_request.status(), StatusCode::FORBIDDEN);

    let forwarded = client
        .post(&endpoint)
        .header("authorization", "Bearer test-key")
        .header("mcp-protocol-version", "2025-03-26")
        .header("content-type", "application/json")
        .body("{\"jsonrpc\":\"2.0\"}")
        .send()
        .await
        .unwrap();
    assert_eq!(forwarded.status(), StatusCode::ACCEPTED);
    assert_eq!(
        forwarded.text().await.unwrap(),
        "Bearer test-key|2025-03-26|{\"jsonrpc\":\"2.0\"}"
    );

    gateway_task.abort();
    upstream_task.abort();
}
