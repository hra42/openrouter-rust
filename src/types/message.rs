//! Chat-style messages and multimodal content parts.

use serde::{Deserialize, Serialize};

use super::{Annotation, ToolCall};

/// Role of a chat message author.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

/// A chat message.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: Content,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub tool_call_id: Option<String>,
    /// Reasoning trace returned by the model (non-streaming responses).
    /// Streaming reasoning chunks come through [`crate::types::Delta::reasoning`].
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub reasoning: Option<String>,
    /// Typed annotations attached by plugins (e.g. web-search citations).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub annotations: Option<Vec<Annotation>>,
}

/// Message content: either a plain string or an array of typed parts.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Content {
    Text(String),
    Parts(Vec<ContentPart>),
}

impl Content {
    /// Borrowed plain-text view when the content is a single string.
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Content::Text(s) => Some(s),
            Content::Parts(_) => None,
        }
    }
}

impl From<String> for Content {
    fn from(s: String) -> Self {
        Content::Text(s)
    }
}

impl From<&str> for Content {
    fn from(s: &str) -> Self {
        Content::Text(s.to_string())
    }
}

/// One element of a multimodal content array.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentPart {
    Text { text: String },
    ImageUrl { image_url: ImageUrl },
    File { file: FileRef },
    InputAudio { input_audio: InputAudio },
}

/// Image URL (or data URL) reference.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ImageUrl {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub detail: Option<String>,
}

/// File reference (URL or inline base64). Multimodal Phase 4 expands this.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FileRef {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub file_data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub file_url: Option<String>,
}

/// Inline audio input.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InputAudio {
    pub data: String,
    pub format: String,
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self::new(Role::System, content)
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self::new(Role::User, content)
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self::new(Role::Assistant, content)
    }

    pub fn tool(content: impl Into<String>, tool_call_id: impl Into<String>) -> Self {
        Self {
            role: Role::Tool,
            content: Content::Text(content.into()),
            name: None,
            tool_calls: None,
            tool_call_id: Some(tool_call_id.into()),
            reasoning: None,
            annotations: None,
        }
    }

    fn new(role: Role, content: impl Into<String>) -> Self {
        Self {
            role,
            content: Content::Text(content.into()),
            name: None,
            tool_calls: None,
            tool_call_id: None,
            reasoning: None,
            annotations: None,
        }
    }

    /// Plain-text view of this message's content, when available.
    pub fn content_text(&self) -> Option<&str> {
        self.content.as_text()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn string_content_round_trip() {
        let m = Message::user("hello");
        let v = serde_json::to_value(&m).unwrap();
        assert_eq!(v, json!({"role":"user","content":"hello"}));
        let back: Message = serde_json::from_value(v).unwrap();
        assert_eq!(back, m);
    }

    #[test]
    fn parts_content_deserializes() {
        let v = json!({
            "role": "user",
            "content": [
                {"type": "text", "text": "look at this"},
                {"type": "image_url", "image_url": {"url": "https://x/y.png"}}
            ]
        });
        let m: Message = serde_json::from_value(v).unwrap();
        match &m.content {
            Content::Parts(p) => assert_eq!(p.len(), 2),
            _ => panic!("expected parts"),
        }
    }

    #[test]
    fn assistant_with_tool_calls() {
        let v = json!({
            "role": "assistant",
            "content": "",
            "tool_calls": [
                {"id":"c1","type":"function","function":{"name":"f","arguments":"{}"}}
            ]
        });
        let m: Message = serde_json::from_value(v).unwrap();
        assert_eq!(m.role, Role::Assistant);
        assert_eq!(m.tool_calls.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn optional_fields_skipped_when_none() {
        let m = Message::system("hi");
        let s = serde_json::to_string(&m).unwrap();
        assert!(!s.contains("name"));
        assert!(!s.contains("tool_calls"));
        assert!(!s.contains("tool_call_id"));
    }
}
