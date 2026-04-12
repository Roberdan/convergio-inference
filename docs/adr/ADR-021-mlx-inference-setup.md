---
version: "1.0"
last_updated: "2026-04-07"
author: "convergio-team"
tags: ["adr"]
---

# ADR-021: MLX Inference + Telegram Setup

**Status**: Accepted
**Date**: 2026-04-05
**Supersedes**: Extends ADR-015 (MLX model selection) with operational setup details.

## Decision

Use **Qwen2.5-Coder-7B-Instruct-4bit** via **MLX direct subprocess** on Apple Silicon nodes for local inference. Integrate with Telegram bot for Kernel/Jarvis PM assistant.

## Context

Session 8 benchmarks on M1 Pro 32GB showed Qwen 7B Coder is the best trade-off for local inference:

| Metric | Value |
|--------|-------|
| Classification latency | 0.58s |
| Code generation throughput | 32 tok/s |
| RAM usage | ~4.5 GB |
| Cost | $0.00 |

Ollama adds process overhead with no benefit on Apple Silicon. MLX subprocess is faster and lighter.

## Configuration

### 1. MLX Python environment

```bash
# Create venv (one-time)
python3 -m venv ~/.convergio/mlx-env
~/.convergio/mlx-env/bin/pip install mlx-lm
```

Set in `~/.convergio/env`:

```bash
CONVERGIO_PYTHON=~/.convergio/mlx-env/bin/python3
CONVERGIO_MLX_MODEL=mlx-community/Qwen2.5-Coder-7B-Instruct-4bit
```

### 2. Inference router config

In `daemon/config/inference-models.toml`, the MLX entry:

```toml
[[models]]
name = "qwen2.5-coder-7b-4bit"
provider = "mlx"
model_id = "mlx-community/Qwen2.5-Coder-7B-Instruct-4bit"
cost_per_1k_input = 0.0
cost_per_1k_output = 0.0
tier_min = "t1"
tier_max = "t2"
turbo_quant = false
```

### 3. Node role

In `~/.convergio/config.toml`, the node must have role `"kernel"` or `"all"`:

```toml
[node]
role = "kernel"
```

MLX extensions only load on kernel-role nodes (see ADR-015, PR #134).

### 4. Telegram integration

Set in `~/.convergio/env`:

```bash
CONVERGIO_TELEGRAM_BOT_TOKEN=<from @BotFather>
CONVERGIO_TELEGRAM_CHAT_ID=<your chat ID>
```

Only ONE daemon instance should have the kernel role to avoid duplicate bot responses.

## Known Issues

| Issue | Cause | Fix |
|-------|-------|-----|
| Router picks Ollama over MLX | llama3 entry has lower tier and is checked first | Remove/comment llama3 entry on nodes without Ollama |
| TurboQuant garbage output | KV cache quantization incompatible with weight-quantized models | Keep `turbo_quant = false` (see ADR-015) |
| `mlx_available()` returns false | `CONVERGIO_PYTHON` not set or venv missing `mlx-lm` | Verify: `$CONVERGIO_PYTHON -c "import mlx_lm"` |

## Consequences

- Zero-cost local inference for classification and simple code tasks
- Telegram bot responds via Kernel/Jarvis on kernel-role nodes
- Cloud models (Claude, GPT-4o) reserved for t2+ tasks via tier routing
- Model cached in `~/.cache/huggingface/hub/` (~4 GB)
