//! Exercise the Phase 4 multimodal helpers against the live OpenRouter API.
//!
//! Sends a URL image, an inline base64 image (a tiny embedded PNG), a PDF
//! routed through the `file-parser` plugin, and a `ContentBuilder` message
//! mixing text + image.
//!
//! Run with:
//!
//! ```bash
//! OPENROUTER_API_KEY=sk-... cargo run --example multimodal
//! ```

use openrouter::{
    create_file_parser_plugin, create_user_message_with_base64_image_bytes,
    create_user_message_with_image, create_user_message_with_pdf, Annotation,
    ChatCompletionRequest, Client, ContentBuilder, FileParserEngine, ImageDetail, Role,
};

const MODEL: &str = "google/gemini-3.1-flash-lite";
const IMAGE_URL: &str = "https://hra42.com/test-image.png";
const PDF_URL: &str = "https://www.w3.org/WAI/ER/tests/xhtml/testfiles/resources/pdf/dummy.pdf";

// 1x1 transparent PNG.
const TINY_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
    0x89, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,
    0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE,
    0x42, 0x60, 0x82,
];

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key =
        std::env::var("OPENROUTER_API_KEY").map_err(|_| "OPENROUTER_API_KEY must be set")?;
    let client = Client::builder()
        .api_key(api_key)
        .app_name("openrouter-rust multimodal example")
        .build()?;

    // 1) Image via URL.
    println!("=== image (url) ===");
    let req = ChatCompletionRequest::new(
        MODEL,
        vec![create_user_message_with_image(
            "Describe this image in one sentence.",
            IMAGE_URL,
        )],
    );
    let resp = client.chat_complete(req).await?;
    println!(
        "{}",
        resp.choices
            .first()
            .and_then(|c| c.message.as_ref())
            .and_then(|m| m.content_text())
            .unwrap_or("")
    );

    // 2) Inline base64 image from in-memory bytes.
    println!("\n=== image (inline base64) ===");
    let req = ChatCompletionRequest::new(
        MODEL,
        vec![create_user_message_with_base64_image_bytes(
            "What can you tell about this image?",
            TINY_PNG,
            "image/png",
        )],
    );
    let resp = client.chat_complete(req).await?;
    println!(
        "{}",
        resp.choices
            .first()
            .and_then(|c| c.message.as_ref())
            .and_then(|m| m.content_text())
            .unwrap_or("")
    );

    // 3) ContentBuilder composing text + image with explicit detail.
    println!("\n=== ContentBuilder (text + image) ===");
    let msg = ContentBuilder::new()
        .add_text("In one short sentence, what is in this image?")
        .add_image_with_detail(IMAGE_URL, ImageDetail::Low)
        .build_message(Role::User);
    let resp = client
        .chat_complete(ChatCompletionRequest::new(MODEL, vec![msg]))
        .await?;
    println!(
        "{}",
        resp.choices
            .first()
            .and_then(|c| c.message.as_ref())
            .and_then(|m| m.content_text())
            .unwrap_or("")
    );

    // 4) PDF via URL with the file-parser plugin.
    println!("\n=== pdf (file-parser plugin) ===");
    let pdf_req = ChatCompletionRequest::new(
        MODEL,
        vec![create_user_message_with_pdf(
            "Summarize this PDF in one sentence.",
            PDF_URL,
            "dummy.pdf",
        )],
    )
    .with_plugins(vec![create_file_parser_plugin(FileParserEngine::PdfText)]);
    let pdf_resp = client.chat_complete(pdf_req).await?;
    let pdf_msg = pdf_resp
        .choices
        .first()
        .and_then(|c| c.message.as_ref())
        .ok_or("no pdf message")?;
    println!("{}", pdf_msg.content_text().unwrap_or(""));
    if let Some(anns) = &pdf_msg.annotations {
        for a in anns {
            if let Annotation::File { file } = a {
                println!("  (annotation reusable for: {})", file.filename);
            }
        }
    }

    Ok(())
}
