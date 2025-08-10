use super::{common, types};
use crate::impl_builder_methods;

use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum ToolChoiceType {
    None,
    Auto,
    Required,
    ToolChoice { tool: Tool },
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningEffort {
    Low,
    Medium,
    High,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum ReasoningMode {
    Effort { effort: ReasoningEffort },
    MaxTokens { max_tokens: i64 },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Reasoning {
    #[serde(flatten)]
    pub mode: Option<ReasoningMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatCompletionMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<HashMap<String, i32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "serialize_tool_choice")]
    pub tool_choice: Option<ToolChoiceType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<Reasoning>,
}

impl ChatCompletionRequest {
    pub fn new(model: String, messages: Vec<ChatCompletionMessage>) -> Self {
        Self {
            model,
            messages,
            temperature: None,
            top_p: None,
            stream: None,
            n: None,
            response_format: None,
            stop: None,
            max_tokens: None,
            presence_penalty: None,
            frequency_penalty: None,
            logit_bias: None,
            user: None,
            seed: None,
            tools: None,
            parallel_tool_calls: None,
            tool_choice: None,
            reasoning: None,
        }
    }
}

impl_builder_methods!(
    ChatCompletionRequest,
    temperature: f64,
    top_p: f64,
    n: i64,
    response_format: Value,
    stream: bool,
    stop: Vec<String>,
    max_tokens: i64,
    presence_penalty: f64,
    frequency_penalty: f64,
    logit_bias: HashMap<String, i32>,
    user: String,
    seed: i64,
    tools: Vec<Tool>,
    parallel_tool_calls: bool,
    tool_choice: ToolChoiceType,
    reasoning: Reasoning
);

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum MessageRole {
    user,
    system,
    assistant,
    function,
    tool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Content {
    Text(String),
    ImageUrl(Vec<ImageUrl>),
}

impl serde::Serialize for Content {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match *self {
            Content::Text(ref text) => {
                if text.is_empty() {
                    serializer.serialize_none()
                } else {
                    serializer.serialize_str(text)
                }
            }
            Content::ImageUrl(ref image_url) => image_url.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for Content {
    fn deserialize<D>(deserializer: D) -> Result<Content, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ContentVisitor;

        impl<'de> Visitor<'de> for ContentVisitor {
            type Value = Content;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid content type")
            }

            fn visit_str<E>(self, value: &str) -> Result<Content, E>
            where
                E: de::Error,
            {
                Ok(Content::Text(value.to_string()))
            }

            fn visit_seq<A>(self, seq: A) -> Result<Content, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let image_urls: Vec<ImageUrl> =
                    Deserialize::deserialize(de::value::SeqAccessDeserializer::new(seq))?;
                Ok(Content::ImageUrl(image_urls))
            }

            fn visit_map<M>(self, map: M) -> Result<Content, M::Error>
            where
                M: MapAccess<'de>,
            {
                let image_urls: Vec<ImageUrl> =
                    Deserialize::deserialize(de::value::MapAccessDeserializer::new(map))?;
                Ok(Content::ImageUrl(image_urls))
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Content::Text(String::new()))
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Content::Text(String::new()))
            }
        }

        deserializer.deserialize_any(ContentVisitor)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum ContentType {
    text,
    image_url,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub struct ImageUrlType {
    pub url: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub struct ImageUrl {
    pub r#type: ContentType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<ImageUrlType>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChatCompletionMessage {
    pub role: MessageRole,
    pub content: Content,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl ChatCompletionMessage {
	pub fn new(role: MessageRole, content: Content) -> Self {
		ChatCompletionMessage{
			role,
			content,
			name: None,
			tool_calls: None,
			tool_call_id: None
		}
	}

	pub fn new_text(role: MessageRole, content: String) -> Self {
		ChatCompletionMessage{
			role,
			content: Content::Text(content),
			name: None,
			tool_call_id: None,
			tool_calls: None
		}
	}
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChatCompletionMessageForResponse {
    pub role: MessageRole,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChatCompletionChoice {
    pub index: i64,
    pub message: ChatCompletionMessageForResponse,
    pub finish_reason: Option<FinishReason>,
    pub finish_details: Option<FinishDetails>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChatCompletionResponse {
    pub id: Option<String>,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<ChatCompletionChoice>,
    pub usage: common::Usage,
    pub system_fingerprint: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum FinishReason {
    stop,
    length,
    content_filter,
    tool_calls,
    null,
}

#[derive(Debug, Deserialize, Serialize)]
#[allow(non_camel_case_types)]
pub struct FinishDetails {
    pub r#type: FinishReason,
    pub stop: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ToolCall {
    pub id: String,
    pub r#type: String,
    pub function: ToolCallFunction,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ToolCallFunction {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
}

fn serialize_tool_choice<S>(
    value: &Option<ToolChoiceType>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        Some(ToolChoiceType::None) => serializer.serialize_str("none"),
        Some(ToolChoiceType::Auto) => serializer.serialize_str("auto"),
        Some(ToolChoiceType::Required) => serializer.serialize_str("required"),
        Some(ToolChoiceType::ToolChoice { tool }) => {
            let mut map = serializer.serialize_map(Some(2))?;
            map.serialize_entry("type", &tool.r#type)?;
            map.serialize_entry("function", &tool.function)?;
            map.end()
        }
        None => serializer.serialize_none(),
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct Tool {
    pub r#type: ToolType,
    pub function: types::Function,
}

#[derive(Debug, Deserialize, Serialize, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ToolType {
    Function,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_reasoning_effort_serialization() {
        let reasoning = Reasoning {
            mode: Some(ReasoningMode::Effort {
                effort: ReasoningEffort::High,
            }),
            exclude: Some(false),
            enabled: None,
        };

        let serialized = serde_json::to_value(&reasoning).unwrap();
        let expected = json!({
            "effort": "high",
            "exclude": false
        });

        assert_eq!(serialized, expected);
    }

    #[test]
    fn test_reasoning_max_tokens_serialization() {
        let reasoning = Reasoning {
            mode: Some(ReasoningMode::MaxTokens { max_tokens: 2000 }),
            exclude: None,
            enabled: Some(true),
        };

        let serialized = serde_json::to_value(&reasoning).unwrap();
        let expected = json!({
            "max_tokens": 2000,
            "enabled": true
        });

        assert_eq!(serialized, expected);
    }

    #[test]
    fn test_reasoning_deserialization() {
        let json_str = r#"{"effort": "medium", "exclude": true}"#;
        let reasoning: Reasoning = serde_json::from_str(json_str).unwrap();

        match reasoning.mode {
            Some(ReasoningMode::Effort { effort }) => {
                assert_eq!(effort, ReasoningEffort::Medium);
            }
            _ => panic!("Expected effort mode"),
        }
        assert_eq!(reasoning.exclude, Some(true));
    }

    #[test]
    fn test_chat_completion_request_with_reasoning() {
        let mut req = ChatCompletionRequest::new("gpt-4".to_string(), vec![]);

        req.reasoning = Some(Reasoning {
            mode: Some(ReasoningMode::Effort {
                effort: ReasoningEffort::Low,
            }),
            exclude: None,
            enabled: None,
        });

        let serialized = serde_json::to_value(&req).unwrap();
        assert_eq!(serialized["reasoning"]["effort"], "low");
    }
}
