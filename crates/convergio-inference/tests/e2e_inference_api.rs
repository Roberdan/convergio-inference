//! E2E tests: inference API routes (costs, routing-decision, complete).

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use convergio_inference::metrics::MetricsCollector;
use convergio_inference::router::ModelRouter;
use convergio_inference::routes::{inference_routes, InferenceState};
use convergio_inference::types::{InferenceTier, ModelEndpoint, ModelProvider};
use tokio::sync::RwLock;
use tower::ServiceExt;

fn setup_db() -> convergio_db::pool::ConnPool {
    let pool = convergio_db::pool::create_memory_pool().unwrap();
    let conn = pool.get().unwrap();
    convergio_db::migration::ensure_registry(&conn).unwrap();
    convergio_db::migration::apply_migrations(
        &conn,
        "inference",
        &convergio_inference::schema::migrations(),
    )
    .unwrap();
    pool
}

fn local_model(name: &str) -> ModelEndpoint {
    ModelEndpoint {
        name: name.into(),
        provider: ModelProvider::Local,
        url: String::new(),
        cost_per_1k_input: 0.0,
        cost_per_1k_output: 0.0,
        tier_range: (InferenceTier::T1Trivial, InferenceTier::T4Critical),
        healthy: true,
    }
}

fn build_app(pool: convergio_db::pool::ConnPool, router: ModelRouter) -> axum::Router {
    let state = Arc::new(InferenceState {
        pool,
        router: Arc::new(RwLock::new(router)),
        metrics: Arc::new(RwLock::new(MetricsCollector::new())),
    });
    inference_routes(state)
}

async fn body_json(resp: axum::http::Response<Body>) -> serde_json::Value {
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

fn get(uri: &str) -> Request<Body> {
    Request::builder().uri(uri).body(Body::empty()).unwrap()
}

fn post_json(uri: &str, body: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(body.to_owned()))
        .unwrap()
}

// ── route tests ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn api_routing_decision_returns_json() {
    let pool = setup_db();
    let mut router = ModelRouter::new();
    router.register_model(local_model("test-model"));
    let app = build_app(pool, router);

    let resp = app
        .oneshot(get("/api/inference/routing-decision?prompt=hello&tier=t1"))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert_eq!(json["decision"]["selected_model"], "test-model");
    assert!(json["model_metrics"].is_array());
}

#[tokio::test]
async fn api_costs_returns_zero_for_unknown_agent() {
    let pool = setup_db();
    let app = build_app(pool, ModelRouter::new());

    let resp = app
        .oneshot(get("/api/inference/costs?agent_id=nobody"))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    let summaries = json["summaries"].as_array().unwrap();
    assert_eq!(summaries.len(), 1);
    assert_eq!(summaries[0]["total_tokens"], 0);
}

#[tokio::test]
async fn api_complete_returns_echo_without_backend() {
    let pool = setup_db();
    let mut router = ModelRouter::new();
    router.register_model(local_model("echo-model"));
    let app = build_app(pool, router);

    let body = serde_json::json!({
        "prompt": "test prompt",
        "max_tokens": 64,
        "agent_id": "e2e-agent"
    });
    let resp = app
        .oneshot(post_json("/api/inference/complete", &body.to_string()))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert!(json["response"]["content"]
        .as_str()
        .unwrap()
        .contains("[echo:echo-model]"));
    assert_eq!(json["decision"]["selected_model"], "echo-model");
}
