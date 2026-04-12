//! MCP tool definitions for the inference extension.

use convergio_types::extension::McpToolDef;
use serde_json::json;

pub fn inference_tools() -> Vec<McpToolDef> {
    vec![
        McpToolDef {
            name: "cvg_inference_complete".into(),
            description: "Run an inference completion.".into(),
            method: "POST".into(),
            path: "/api/inference/complete".into(),
            input_schema: json!({"type": "object", "properties": {"prompt": {"type": "string"}, "model": {"type": "string"}}, "required": ["prompt"]}),
            min_ring: "trusted".into(),
            path_params: vec![],
        },
        McpToolDef {
            name: "cvg_inference_costs".into(),
            description: "Get inference cost tracking data.".into(),
            method: "GET".into(),
            path: "/api/inference/costs".into(),
            input_schema: json!({"type": "object", "properties": {}}),
            min_ring: "community".into(),
            path_params: vec![],
        },
        McpToolDef {
            name: "cvg_inference_routing".into(),
            description: "Get inference routing decision info.".into(),
            method: "GET".into(),
            path: "/api/inference/routing-decision".into(),
            input_schema: json!({"type": "object", "properties": {}}),
            min_ring: "community".into(),
            path_params: vec![],
        },
    ]
}
