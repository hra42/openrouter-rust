# Provider routing

OpenRouter routes each request to a provider behind the model. The SDK
exposes the full routing surface as builder methods on
[`ChatCompletionRequest`](crate::ChatCompletionRequest):

| Method | Effect |
|---|---|
| `with_provider_order(["openai", "anthropic"])` | Try providers in this order |
| `with_provider_sort("throughput")` | Sort by `throughput`, `latency`, or `price` |
| `with_only_providers([...])` | Restrict to a set |
| `with_ignore_providers([...])` | Exclude providers |
| `with_quantizations(["fp16", "bf16"])` | Restrict by quantization |
| `with_max_price(json!({...}))` | Set per-token max price |
| `with_data_collection("deny")` | Allow/deny model-training collection |
| `with_require_parameters(true)` | Only providers supporting all parameters |
| `with_allow_fallbacks(false)` | Fail instead of cascading |
| `with_zdr(true)` | Require zero-data-retention endpoints |
| `with_nitro()` / `with_floor()` | Convenience shortcuts for sort by latency / price |

```rust,no_run
use openrouter::{ChatCompletionRequest, Client, Message};

let _req = ChatCompletionRequest::new(
    "google/gemini-3.1-flash-lite",
    vec![Message::user("Hi")],
)
.with_provider_sort("throughput")
.with_only_providers(["google", "openai"])
.with_require_parameters(true)
.with_allow_fallbacks(false);
```

For one-off custom routing, construct a [`Provider`](crate::Provider)
directly and pass it via
[`with_provider`](crate::ChatCompletionRequest::with_provider).
