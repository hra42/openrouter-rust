# Multimodal inputs

Mix text with images, PDFs, audio, and text-file attachments using the
`create_user_message_with_*` helpers or build content piece by piece with
[`ContentBuilder`](crate::ContentBuilder).

## Image URL

```rust,no_run
use openrouter::{create_user_message_with_image, ChatCompletionRequest, Client, Message};

#[tokio::main]
async fn main() -> openrouter::Result<()> {
    let client = Client::builder()
        .api_key(std::env::var("OPENROUTER_API_KEY").unwrap())
        .build()?;
    let user = create_user_message_with_image(
        "Describe this image.",
        "https://example.com/cat.jpg",
    );
    let req = ChatCompletionRequest::new(
        "google/gemini-3.1-flash-lite",
        vec![user],
    );
    let _ = client.chat_complete(req).await?;
    Ok(())
}
```

## Mixed content with ContentBuilder

```rust,no_run
use openrouter::{ContentBuilder, ChatCompletionRequest, Client, Message, Role};

let content = ContentBuilder::new()
    .text("Compare these two screenshots:")
    .image("https://example.com/before.png")
    .image("https://example.com/after.png")
    .build();
let user = Message { role: Role::User, content, ..Default::default() };
# let _ = user;
```

## PDFs

Use `create_user_message_with_pdf` or
`create_user_message_with_base64_pdf` for local files. The
[`FileParserEngine`](crate::FileParserEngine) variants choose between
text and visual parsing strategies, and
[`Annotation`](crate::Annotation)s let you reuse parsed pages across
turns to avoid reprocessing costs.
