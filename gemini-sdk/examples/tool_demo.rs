//! Demonstrates one round of function calling with the Gemini API.
//!
//! Defines a single tool `lookup(kind)` where `kind` is `"color"` or `"fruit"`.
//! Local implementation returns `"red"` for color and `"apple"` for fruit.
//!
//! Run with `GEMINI_API_KEY` set:
//!   cargo run -p gemini-sdk --example tool_demo
//! Optional override:
//!   GEMINI_MODEL=gemini-2.5-flash cargo run -p gemini-sdk --example tool_demo

use gemini_sdk::{
    Content, FunctionDeclaration, FunctionResponse, GeminiClient, GeminiClientTrait, GeminiModel,
    GenerateContentRequest, Part, Tool,
};
use serde_json::json;

fn run_lookup(kind: &str) -> serde_json::Value {
    match kind {
        "color" => json!({"result": "red"}),
        "fruit" => json!({"result": "apple"}),
        other => json!({"error": format!("unknown kind: {other}")}),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = std::env::var("GEMINI_API_KEY")
        .map_err(|_| anyhow::anyhow!("GEMINI_API_KEY must be set"))?;
    let model_name =
        std::env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-2.5-flash".to_string());
    let model = GeminiModel::Custom(model_name.clone());

    let client = GeminiClient::new(api_key);

    let tool = Tool {
        function_declarations: vec![FunctionDeclaration {
            name: "lookup".to_string(),
            description: "Look up a canonical example for a category. \
                Pass kind=\"color\" to get a color, or kind=\"fruit\" to get a fruit."
                .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "kind": {
                        "type": "string",
                        "enum": ["color", "fruit"],
                        "description": "Which kind of example to look up.",
                    }
                },
                "required": ["kind"],
            }),
        }],
    };

    let mut history = vec![Content {
        role: "user".to_string(),
        parts: vec![Part::Text {
            text: "Use the lookup tool to fetch the canonical color, then the canonical fruit. \
                   After both tool calls, reply with one sentence naming them."
                .to_string(),
        }],
    }];

    println!("model: {model_name}");
    println!("> sending initial prompt with `lookup` tool registered\n");

    for turn in 1..=6 {
        let request = GenerateContentRequest {
            contents: history.clone(),
            generation_config: None,
            safety_settings: None,
            system_instruction: None,
            tools: Some(vec![tool.clone()]),
            tool_config: None,
        };

        let response = client.generate_content(model.clone(), request).await?;
        let candidate = response
            .candidates
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("no candidates returned"))?;

        let mut tool_responses: Vec<Part> = Vec::new();
        let mut final_text: Option<String> = None;

        for part in &candidate.content.parts {
            match part {
                Part::Text { text } => {
                    final_text.get_or_insert_with(String::new).push_str(text);
                }
                Part::FunctionCall { function_call } => {
                    let kind = function_call
                        .args
                        .get("kind")
                        .and_then(|v| v.as_str())
                        .unwrap_or("<missing>");
                    let result = run_lookup(kind);
                    println!(
                        "[turn {turn}] model called {}({{kind: {:?}}}) -> {}",
                        function_call.name, kind, result
                    );
                    tool_responses.push(Part::FunctionResponse {
                        function_response: FunctionResponse {
                            name: function_call.name.clone(),
                            response: result,
                        },
                    });
                }
                Part::InlineData { .. } | Part::FunctionResponse { .. } => {}
            }
        }

        history.push(candidate.content.clone());

        if tool_responses.is_empty() {
            let text = final_text.unwrap_or_default();
            println!("\n[turn {turn}] final reply:\n{text}");
            return Ok(());
        }

        history.push(Content {
            role: "user".to_string(),
            parts: tool_responses,
        });
    }

    anyhow::bail!("gave up after 6 turns without a plain text answer")
}
