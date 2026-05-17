use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GenerateContentRequest {
    pub contents: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_config: Option<GenerationConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_settings: Option<Vec<SafetySetting>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_instruction: Option<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "toolConfig")]
    pub tool_config: Option<ToolConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolConfig {
    #[serde(rename = "functionCallingConfig")]
    pub function_calling_config: FunctionCallingConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionCallingConfig {
    /// One of "AUTO", "ANY", "NONE".
    pub mode: String,
    #[serde(skip_serializing_if = "Option::is_none", rename = "allowedFunctionNames")]
    pub allowed_function_names: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Content {
    pub role: String, // "user" or "model"
    /// Defaulted so candidates with no `parts` (e.g. when a thinking model
    /// exhausts `maxOutputTokens` before producing visible output) round-trip
    /// instead of failing JSON decode.
    #[serde(default)]
    pub parts: Vec<Part>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum Part {
    Text {
        text: String,
    },
    InlineData {
        inline_data: Blob,
    },
    FunctionCall {
        #[serde(rename = "functionCall")]
        function_call: FunctionCall,
    },
    FunctionResponse {
        #[serde(rename = "functionResponse")]
        function_response: FunctionResponse,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tool {
    #[serde(rename = "functionDeclarations")]
    pub function_declarations: Vec<FunctionDeclaration>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionDeclaration {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub args: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionResponse {
    pub name: String,
    pub response: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Blob {
    pub mime_type: String,
    pub data: String, // base64 encoded
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "responseMimeType")]
    pub response_mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "responseSchema")]
    pub response_schema: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SafetySetting {
    pub category: SafetyCategory,
    pub threshold: SafetyThreshold,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SafetyCategory {
    #[serde(rename = "HARM_CATEGORY_HARASSMENT")]
    Harassment,
    #[serde(rename = "HARM_CATEGORY_HATE_SPEECH")]
    HateSpeech,
    #[serde(rename = "HARM_CATEGORY_SEXUALLY_EXPLICIT")]
    SexuallyExplicit,
    #[serde(rename = "HARM_CATEGORY_DANGEROUS_CONTENT")]
    DangerousContent,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SafetyThreshold {
    #[serde(rename = "BLOCK_LOW_AND_ABOVE")]
    BlockLowAndAbove,
    #[serde(rename = "BLOCK_MEDIUM_AND_ABOVE")]
    BlockMediumAndAbove,
    #[serde(rename = "BLOCK_ONLY_HIGH")]
    BlockOnlyHigh,
    #[serde(rename = "BLOCK_NONE")]
    BlockNone,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GenerateContentResponse {
    pub candidates: Vec<Candidate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage_metadata: Option<UsageMetadata>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Candidate {
    pub content: Content,
    pub finish_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_ratings: Option<Vec<SafetyRating>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SafetyRating {
    pub category: SafetyCategory,
    pub probability: String, // e.g., "NEGLIGIBLE"
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UsageMetadata {
    pub prompt_token_count: i32,
    pub candidates_token_count: i32,
    pub total_token_count: i32,
}

#[derive(Debug, Clone)]
pub enum GeminiModel {
    Gemini1_5Pro,
    Gemini1_5Flash,
    Gemini1_0Pro,
    Custom(String),
}

impl GeminiModel {
    pub fn as_str(&self) -> &str {
        match self {
            GeminiModel::Gemini1_5Pro => "gemini-1.5-pro",
            GeminiModel::Gemini1_5Flash => "gemini-1.5-flash",
            GeminiModel::Gemini1_0Pro => "gemini-pro",
            GeminiModel::Custom(value) => value.as_str(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn tool_serializes_with_function_declarations_key() {
        let tool = Tool {
            function_declarations: vec![FunctionDeclaration {
                name: "lookup".to_string(),
                description: "demo".to_string(),
                parameters: json!({"type": "object"}),
            }],
        };
        let v = serde_json::to_value(&tool).unwrap();
        assert!(v.get("functionDeclarations").is_some(), "got: {v}");
        assert!(v.get("function_declarations").is_none());
    }

    #[test]
    fn part_function_call_serializes_as_camel_case() {
        let part = Part::FunctionCall {
            function_call: FunctionCall {
                name: "lookup".to_string(),
                args: json!({"kind": "color"}),
            },
        };
        let v = serde_json::to_value(&part).unwrap();
        assert_eq!(v, json!({"functionCall": {"name": "lookup", "args": {"kind": "color"}}}));
    }

    #[test]
    fn part_function_response_serializes_as_camel_case() {
        let part = Part::FunctionResponse {
            function_response: FunctionResponse {
                name: "lookup".to_string(),
                response: json!({"result": "red"}),
            },
        };
        let v = serde_json::to_value(&part).unwrap();
        assert_eq!(v, json!({"functionResponse": {"name": "lookup", "response": {"result": "red"}}}));
    }

    #[test]
    fn part_function_call_roundtrips_from_api_shape() {
        let wire = json!({"functionCall": {"name": "lookup", "args": {"kind": "fruit"}}});
        let part: Part = serde_json::from_value(wire).unwrap();
        match part {
            Part::FunctionCall { function_call } => {
                assert_eq!(function_call.name, "lookup");
                assert_eq!(function_call.args, json!({"kind": "fruit"}));
            }
            other => panic!("expected FunctionCall variant, got {other:?}"),
        }
    }

    #[test]
    fn request_omits_tools_when_none() {
        let req = GenerateContentRequest {
            contents: vec![],
            generation_config: None,
            safety_settings: None,
            system_instruction: None,
            tools: None,
            tool_config: None,
        };
        let v = serde_json::to_value(&req).unwrap();
        assert!(v.get("tools").is_none(), "got: {v}");
        assert!(v.get("toolConfig").is_none());
    }

    #[test]
    fn generation_config_serializes_response_schema_camel_case() {
        let cfg = GenerationConfig {
            response_mime_type: Some("application/json".to_string()),
            response_schema: Some(json!({"type": "object"})),
            ..Default::default()
        };
        let v = serde_json::to_value(&cfg).unwrap();
        assert_eq!(v.get("responseMimeType").unwrap(), "application/json");
        assert_eq!(v.get("responseSchema").unwrap(), &json!({"type": "object"}));
        assert!(v.get("response_mime_type").is_none());
        assert!(v.get("temperature").is_none(), "None fields should be skipped");
    }

    #[test]
    fn content_decodes_when_parts_is_missing() {
        // Reproduces the Gemini-2.5-flash thinking-budget case: model returns
        // a content object with no `parts` because thinking consumed all
        // output tokens. The SDK must round-trip this as an empty Vec.
        let wire = json!({"role": "model"});
        let content: Content = serde_json::from_value(wire).unwrap();
        assert_eq!(content.role, "model");
        assert!(content.parts.is_empty());
    }

    #[test]
    fn tool_config_serializes_with_camel_case_keys() {
        let req = GenerateContentRequest {
            contents: vec![],
            generation_config: None,
            safety_settings: None,
            system_instruction: None,
            tools: None,
            tool_config: Some(ToolConfig {
                function_calling_config: FunctionCallingConfig {
                    mode: "ANY".to_string(),
                    allowed_function_names: Some(vec!["edit_text".to_string()]),
                },
            }),
        };
        let v = serde_json::to_value(&req).unwrap();
        let tc = v.get("toolConfig").expect("toolConfig present");
        let fcc = tc.get("functionCallingConfig").expect("functionCallingConfig present");
        assert_eq!(fcc.get("mode").unwrap(), "ANY");
        assert_eq!(fcc.get("allowedFunctionNames").unwrap(), &json!(["edit_text"]));
    }
}