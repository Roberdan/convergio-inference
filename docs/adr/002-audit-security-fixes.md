# ADR-002: Security Audit Fixes

**Status:** Accepted  
**Date:** 2025-07-22  
**Author:** Security Audit (Copilot)

## Context

A comprehensive security audit was performed on `convergio-inference` covering
OWASP Top 10, SSRF, injection, DoS, and data leakage risks specific to an LLM
inference router that calls external model APIs.

## Findings & Fixes

### 1. CRITICAL — MLX Python Code Injection

**Risk:** `model_name` was interpolated directly into a Python f-string in
`backend_mlx.rs`. A crafted model name (e.g., containing `"`) could break out
of the string and execute arbitrary Python code.

**Fix:** Validate `model_name` against an allowlist of safe characters
(alphanumeric, `-/._ `), enforce 256-char max length, and pass both
`model_name` and `prompt` as JSON-serialized strings parsed via `json.loads()`
in the Python script — eliminating string interpolation entirely.

### 2. HIGH — No Input Size Limits (DoS)

**Risk:** `InferenceRequest.prompt` had no length limit; `max_tokens` accepted
up to `u32::MAX`. An attacker could send multi-GB prompts or request billions
of tokens, causing resource exhaustion.

**Fix:**
- Prompt limit: 128 KiB (`MAX_PROMPT_BYTES`)
- max_tokens clamped to 32,768 (`MAX_TOKENS_LIMIT`)
- Request body limit: 256 KiB via `axum::DefaultBodyLimit`
- agent_id/org_id capped at 256 chars

### 3. MEDIUM — SQL Column Name Interpolation

**Risk:** `costs_by_scope()` used `format!()` with a `col` parameter for the
SQL column name. While callers always passed hardcoded strings, this pattern is
fragile and could lead to SQL injection if a future caller passes user input.

**Fix:** Whitelist `col` to `"agent_id"` | `"org_id"`, rejecting all others.

### 4. MEDIUM — SSRF via Configured Endpoint URLs

**Risk:** `backend.rs` sends HTTP requests to whatever URL is configured in
model endpoints. If an attacker controls the config file or env vars, they
could redirect inference calls to internal services.

**Fix:** Validate URL scheme before requests — only `http://` and `https://`
are allowed, rejecting `file://`, `ftp://`, custom schemes, etc.

### 5. MEDIUM — Unbounded Metrics Memory

**Risk:** `MetricsCollector` retained all entries within a 7-day window with
no hard cap. Under sustained load, this could exhaust memory.

**Fix:** Added `MAX_METRICS_ENTRIES = 100,000` hard cap with oldest-first
eviction.

### 6. LOW — Error Body Information Leakage

**Risk:** Full upstream API error bodies were returned to callers, potentially
exposing internal infrastructure details, API keys in error messages, or
debug information from cloud providers.

**Fix:** Truncate error response bodies to 200 chars before including in
error messages.

## Accepted Risks

### Auth/AuthZ on Routes

Routes (`/api/inference/*`) have no authentication in this crate. This is
**by design** — auth is enforced at the daemon gateway layer. Documented here
so future auditors don't flag this redundantly.

### Config File Path from Environment

`CONVERGIO_MODELS_CONFIG` allows reading arbitrary TOML files. This is
acceptable because:
- The daemon runs in a controlled environment
- The env var is set by the operator, not by users
- The file is parsed as TOML, not executed

## Decision

All CRITICAL and HIGH findings have been fixed. MEDIUM findings have been
hardened. The crate now meets baseline security requirements for an LLM
inference router handling external API calls.

## Consequences

- Model names with special characters will be rejected (unlikely in practice)
- Very large prompts (>128 KiB) will be rejected at the API layer
- max_tokens requests above 32K will be silently clamped
- Metrics memory usage is bounded regardless of load
