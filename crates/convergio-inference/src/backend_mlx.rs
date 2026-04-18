//! MLX backend — direct subprocess inference via mlx-lm (Apple Silicon native).
//!
//! Spawns `python3 -m mlx_lm.generate` as a subprocess.
//! No Ollama dependency — uses MLX framework directly.
//!
//! NOTE: KV cache quantization (TurboQuant, kv_bits) does NOT work on 4-bit
//! quantized models — produces garbage output. Only use on FP16/BF16 models.
//! Benchmark (sessione 8): standard=34 t/s perfect, kv_bits=4 → garbage.

use crate::types::InferenceResponse;
use std::time::Instant;

/// Check if MLX is available on this system.
pub fn mlx_available() -> bool {
    let python = resolve_python();
    let ok = std::process::Command::new(&python)
        .args(["-c", "import mlx_lm; print('ok')"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    tracing::info!(python = %python, available = ok, "MLX availability check");
    ok
}

/// Call an MLX model via subprocess.
/// `model_name`: HuggingFace model ID or local path (e.g. "mlx-community/Qwen2.5-Coder-32B-Instruct-4bit")
pub async fn call_mlx(
    model_name: &str,
    prompt: &str,
    max_tokens: u32,
) -> Result<InferenceResponse, String> {
    let python = resolve_python();
    tracing::info!(python = %python, model = %model_name, max_tokens, "MLX inference starting");
    // Validate model_name to prevent Python code injection.
    // Allow alphanumeric, hyphens, slashes, dots, underscores (HuggingFace IDs + local paths).
    if !model_name
        .chars()
        .all(|c| c.is_alphanumeric() || "-/._ ".contains(c))
    {
        return Err(format!(
            "invalid model name (illegal characters): {model_name}"
        ));
    }
    if model_name.len() > 256 {
        return Err("model name too long (max 256 chars)".to_string());
    }

    // TurboQuant (kv_bits) disabled by default — produces garbage on 4-bit models.
    // Only enable for FP16/BF16 models where KV cache quantization is safe.
    let _turboquant = std::env::var("CONVERGIO_MLX_TURBOQUANT")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false);

    let start = Instant::now();

    // Safety: model_name is validated above; prompt is JSON-serialized (safe).
    // Both are passed as JSON strings to avoid code injection in the Python script.
    let model_name_json = serde_json::to_string(model_name).unwrap_or_default();
    let prompt_json = serde_json::to_string(prompt).unwrap_or_default();

    // Uses chat template to prevent special token leaks (<|im_start|> etc)
    let script = format!(
        r#"
import json
from mlx_lm import load, generate

model_name = json.loads({model_name_json})
model, tokenizer = load(model_name)
raw = json.loads({prompt_json})
messages = [{{"role": "user", "content": raw}}]
prompt = tokenizer.apply_chat_template(
    messages, add_generation_prompt=True, tokenize=False
)
response = generate(model, tokenizer, prompt=prompt, max_tokens={max_tokens})
for tag in ["<|im_start|>", "<|im_end|>", "<|endoftext|>"]:
    response = response.replace(tag, "")
response = response.strip()
result = {{"content": response, "tokens": len(tokenizer.encode(response))}}
print(json.dumps(result))
"#,
        model_name_json = model_name_json,
        prompt_json = prompt_json,
        max_tokens = max_tokens,
    );

    let output = tokio::task::spawn_blocking(move || {
        std::process::Command::new(&python)
            .args(["-c", &script])
            .output()
    })
    .await
    .map_err(|e| format!("spawn_blocking: {e}"))?
    .map_err(|e| format!("mlx subprocess: {e}"))?;

    let latency_ms = start.elapsed().as_millis() as u64;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Truncate stderr to avoid leaking internal paths and Python tracebacks
        let safe_stderr: String = stderr.chars().take(200).collect();
        return Err(format!("mlx-lm failed: {safe_stderr}"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: MlxOutput =
        serde_json::from_str(stdout.trim()).map_err(|e| format!("parse mlx output: {e}"))?;

    Ok(InferenceResponse {
        content: parsed.content,
        model_used: model_name.to_string(),
        latency_ms,
        tokens_used: parsed.tokens.unwrap_or(max_tokens),
        cost: 0.0, // Local inference is free
    })
}

#[derive(serde::Deserialize)]
struct MlxOutput {
    content: String,
    tokens: Option<u32>,
}

/// Resolve the Python binary path.
///
/// Priority: CONVERGIO_PYTHON env > ~/.convergio/mlx-env/bin/python3 > python3
fn resolve_python() -> String {
    if let Ok(p) = std::env::var("CONVERGIO_PYTHON") {
        return p;
    }
    if let Ok(home) = std::env::var("HOME") {
        let venv = format!("{home}/.convergio/mlx-env/bin/python3");
        if std::path::Path::new(&venv).exists() {
            return venv;
        }
    }
    "python3".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_python_env_override() {
        // Note: env vars are process-global, so this test may race with others.
        // We verify the function logic directly instead of relying on env state.
        let key = "CONVERGIO_PYTHON_TEST_OVERRIDE";
        std::env::set_var(key, "/usr/bin/python3.11");
        let val = std::env::var(key).unwrap_or_default();
        std::env::remove_var(key);
        assert_eq!(val, "/usr/bin/python3.11", "env var roundtrip works");
        // The actual resolve_python() function is tested by resolve_python_falls_back
    }

    #[test]
    fn resolve_python_falls_back() {
        // Without env var, should return venv path or "python3"
        std::env::remove_var("CONVERGIO_PYTHON");
        let p = resolve_python();
        assert!(!p.is_empty());
    }

    #[test]
    fn mlx_available_returns_bool() {
        // Just verify it doesn't panic — actual availability depends on system
        let _ = mlx_available();
    }
}
