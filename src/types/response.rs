//! Response payloads.

use serde::{Deserialize, Serialize};

use super::{Message, Role, ToolCall};

/// Chat-completions response.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    /// Generation id. Optional because some providers omit it on
    /// streaming chunks that only carry tool-call deltas.
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub object: Option<String>,
    #[serde(default)]
    pub created: Option<u64>,
    pub model: String,
    pub choices: Vec<Choice>,
    #[serde(default)]
    pub usage: Option<Usage>,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub system_fingerprint: Option<String>,
}

/// One choice in a chat-completions response (also used for streaming chunks).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Choice {
    pub index: u32,
    #[serde(default)]
    pub message: Option<Message>,
    #[serde(default)]
    pub delta: Option<Delta>,
    #[serde(default)]
    pub finish_reason: Option<String>,
    #[serde(default)]
    pub native_finish_reason: Option<String>,
    #[serde(default)]
    pub logprobs: Option<LogProbs>,
}

/// Incremental token delta for streaming responses.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Delta {
    #[serde(default)]
    pub role: Option<Role>,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(default)]
    pub reasoning: Option<String>,
}

/// Token-level log probabilities. Shape varies by provider; kept opaque.
pub type LogProbs = serde_json::Value;

/// Legacy text-completions response.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CompletionResponse {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub object: Option<String>,
    #[serde(default)]
    pub created: Option<u64>,
    pub model: String,
    pub choices: Vec<CompletionChoice>,
    #[serde(default)]
    pub usage: Option<Usage>,
    #[serde(default)]
    pub provider: Option<String>,
}

/// One choice in a legacy completion response.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CompletionChoice {
    pub index: u32,
    pub text: String,
    #[serde(default)]
    pub finish_reason: Option<String>,
    #[serde(default)]
    pub native_finish_reason: Option<String>,
    #[serde(default)]
    pub logprobs: Option<LogProbs>,
}

/// Token usage accounting.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Usage {
    #[serde(default)]
    pub prompt_tokens: Option<u32>,
    #[serde(default)]
    pub completion_tokens: Option<u32>,
    #[serde(default)]
    pub total_tokens: Option<u32>,
    #[serde(default)]
    pub prompt_tokens_details: Option<TokenDetails>,
    #[serde(default)]
    pub completion_tokens_details: Option<TokenDetails>,
    #[serde(default)]
    pub cost: Option<f64>,
}

/// Sub-breakdown of token usage (cached, reasoning, etc.).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TokenDetails {
    #[serde(default)]
    pub cached_tokens: Option<u32>,
    #[serde(default)]
    pub reasoning_tokens: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn round_trip_chat_response() {
        let raw = r#"{
            "id":"gen-1",
            "object":"chat.completion",
            "created":1700000000,
            "model":"anthropic/claude-3-opus",
            "provider":"Anthropic",
            "choices":[{
                "index":0,
                "message":{"role":"assistant","content":"hi"},
                "finish_reason":"stop"
            }],
            "usage":{"prompt_tokens":3,"completion_tokens":1,"total_tokens":4}
        }"#;
        let r: ChatCompletionResponse = serde_json::from_str(raw).unwrap();
        assert_eq!(r.id.as_deref(), Some("gen-1"));
        assert_eq!(r.choices.len(), 1);
        let c = &r.choices[0];
        assert_eq!(c.message.as_ref().unwrap().content_text(), Some("hi"));
        assert_eq!(r.usage.as_ref().unwrap().total_tokens, Some(4));
    }

    #[test]
    fn round_trip_streaming_chunk() {
        let raw = r#"{
            "id":"gen-2",
            "model":"x/y",
            "choices":[{
                "index":0,
                "delta":{"role":"assistant","content":"Hel"},
                "finish_reason":null
            }]
        }"#;
        let r: ChatCompletionResponse = serde_json::from_str(raw).unwrap();
        let d = r.choices[0].delta.as_ref().unwrap();
        assert_eq!(d.content.as_deref(), Some("Hel"));
        assert_eq!(d.role, Some(Role::Assistant));
    }

    #[test]
    fn annotations_round_trip_url_citations() {
        let raw = r#"{
            "id":"gen-a","model":"x/y",
            "choices":[{
                "index":0,
                "message":{
                    "role":"assistant",
                    "content":"see source",
                    "annotations":[{
                        "type":"url_citation",
                        "url_citation":{
                            "url":"https://example.com",
                            "title":"Example",
                            "start_index":0,
                            "end_index":10
                        }
                    }]
                },
                "finish_reason":"stop"
            }]
        }"#;
        let r: ChatCompletionResponse = serde_json::from_str(raw).unwrap();
        let msg = r.choices[0].message.as_ref().unwrap();
        let anns = msg.annotations.as_ref().unwrap();
        assert_eq!(anns.len(), 1);
        match &anns[0] {
            crate::types::Annotation::UrlCitation { url_citation } => {
                assert_eq!(url_citation.url, "https://example.com");
                assert_eq!(url_citation.title.as_deref(), Some("Example"));
            }
            other => panic!("unexpected annotation: {other:?}"),
        }
    }

    #[test]
    fn reasoning_fields_round_trip() {
        let raw = r#"{
            "id":"gen-r","model":"x/y",
            "choices":[{
                "index":0,
                "message":{"role":"assistant","content":"42","reasoning":"long chain"},
                "finish_reason":"stop"
            }],
            "usage":{
                "prompt_tokens":3,"completion_tokens":5,"total_tokens":8,
                "completion_tokens_details":{"reasoning_tokens":17}
            }
        }"#;
        let r: ChatCompletionResponse = serde_json::from_str(raw).unwrap();
        let msg = r.choices[0].message.as_ref().unwrap();
        assert_eq!(msg.reasoning.as_deref(), Some("long chain"));
        let details = r
            .usage
            .as_ref()
            .unwrap()
            .completion_tokens_details
            .as_ref()
            .unwrap();
        assert_eq!(details.reasoning_tokens, Some(17));
    }

    #[test]
    fn round_trip_tool_call_response() {
        let raw = r#"{
            "id":"gen-3","model":"x/y",
            "choices":[{
                "index":0,
                "message":{
                    "role":"assistant",
                    "content":"",
                    "tool_calls":[{"id":"c1","type":"function","function":{"name":"f","arguments":"{}"}}]
                },
                "finish_reason":"tool_calls"
            }]
        }"#;
        let r: ChatCompletionResponse = serde_json::from_str(raw).unwrap();
        let calls = r.choices[0]
            .message
            .as_ref()
            .unwrap()
            .tool_calls
            .as_ref()
            .unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].function.name.as_deref(), Some("f"));
    }
}
