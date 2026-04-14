//! E2E tests: model routing, classifier, and budget enforcement.

use convergio_inference::budget::{self, BudgetConfig};
use convergio_inference::classifier;
use convergio_inference::router::ModelRouter;
use convergio_inference::types::{
    InferenceConstraints, InferenceRequest, InferenceTier, ModelEndpoint, ModelProvider,
};

fn local_model(name: &str, cost: f64, tier: (InferenceTier, InferenceTier)) -> ModelEndpoint {
    ModelEndpoint {
        name: name.into(),
        provider: ModelProvider::Local,
        url: String::new(),
        cost_per_1k_input: cost,
        cost_per_1k_output: cost,
        tier_range: tier,
        healthy: true,
    }
}

fn cloud_model(name: &str, cost: f64, tier: (InferenceTier, InferenceTier)) -> ModelEndpoint {
    ModelEndpoint {
        name: name.into(),
        provider: ModelProvider::Cloud,
        url: "https://api.example.com/v1".into(),
        cost_per_1k_input: cost,
        cost_per_1k_output: cost,
        tier_range: tier,
        healthy: true,
    }
}

fn make_request(prompt: &str, hint: Option<InferenceTier>) -> InferenceRequest {
    InferenceRequest {
        prompt: prompt.into(),
        max_tokens: 256,
        tier_hint: hint,
        model_override: None,
        agent_id: "test-agent".into(),
        org_id: Some("test-org".into()),
        plan_id: Some(1),
        constraints: InferenceConstraints::default(),
    }
}

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

// ── routing ──────────────────────────────────────────────────────────────────

#[tokio::test]
async fn route_selects_cheapest_local_model() {
    let mut router = ModelRouter::new();
    router.register_model(local_model(
        "llama-cheap",
        0.0,
        (InferenceTier::T1Trivial, InferenceTier::T4Critical),
    ));
    router.register_model(cloud_model(
        "gpt-4",
        0.03,
        (InferenceTier::T1Trivial, InferenceTier::T4Critical),
    ));
    let req = make_request("hello", Some(InferenceTier::T1Trivial));
    let (resp, decision) = router.route(&req, false).unwrap();
    assert_eq!(decision.selected_model, "llama-cheap");
    assert!(resp.content.contains("[echo:llama-cheap]"));
    assert_eq!(decision.fallback_chain, vec!["gpt-4"]);
}

#[tokio::test]
async fn route_skips_unhealthy_model() {
    let mut router = ModelRouter::new();
    let mut sick = local_model(
        "sick-model",
        0.0,
        (InferenceTier::T1Trivial, InferenceTier::T4Critical),
    );
    sick.healthy = false;
    router.register_model(sick);
    router.register_model(cloud_model(
        "backup-cloud",
        0.01,
        (InferenceTier::T1Trivial, InferenceTier::T4Critical),
    ));
    let req = make_request("test", Some(InferenceTier::T1Trivial));
    let (_, decision) = router.route(&req, false).unwrap();
    assert_eq!(decision.selected_model, "backup-cloud");
}

#[tokio::test]
async fn route_errors_when_no_healthy_model() {
    let mut router = ModelRouter::new();
    let mut m = local_model(
        "only",
        0.0,
        (InferenceTier::T1Trivial, InferenceTier::T1Trivial),
    );
    m.healthy = false;
    router.register_model(m);
    let req = make_request("test", Some(InferenceTier::T1Trivial));
    assert!(router
        .route(&req, false)
        .unwrap_err()
        .contains("no healthy model"));
}

#[tokio::test]
async fn route_respects_max_cost_constraint() {
    let mut router = ModelRouter::new();
    router.register_model(cloud_model(
        "expensive",
        0.10,
        (InferenceTier::T1Trivial, InferenceTier::T4Critical),
    ));
    router.register_model(local_model(
        "cheap",
        0.001,
        (InferenceTier::T1Trivial, InferenceTier::T4Critical),
    ));
    let mut req = make_request("test", Some(InferenceTier::T2Standard));
    req.constraints.max_cost = Some(0.005);
    let (_, decision) = router.route(&req, false).unwrap();
    assert_eq!(decision.selected_model, "cheap");
}

// ── classifier ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn classifier_respects_hint_override() {
    let req = make_request("hello", Some(InferenceTier::T4Critical));
    assert_eq!(classifier::classify(&req), InferenceTier::T4Critical);
}

#[tokio::test]
async fn classifier_boosts_security_keyword() {
    let req = make_request("review the security architecture design", None);
    assert!(classifier::classify(&req) >= InferenceTier::T3Complex);
}

// ── budget ───────────────────────────────────────────────────────────────────

#[tokio::test]
async fn budget_records_and_aggregates_costs() {
    let pool = setup_db();
    let conn = pool.get().unwrap();
    let record = convergio_inference::types::CostRecord {
        agent_id: "agent-a".into(),
        org_id: Some("org-1".into()),
        plan_id: Some(42),
        model: "llama3".into(),
        tokens_input: 500,
        tokens_output: 200,
        cost_usd: 0.05,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    budget::record_cost(&conn, &record).unwrap();
    let summary = budget::plan_costs(&conn, 42).unwrap();
    assert_eq!(summary.total_tokens, 700);
    assert!((summary.total_cost_usd - 0.05).abs() < 0.001);
    assert!(summary.models_used.contains(&"llama3".to_string()));
}

#[tokio::test]
async fn budget_downgrade_triggers_at_threshold() {
    let pool = setup_db();
    let conn = pool.get().unwrap();
    let record = convergio_inference::types::CostRecord {
        agent_id: "spender".into(),
        org_id: None,
        plan_id: None,
        model: "gpt-4".into(),
        tokens_input: 50000,
        tokens_output: 50000,
        cost_usd: 0.90,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    budget::record_cost(&conn, &record).unwrap();
    let config = BudgetConfig {
        daily_token_limit: 10_000_000,
        daily_cost_limit_usd: 1.0,
        downgrade_threshold: 0.8,
    };
    assert!(budget::should_downgrade(&conn, "spender", &config).unwrap());
}

#[test]
fn budget_downgrade_tier_steps_down() {
    assert_eq!(
        budget::downgrade_tier(InferenceTier::T4Critical),
        InferenceTier::T3Complex
    );
    assert_eq!(
        budget::downgrade_tier(InferenceTier::T1Trivial),
        InferenceTier::T1Trivial
    );
}
