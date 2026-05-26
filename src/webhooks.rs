//! Parser for OpenRouter Broadcast webhook payloads (OTLP JSON traces).
//!
//! Use [`parse_broadcast_payload`] for the raw OTLP envelope,
//! [`extract_broadcast_traces`] to flatten it into [`BroadcastTrace`] rows,
//! or [`parse_broadcast_traces`] to do both in one call. The convenience
//! function is what most webhook handlers want.
//!
//! Wiring this into an HTTP handler is intentionally left to the caller —
//! the parser is framework-agnostic. A typical axum handler looks like:
//!
//! ```ignore
//! async fn webhook(body: axum::body::Bytes) -> impl axum::response::IntoResponse {
//!     match openrouter::webhooks::parse_broadcast_traces(&body) {
//!         Ok(traces) => { /* process */ axum::http::StatusCode::OK }
//!         Err(_)     => axum::http::StatusCode::BAD_REQUEST,
//!     }
//! }
//! ```
//!
//! Shapes mirror the Go SDK (`broadcast.go`, `broadcast_models.go`).

use std::collections::BTreeMap;
use std::time::Duration;

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::error::{Error, Result};

/// Top-level OTLP JSON trace payload sent by Broadcast.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct OtlpExportTraceRequest {
    #[serde(rename = "resourceSpans", default)]
    pub resource_spans: Vec<OtlpResourceSpan>,
}

/// Spans grouped by their originating resource.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct OtlpResourceSpan {
    #[serde(default)]
    pub resource: OtlpResource,
    #[serde(rename = "scopeSpans", default)]
    pub scope_spans: Vec<OtlpScopeSpan>,
}

/// The entity producing telemetry.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct OtlpResource {
    #[serde(default)]
    pub attributes: Vec<OtlpAttribute>,
}

/// Spans grouped by instrumentation scope.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct OtlpScopeSpan {
    #[serde(default)]
    pub scope: Option<OtlpScope>,
    #[serde(default)]
    pub spans: Vec<OtlpSpan>,
}

/// Identifies the instrumentation library.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct OtlpScope {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub version: String,
}

/// A single span within a trace.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct OtlpSpan {
    #[serde(rename = "traceId", default)]
    pub trace_id: String,
    #[serde(rename = "spanId", default)]
    pub span_id: String,
    #[serde(rename = "parentSpanId", default)]
    pub parent_span_id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub kind: i32,
    #[serde(rename = "startTimeUnixNano", default)]
    pub start_time_unix_nano: String,
    #[serde(rename = "endTimeUnixNano", default)]
    pub end_time_unix_nano: String,
    #[serde(default)]
    pub attributes: Vec<OtlpAttribute>,
    #[serde(default)]
    pub status: Option<OtlpStatus>,
    #[serde(default)]
    pub events: Vec<OtlpEvent>,
}

/// Key-value pair attached to a span or resource.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct OtlpAttribute {
    #[serde(default)]
    pub key: String,
    #[serde(default)]
    pub value: OtlpAnyValue,
}

/// A polymorphic OTLP value. The OTLP spec encodes int64 values as strings,
/// but some emitters send them as JSON numbers — [`flex_int`] tolerates
/// both.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct OtlpAnyValue {
    #[serde(
        rename = "stringValue",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub string_value: Option<String>,
    #[serde(
        rename = "intValue",
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_flex_int_opt"
    )]
    pub int_value: Option<String>,
    #[serde(
        rename = "doubleValue",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub double_value: Option<f64>,
    #[serde(rename = "boolValue", default, skip_serializing_if = "Option::is_none")]
    pub bool_value: Option<bool>,
    #[serde(
        rename = "arrayValue",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub array_value: Option<OtlpArrayValue>,
}

fn deserialize_flex_int_opt<'de, D>(de: D) -> std::result::Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let v = Option::<Value>::deserialize(de)?;
    match v {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => Ok(Some(s)),
        Some(Value::Number(n)) => Ok(Some(n.to_string())),
        Some(other) => Err(serde::de::Error::custom(format!(
            "intValue: expected string or number, got {other}"
        ))),
    }
}

impl OtlpAnyValue {
    /// Render the value as a string, regardless of its underlying type.
    /// Returns the empty string for [`Self::array_value`] or an
    /// all-`None` value.
    pub fn string_val(&self) -> String {
        if let Some(s) = &self.string_value {
            return s.clone();
        }
        if let Some(i) = &self.int_value {
            return i.clone();
        }
        if let Some(d) = self.double_value {
            // Match Go's `%g` formatting reasonably well.
            return format!("{d}");
        }
        if let Some(b) = self.bool_value {
            return b.to_string();
        }
        String::new()
    }
}

/// Wraps a slice of OTLP values.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct OtlpArrayValue {
    #[serde(default)]
    pub values: Vec<OtlpAnyValue>,
}

/// The status of a span.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct OtlpStatus {
    #[serde(default)]
    pub code: i32,
    #[serde(default)]
    pub message: String,
}

/// A timed event within a span.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct OtlpEvent {
    #[serde(default)]
    pub name: String,
    #[serde(rename = "timeUnixNano", default)]
    pub time_unix_nano: String,
    #[serde(default)]
    pub attributes: Vec<OtlpAttribute>,
}

/// User-friendly representation of a single span extracted from an OTLP
/// trace payload sent by OpenRouter Broadcast. Field names mirror the Go
/// SDK; deprecated aliases ([`Self::prompt_tokens`], etc.) are kept so
/// callers porting from the Go SDK don't have to rename everything.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BroadcastTrace {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: String,
    pub span_name: String,

    /// Start time as nanoseconds since the Unix epoch (0 when missing).
    pub start_time_unix_nano: i64,
    /// End time as nanoseconds since the Unix epoch (0 when missing).
    pub end_time_unix_nano: i64,
    /// `end - start` when both timestamps are present.
    pub duration: Duration,

    // Deprecated aliases (still populated for backward compatibility).
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub total_tokens: i64,
    pub cost: f64,
    pub model: String,

    // New canonical token usage fields.
    pub input_tokens: i64,
    pub output_tokens: i64,

    // Cost breakdown.
    pub total_cost: f64,
    pub input_cost: f64,
    pub output_cost: f64,

    // Token detail.
    pub cached_tokens: i64,
    pub audio_input_tokens: i64,
    pub video_input_tokens: i64,
    pub image_output_tokens: i64,
    pub reasoning_tokens: i64,

    // GenAI semantic-convention fields.
    pub operation_name: String,
    pub system: String,
    pub provider_name: String,
    pub response_model: String,
    pub finish_reason: String,
    pub finish_reasons: String,
    pub request_model: String,

    // OpenRouter-specific fields.
    pub provider_slug: String,
    pub openrouter_provider_name: String,
    pub api_key_name: String,
    pub entity_id: String,
    pub openrouter_user_id: String,
    pub openrouter_finish_reason: String,
    pub input_unit_price: f64,
    pub output_unit_price: f64,
    pub source: String,

    // Content fields.
    pub prompt: String,
    pub completion: String,

    // Span-level fields.
    pub span_type: String,
    pub span_level: String,
    pub span_input: String,
    pub span_output: String,

    // Trace-level fields.
    pub trace_name: String,
    pub trace_input: String,
    pub trace_output: String,
    pub trace_tags: String,

    pub user_id: String,
    pub session_id: String,

    /// Values from `trace.metadata.*` attributes (prefix stripped).
    pub metadata: BTreeMap<String, String>,
    /// Values from `span.metadata.*` attributes (prefix stripped).
    pub span_metadata: BTreeMap<String, String>,
    /// Attributes from the OTLP resource.
    pub resource_attributes: BTreeMap<String, String>,
    /// All other span attributes not mapped to a named field.
    pub raw_attributes: BTreeMap<String, String>,
}

/// Parse raw JSON bytes into the OTLP trace envelope.
pub fn parse_broadcast_payload(data: &[u8]) -> Result<OtlpExportTraceRequest> {
    serde_json::from_slice(data).map_err(Error::Decode)
}

/// Flatten an OTLP envelope into a vector of [`BroadcastTrace`] rows.
/// Missing attributes produce zero values; extraction is best-effort.
pub fn extract_broadcast_traces(payload: &OtlpExportTraceRequest) -> Vec<BroadcastTrace> {
    let mut out = Vec::new();
    for rs in &payload.resource_spans {
        let res_attrs = extract_attribute_map(&rs.resource.attributes);
        for ss in &rs.scope_spans {
            for span in &ss.spans {
                out.push(build_trace(span, &res_attrs));
            }
        }
    }
    out
}

/// Convenience: parse + flatten in one call.
pub fn parse_broadcast_traces(data: &[u8]) -> Result<Vec<BroadcastTrace>> {
    Ok(extract_broadcast_traces(&parse_broadcast_payload(data)?))
}

fn build_trace(span: &OtlpSpan, res_attrs: &BTreeMap<String, String>) -> BroadcastTrace {
    let mut t = BroadcastTrace {
        trace_id: span.trace_id.clone(),
        span_id: span.span_id.clone(),
        parent_span_id: span.parent_span_id.clone(),
        span_name: span.name.clone(),
        resource_attributes: res_attrs.clone(),
        ..Default::default()
    };
    let start = span.start_time_unix_nano.parse::<i64>().unwrap_or(0);
    let end = span.end_time_unix_nano.parse::<i64>().unwrap_or(0);
    t.start_time_unix_nano = start;
    t.end_time_unix_nano = end;
    if start > 0 && end > start {
        let delta = (end - start) as u64;
        t.duration = Duration::from_nanos(delta);
    }
    for attr in &span.attributes {
        let val = attr.value.string_val();
        apply_attribute(&mut t, &attr.key, val);
    }
    if t.total_tokens == 0 && (t.input_tokens > 0 || t.output_tokens > 0) {
        t.total_tokens = t.input_tokens + t.output_tokens;
    }
    t
}

fn apply_attribute(t: &mut BroadcastTrace, key: &str, val: String) {
    let v = val.as_str();
    let parse_i = |s: &str| s.parse::<i64>().unwrap_or(0);
    let parse_f = |s: &str| s.parse::<f64>().unwrap_or(0.0);
    match key {
        // Model fields
        "gen_ai.response.model" => {
            t.response_model = val.clone();
            t.model = val;
        }
        "gen_ai.request.model" => {
            t.request_model = val.clone();
            if t.model.is_empty() {
                t.model = val;
            }
        }
        // Token usage (new canonical keys)
        "gen_ai.usage.input_tokens" => {
            t.input_tokens = parse_i(v);
            t.prompt_tokens = t.input_tokens;
        }
        "gen_ai.usage.output_tokens" => {
            t.output_tokens = parse_i(v);
            t.completion_tokens = t.output_tokens;
        }
        // Token usage (old keys, backward compat)
        "gen_ai.usage.prompt_tokens" => {
            let n = parse_i(v);
            t.prompt_tokens = n;
            if t.input_tokens == 0 {
                t.input_tokens = n;
            }
        }
        "gen_ai.usage.completion_tokens" => {
            let n = parse_i(v);
            t.completion_tokens = n;
            if t.output_tokens == 0 {
                t.output_tokens = n;
            }
        }
        "gen_ai.usage.total_tokens" => t.total_tokens = parse_i(v),
        // Cost fields
        "gen_ai.usage.total_cost" => {
            t.total_cost = parse_f(v);
            t.cost = t.total_cost;
        }
        "gen_ai.usage.cost" => {
            let f = parse_f(v);
            t.cost = f;
            if t.total_cost == 0.0 {
                t.total_cost = f;
            }
        }
        "gen_ai.usage.input_cost" => t.input_cost = parse_f(v),
        "gen_ai.usage.output_cost" => t.output_cost = parse_f(v),
        // Token detail
        "gen_ai.usage.input_tokens.cached" => t.cached_tokens = parse_i(v),
        "gen_ai.usage.input_tokens.audio" => t.audio_input_tokens = parse_i(v),
        "gen_ai.usage.input_tokens.video" => t.video_input_tokens = parse_i(v),
        "gen_ai.usage.output_tokens.image" => t.image_output_tokens = parse_i(v),
        "gen_ai.usage.output_tokens.reasoning" => t.reasoning_tokens = parse_i(v),
        // GenAI semantic convention
        "gen_ai.operation.name" => t.operation_name = val,
        "gen_ai.system" => t.system = val,
        "gen_ai.provider.name" => t.provider_name = val,
        "gen_ai.response.finish_reason" => t.finish_reason = val,
        "gen_ai.response.finish_reasons" => t.finish_reasons = val,
        // OpenRouter-specific
        "openrouter.provider_slug" => t.provider_slug = val,
        "openrouter.provider_name" => t.openrouter_provider_name = val,
        "openrouter.api_key_name" => t.api_key_name = val,
        "openrouter.entity_id" => t.entity_id = val,
        "openrouter.user_id" => t.openrouter_user_id = val,
        "openrouter.finish_reason" => t.openrouter_finish_reason = val,
        "openrouter.input_unit_price" => t.input_unit_price = parse_f(v),
        "openrouter.output_unit_price" => t.output_unit_price = parse_f(v),
        "openrouter.source" => t.source = val,
        // Content
        "gen_ai.prompt" => t.prompt = val,
        "gen_ai.completion" => t.completion = val,
        // Span-level
        "span.type" => t.span_type = val,
        "span.level" => t.span_level = val,
        "span.input" => t.span_input = val,
        "span.output" => t.span_output = val,
        // Trace-level
        "trace.name" => t.trace_name = val,
        "trace.input" => t.trace_input = val,
        "trace.output" => t.trace_output = val,
        "trace.tags" => t.trace_tags = val,
        // Identity
        "user.id" => t.user_id = val,
        "session.id" => t.session_id = val,
        // Prefixed metadata / unknown attributes
        other => {
            if let Some(rest) = other.strip_prefix("trace.metadata.") {
                t.metadata.insert(rest.to_string(), val);
            } else if let Some(rest) = other.strip_prefix("span.metadata.") {
                t.span_metadata.insert(rest.to_string(), val);
            } else {
                t.raw_attributes.insert(other.to_string(), val);
            }
        }
    }
}

fn extract_attribute_map(attrs: &[OtlpAttribute]) -> BTreeMap<String, String> {
    attrs
        .iter()
        .map(|a| (a.key.clone(), a.value.string_val()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flex_int_accepts_string_or_number() {
        let from_string: OtlpAnyValue = serde_json::from_str(r#"{"intValue":"42"}"#).unwrap();
        assert_eq!(from_string.int_value.as_deref(), Some("42"));

        let from_number: OtlpAnyValue = serde_json::from_str(r#"{"intValue":42}"#).unwrap();
        assert_eq!(from_number.int_value.as_deref(), Some("42"));
    }

    #[test]
    fn parse_minimal_payload_extracts_one_trace() {
        let payload = br#"{
            "resourceSpans": [{
                "resource": {"attributes": [{"key":"service.name","value":{"stringValue":"or"}}]},
                "scopeSpans": [{
                    "scope": {"name":"or-gateway","version":"1.0"},
                    "spans": [{
                        "traceId":"abc","spanId":"s1","name":"gen_ai.chat",
                        "kind":2,
                        "startTimeUnixNano":"1700000000000000000",
                        "endTimeUnixNano":"1700000000500000000",
                        "attributes": [
                            {"key":"gen_ai.response.model","value":{"stringValue":"openai/gpt-5"}},
                            {"key":"gen_ai.usage.input_tokens","value":{"intValue":"120"}},
                            {"key":"gen_ai.usage.output_tokens","value":{"intValue":"30"}},
                            {"key":"gen_ai.usage.total_cost","value":{"stringValue":"0.0042"}},
                            {"key":"openrouter.provider_slug","value":{"stringValue":"openai"}},
                            {"key":"trace.metadata.tenant","value":{"stringValue":"acme"}},
                            {"key":"span.metadata.region","value":{"stringValue":"us-west"}},
                            {"key":"some.custom.attr","value":{"stringValue":"x"}}
                        ]
                    }]
                }]
            }]
        }"#;
        let traces = parse_broadcast_traces(payload).unwrap();
        assert_eq!(traces.len(), 1);
        let t = &traces[0];
        assert_eq!(t.trace_id, "abc");
        assert_eq!(t.span_id, "s1");
        assert_eq!(t.model, "openai/gpt-5");
        assert_eq!(t.response_model, "openai/gpt-5");
        assert_eq!(t.input_tokens, 120);
        assert_eq!(t.prompt_tokens, 120); // backward-compat alias populated
        assert_eq!(t.output_tokens, 30);
        assert_eq!(t.total_tokens, 150); // computed when absent
        assert!((t.total_cost - 0.0042).abs() < 1e-9);
        assert!((t.cost - 0.0042).abs() < 1e-9);
        assert_eq!(t.provider_slug, "openai");
        assert_eq!(t.metadata.get("tenant").map(String::as_str), Some("acme"));
        assert_eq!(
            t.span_metadata.get("region").map(String::as_str),
            Some("us-west")
        );
        assert_eq!(
            t.raw_attributes.get("some.custom.attr").map(String::as_str),
            Some("x")
        );
        assert_eq!(
            t.resource_attributes
                .get("service.name")
                .map(String::as_str),
            Some("or")
        );
        assert_eq!(t.duration, Duration::from_millis(500));
    }

    #[test]
    fn old_token_keys_are_back_filled() {
        let payload = br#"{
            "resourceSpans":[{"resource":{"attributes":[]},"scopeSpans":[{"spans":[{
                "traceId":"a","spanId":"b","name":"x",
                "kind":1,"startTimeUnixNano":"0","endTimeUnixNano":"0",
                "attributes":[
                    {"key":"gen_ai.usage.prompt_tokens","value":{"intValue":"10"}},
                    {"key":"gen_ai.usage.completion_tokens","value":{"intValue":"5"}}
                ]
            }]}]}]
        }"#;
        let traces = parse_broadcast_traces(payload).unwrap();
        let t = &traces[0];
        assert_eq!(t.prompt_tokens, 10);
        assert_eq!(t.input_tokens, 10); // back-filled
        assert_eq!(t.completion_tokens, 5);
        assert_eq!(t.output_tokens, 5);
        assert_eq!(t.total_tokens, 15);
    }

    #[test]
    fn invalid_json_errors() {
        let err = parse_broadcast_traces(b"not json").unwrap_err();
        assert!(matches!(err, Error::Decode(_)));
    }
}
