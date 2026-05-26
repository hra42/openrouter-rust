# Quickstart

```rust,no_run
use openrouter::{ChatCompletionRequest, Client, Message};

#[tokio::main]
async fn main() -> openrouter::Result<()> {
    let client = Client::builder()
        .api_key(std::env::var("OPENROUTER_API_KEY").unwrap())
        .app_name("my-app")
        .referer("https://my-app.example.com")
        .build()?;

    let req = ChatCompletionRequest::new(
        "google/gemini-3.1-flash-lite",
        vec![Message::user("Hello, who are you?")],
    );
    let resp = client.chat_complete(req).await?;
    if let Some(text) = resp.choices.first()
        .and_then(|c| c.message.as_ref())
        .and_then(|m| m.content_text())
    {
        println!("{text}");
    }
    Ok(())
}
```

## Client basics

- `Client` is cheap to clone (`Arc<ClientInner>`). Clone it freely across
  tasks; all clones share the same `reqwest::Client` and connection pool.
- The builder validates eagerly: missing `api_key` returns
  [`Error::MissingField`](crate::Error::MissingField); a malformed
  `base_url` returns [`Error::InvalidInput`](crate::Error::InvalidInput).
- Custom HTTP client: pass your own `reqwest::Client` to the builder; the
  builder's `timeout()` is ignored in that case.
- Retries (3 attempts by default) cover 429 and 5xx responses plus
  transport-level errors. Configure with `.retry(max, base_delay)`.
