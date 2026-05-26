//! Accumulate streaming tool-call deltas into complete [`ToolCall`]s.
//!
//! OpenRouter streams tool calls as a sequence of partial [`Delta::tool_calls`]
//! entries. Each entry carries an `index` that ties partial pieces of the same
//! call together. The `id` and function `name` typically arrive on the first
//! chunk for an index; subsequent chunks append to `function.arguments` (a
//! JSON-serialized string the caller can parse once complete).
//!
//! [`ToolCallAccumulator`] consumes [`Delta`]s in order and exposes the
//! finalized [`ToolCall`]s once the stream ends (or `finish_reason ==
//! "tool_calls"` is observed).

use std::collections::BTreeMap;

use crate::types::{Delta, FunctionCall, ToolCall};

/// Builder that merges streaming [`Delta::tool_calls`] fragments into
/// complete [`ToolCall`] values, keyed by their `index`.
#[derive(Debug, Default)]
pub struct ToolCallAccumulator {
    by_index: BTreeMap<u32, PartialCall>,
}

#[derive(Debug, Default)]
struct PartialCall {
    id: Option<String>,
    kind: Option<String>,
    name: Option<String>,
    arguments: String,
}

impl ToolCallAccumulator {
    /// Create an empty accumulator.
    pub fn new() -> Self {
        Self::default()
    }

    /// Merge a streaming [`Delta`]'s tool-call fragments into the accumulator.
    pub fn push_delta(&mut self, delta: &Delta) {
        let Some(calls) = delta.tool_calls.as_ref() else {
            return;
        };
        for call in calls {
            // Fall back to 0 when no index is present (single-tool case).
            let idx = call.index.unwrap_or(0);
            let entry = self.by_index.entry(idx).or_default();
            if entry.id.is_none() && !call.id.is_empty() {
                entry.id = Some(call.id.clone());
            }
            if entry.kind.is_none() && !call.kind.is_empty() {
                entry.kind = Some(call.kind.clone());
            }
            if let Some(name) = call.function.name.as_deref() {
                if entry.name.is_none() && !name.is_empty() {
                    entry.name = Some(name.to_string());
                }
            }
            if let Some(args) = call.function.arguments.as_deref() {
                entry.arguments.push_str(args);
            }
        }
    }

    /// Consume the accumulator and return the finalized tool calls in
    /// ascending `index` order.
    pub fn finish(self) -> Vec<ToolCall> {
        self.by_index
            .into_iter()
            .map(|(idx, p)| ToolCall {
                id: p.id.unwrap_or_default(),
                kind: p.kind.unwrap_or_else(|| "function".to_string()),
                function: FunctionCall {
                    name: p.name,
                    arguments: Some(p.arguments),
                },
                index: Some(idx),
            })
            .collect()
    }

    /// Number of partial calls tracked so far.
    pub fn len(&self) -> usize {
        self.by_index.len()
    }

    /// True when nothing has been accumulated yet.
    pub fn is_empty(&self) -> bool {
        self.by_index.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Delta, FunctionCall, ToolCall};
    use pretty_assertions::assert_eq;

    fn chunk(index: u32, id: &str, name: Option<&str>, args: &str) -> Delta {
        Delta {
            role: None,
            content: None,
            tool_calls: Some(vec![ToolCall {
                id: id.to_string(),
                kind: if id.is_empty() {
                    String::new()
                } else {
                    "function".to_string()
                },
                function: FunctionCall {
                    name: name.map(str::to_string),
                    arguments: Some(args.to_string()),
                },
                index: Some(index),
            }]),
            reasoning: None,
        }
    }

    #[test]
    fn accumulates_split_arguments() {
        let mut acc = ToolCallAccumulator::new();
        acc.push_delta(&chunk(0, "call_1", Some("get_weather"), "{\"loc"));
        acc.push_delta(&chunk(0, "", None, "ation\":\"SF\"}"));
        let calls = acc.finish();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].id, "call_1");
        assert_eq!(calls[0].function.name.as_deref(), Some("get_weather"));
        assert_eq!(
            calls[0].function.arguments.as_deref(),
            Some("{\"location\":\"SF\"}")
        );
    }

    #[test]
    fn separates_calls_by_index() {
        let mut acc = ToolCallAccumulator::new();
        acc.push_delta(&chunk(0, "a", Some("f1"), "{}"));
        acc.push_delta(&chunk(1, "b", Some("f2"), "{\"x\":1}"));
        let calls = acc.finish();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].id, "a");
        assert_eq!(calls[1].id, "b");
        assert_eq!(calls[1].function.name.as_deref(), Some("f2"));
    }

    #[test]
    fn ignores_deltas_without_tool_calls() {
        let mut acc = ToolCallAccumulator::new();
        let d = Delta {
            content: Some("hi".to_string()),
            ..Delta::default()
        };
        acc.push_delta(&d);
        assert!(acc.is_empty());
        assert!(acc.finish().is_empty());
    }
}
