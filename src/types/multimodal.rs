//! Multimodal input helpers: images, PDFs, audio, text files, and a
//! [`ContentBuilder`] for composing mixed-content messages.
//!
//! These wrap the wire types in [`message`](super::message) into ergonomic
//! constructors that mirror the Go SDK's `CreateUserMessageWith*` API.

use std::path::Path;

use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use serde::{Deserialize, Serialize};

use super::message::{Content, ContentPart, FileRef, ImageUrl, InputAudio, Message, Role};
use super::{FilePdfConfig, FilePluginConfig, Plugin};
use crate::error::{Error, Result};

// ---------------------------------------------------------------------------
// Images
// ---------------------------------------------------------------------------

/// Image-detail hint forwarded to the model (`low`, `high`, `auto`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageDetail {
    Low,
    High,
    Auto,
}

impl ImageDetail {
    fn as_str(self) -> &'static str {
        match self {
            ImageDetail::Low => "low",
            ImageDetail::High => "high",
            ImageDetail::Auto => "auto",
        }
    }
}

/// Build a user message with a single image URL.
pub fn create_user_message_with_image(text: impl Into<String>, url: impl Into<String>) -> Message {
    user_message_with_parts(
        text,
        vec![ContentPart::ImageUrl {
            image_url: ImageUrl {
                url: url.into(),
                detail: None,
            },
        }],
    )
}

/// Build a user message with several image URLs.
pub fn create_user_message_with_images<I, S>(text: impl Into<String>, urls: I) -> Message
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let extras = urls
        .into_iter()
        .map(|u| ContentPart::ImageUrl {
            image_url: ImageUrl {
                url: u.into(),
                detail: None,
            },
        })
        .collect();
    user_message_with_parts(text, extras)
}

/// Build a user message with a single image URL plus an explicit detail level.
pub fn create_user_message_with_image_detail(
    text: impl Into<String>,
    url: impl Into<String>,
    detail: ImageDetail,
) -> Message {
    user_message_with_parts(
        text,
        vec![ContentPart::ImageUrl {
            image_url: ImageUrl {
                url: url.into(),
                detail: Some(detail.as_str().to_string()),
            },
        }],
    )
}

/// Read an image from disk, base64-encode it, and attach it to a user message.
/// MIME is inferred from the file extension (png/jpg/jpeg/webp/gif).
pub fn create_user_message_with_base64_image(
    text: impl Into<String>,
    path: impl AsRef<Path>,
) -> Result<Message> {
    let data_url = encode_image_to_base64(path)?;
    Ok(user_message_with_parts(
        text,
        vec![ContentPart::ImageUrl {
            image_url: ImageUrl {
                url: data_url,
                detail: None,
            },
        }],
    ))
}

/// Attach an in-memory image to a user message, encoding it as base64 with
/// the given MIME type (e.g. `"image/png"`).
pub fn create_user_message_with_base64_image_bytes(
    text: impl Into<String>,
    bytes: &[u8],
    mime: &str,
) -> Message {
    let data_url = encode_image_bytes_to_base64(bytes, mime);
    user_message_with_parts(
        text,
        vec![ContentPart::ImageUrl {
            image_url: ImageUrl {
                url: data_url,
                detail: None,
            },
        }],
    )
}

/// Read an image from disk and return a `data:<mime>;base64,...` URL.
pub fn encode_image_to_base64(path: impl AsRef<Path>) -> Result<String> {
    let path = path.as_ref();
    let mime = image_mime_from_path(path)?;
    let bytes =
        std::fs::read(path).map_err(|_| Error::InvalidInput("failed to read image file"))?;
    Ok(encode_image_bytes_to_base64(&bytes, mime))
}

/// Encode raw image bytes into a `data:<mime>;base64,...` URL.
pub fn encode_image_bytes_to_base64(bytes: &[u8], mime: &str) -> String {
    format!("data:{};base64,{}", mime, BASE64_STANDARD.encode(bytes))
}

fn image_mime_from_path(path: &Path) -> Result<&'static str> {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_ascii_lowercase());
    match ext.as_deref() {
        Some("png") => Ok("image/png"),
        Some("jpg" | "jpeg") => Ok("image/jpeg"),
        Some("webp") => Ok("image/webp"),
        Some("gif") => Ok("image/gif"),
        _ => Err(Error::InvalidInput("unsupported image format")),
    }
}

// ---------------------------------------------------------------------------
// PDFs / files
// ---------------------------------------------------------------------------

/// PDF parsing engine selection for the `file-parser` plugin.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FileParserEngine {
    /// Lightweight text extraction (`pdf-text`).
    PdfText,
    /// OCR-based extraction (`mistral-ocr`).
    MistralOcr,
    /// Native model handling (`native`).
    Native,
    /// Let OpenRouter pick a default (no `pdf.engine` emitted).
    Auto,
}

impl FileParserEngine {
    fn as_str(self) -> Option<&'static str> {
        match self {
            FileParserEngine::PdfText => Some("pdf-text"),
            FileParserEngine::MistralOcr => Some("mistral-ocr"),
            FileParserEngine::Native => Some("native"),
            FileParserEngine::Auto => None,
        }
    }
}

/// A single file attached to a message (PDF or other parser-supported type).
/// Provide either `file_url` (remote) or `file_data` (already-encoded data URL).
#[derive(Clone, Debug, PartialEq)]
pub struct File {
    pub filename: String,
    pub file_url: Option<String>,
    pub file_data: Option<String>,
}

impl File {
    /// File served from a public URL.
    pub fn from_url(filename: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            filename: filename.into(),
            file_url: Some(url.into()),
            file_data: None,
        }
    }

    /// File supplied as a pre-encoded `data:...;base64,...` URL.
    pub fn from_data_url(filename: impl Into<String>, data_url: impl Into<String>) -> Self {
        Self {
            filename: filename.into(),
            file_url: None,
            file_data: Some(data_url.into()),
        }
    }

    /// Read a PDF from disk and base64-encode it into a data URL.
    pub fn from_pdf_path(filename: impl Into<String>, path: impl AsRef<Path>) -> Result<Self> {
        let bytes =
            std::fs::read(path).map_err(|_| Error::InvalidInput("failed to read pdf file"))?;
        let data_url = format!(
            "data:application/pdf;base64,{}",
            BASE64_STANDARD.encode(&bytes)
        );
        Ok(Self::from_data_url(filename, data_url))
    }

    fn into_part(self) -> ContentPart {
        ContentPart::File {
            file: FileRef {
                filename: Some(self.filename),
                file_data: self.file_data,
                file_url: self.file_url,
            },
        }
    }
}

/// Build a user message that references a PDF by URL.
pub fn create_user_message_with_pdf(
    text: impl Into<String>,
    url: impl Into<String>,
    filename: impl Into<String>,
) -> Message {
    user_message_with_parts(text, vec![File::from_url(filename, url).into_part()])
}

/// Build a user message with an on-disk PDF, base64-encoded inline.
pub fn create_user_message_with_base64_pdf(
    text: impl Into<String>,
    path: impl AsRef<Path>,
    filename: impl Into<String>,
) -> Result<Message> {
    let file = File::from_pdf_path(filename, path)?;
    Ok(user_message_with_parts(text, vec![file.into_part()]))
}

/// Build a user message containing several attached files.
pub fn create_user_message_with_files(text: impl Into<String>, files: Vec<File>) -> Message {
    let parts = files.into_iter().map(File::into_part).collect();
    user_message_with_parts(text, parts)
}

/// Build a file-parser plugin with the given engine selection.
///
/// `FileParserEngine::Auto` omits `pdf.engine` so OpenRouter chooses.
pub fn create_file_parser_plugin(engine: FileParserEngine) -> Plugin {
    let pdf = engine.as_str().map(|e| FilePdfConfig {
        engine: Some(e.to_string()),
    });
    Plugin::File(FilePluginConfig { pdf })
}

// ---------------------------------------------------------------------------
// Audio
// ---------------------------------------------------------------------------

/// Inline audio format.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AudioFormat {
    Wav,
    Mp3,
}

impl AudioFormat {
    fn as_str(self) -> &'static str {
        match self {
            AudioFormat::Wav => "wav",
            AudioFormat::Mp3 => "mp3",
        }
    }

    fn from_path(path: &Path) -> Result<Self> {
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_ascii_lowercase());
        match ext.as_deref() {
            Some("wav") => Ok(AudioFormat::Wav),
            Some("mp3") => Ok(AudioFormat::Mp3),
            _ => Err(Error::InvalidInput("unsupported audio format")),
        }
    }
}

/// Read an audio file from disk and attach it to a user message.
/// Format is inferred from the extension (`.wav` / `.mp3`).
pub fn create_user_message_with_audio(
    text: impl Into<String>,
    path: impl AsRef<Path>,
) -> Result<Message> {
    let path = path.as_ref();
    let format = AudioFormat::from_path(path)?;
    let bytes =
        std::fs::read(path).map_err(|_| Error::InvalidInput("failed to read audio file"))?;
    Ok(create_user_message_with_audio_bytes(text, &bytes, format))
}

/// Attach in-memory audio bytes to a user message.
pub fn create_user_message_with_audio_bytes(
    text: impl Into<String>,
    bytes: &[u8],
    format: AudioFormat,
) -> Message {
    user_message_with_parts(
        text,
        vec![ContentPart::InputAudio {
            input_audio: InputAudio {
                data: BASE64_STANDARD.encode(bytes),
                format: format.as_str().to_string(),
            },
        }],
    )
}

// ---------------------------------------------------------------------------
// Text files
// ---------------------------------------------------------------------------

const ALLOWED_TEXT_EXTENSIONS: &[&str] = &[
    "txt", "md", "json", "yaml", "yml", "toml", "csv", "log", "xml", "html", "css", "js", "ts",
    "py", "rs", "go", "java", "c", "cpp", "h", "sh",
];

fn check_text_extension(path: &Path) -> Result<()> {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_ascii_lowercase());
    match ext {
        Some(e) if ALLOWED_TEXT_EXTENSIONS.contains(&e.as_str()) => Ok(()),
        _ => Err(Error::InvalidInput("unsupported text file extension")),
    }
}

fn format_text_file(filename: &str, content: &str) -> String {
    format!("--- filename: {filename} ---\n{content}")
}

/// Read a UTF-8 text file from disk and inline it after `text` in a user message.
pub fn create_user_message_with_text_file(
    text: impl Into<String>,
    path: impl AsRef<Path>,
) -> Result<Message> {
    let path = path.as_ref();
    check_text_extension(path)?;
    let content = std::fs::read_to_string(path)
        .map_err(|_| Error::InvalidInput("failed to read text file (non-UTF-8?)"))?;
    let filename = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("file")
        .to_string();
    Ok(create_user_message_with_text_content(
        text, content, filename,
    ))
}

/// Inline multiple UTF-8 text files into a single user message.
pub fn create_user_message_with_text_files(
    text: impl Into<String>,
    paths: &[impl AsRef<Path>],
) -> Result<Message> {
    let mut combined = String::new();
    for path in paths {
        let path = path.as_ref();
        check_text_extension(path)?;
        let content = std::fs::read_to_string(path)
            .map_err(|_| Error::InvalidInput("failed to read text file (non-UTF-8?)"))?;
        let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("file");
        if !combined.is_empty() {
            combined.push_str("\n\n");
        }
        combined.push_str(&format_text_file(filename, &content));
    }
    let text = text.into();
    let body = if text.is_empty() {
        combined
    } else {
        format!("{text}\n\n{combined}")
    };
    Ok(Message::user(body))
}

/// Build a user message with already-loaded text content (no I/O).
pub fn create_user_message_with_text_content(
    text: impl Into<String>,
    content: impl Into<String>,
    filename: impl Into<String>,
) -> Message {
    let text = text.into();
    let formatted = format_text_file(&filename.into(), &content.into());
    let body = if text.is_empty() {
        formatted
    } else {
        format!("{text}\n\n{formatted}")
    };
    Message::user(body)
}

// ---------------------------------------------------------------------------
// ContentBuilder
// ---------------------------------------------------------------------------

/// Fluent builder for composing a multimodal `Message` from interleaved
/// text, image, file, and audio parts.
///
/// ```
/// use openrouter::{ContentBuilder, ImageDetail, Role};
///
/// let msg = ContentBuilder::new()
///     .add_text("Compare these:")
///     .add_image("https://example.com/a.png")
///     .add_image_with_detail("https://example.com/b.png", ImageDetail::High)
///     .build_message(Role::User);
/// assert_eq!(msg.role, Role::User);
/// ```
#[derive(Clone, Debug, Default)]
pub struct ContentBuilder {
    parts: Vec<ContentPart>,
}

impl ContentBuilder {
    /// New empty builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a plain-text part.
    pub fn add_text(mut self, text: impl Into<String>) -> Self {
        self.parts.push(ContentPart::Text { text: text.into() });
        self
    }

    /// Append an image-URL part.
    pub fn add_image(mut self, url: impl Into<String>) -> Self {
        self.parts.push(ContentPart::ImageUrl {
            image_url: ImageUrl {
                url: url.into(),
                detail: None,
            },
        });
        self
    }

    /// Append an image-URL part with an explicit detail level.
    pub fn add_image_with_detail(mut self, url: impl Into<String>, detail: ImageDetail) -> Self {
        self.parts.push(ContentPart::ImageUrl {
            image_url: ImageUrl {
                url: url.into(),
                detail: Some(detail.as_str().to_string()),
            },
        });
        self
    }

    /// Read an image from disk, base64-encode it, and append it.
    pub fn add_base64_image(mut self, path: impl AsRef<Path>) -> Result<Self> {
        let data_url = encode_image_to_base64(path)?;
        self.parts.push(ContentPart::ImageUrl {
            image_url: ImageUrl {
                url: data_url,
                detail: None,
            },
        });
        Ok(self)
    }

    /// Append a PDF reference by URL.
    pub fn add_pdf(mut self, url: impl Into<String>, filename: impl Into<String>) -> Self {
        self.parts.push(File::from_url(filename, url).into_part());
        self
    }

    /// Read a PDF from disk, base64-encode it, and append it.
    pub fn add_base64_pdf(
        mut self,
        path: impl AsRef<Path>,
        filename: impl Into<String>,
    ) -> Result<Self> {
        self.parts
            .push(File::from_pdf_path(filename, path)?.into_part());
        Ok(self)
    }

    /// Read an audio file from disk and append it.
    pub fn add_audio(mut self, path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let format = AudioFormat::from_path(path)?;
        let bytes =
            std::fs::read(path).map_err(|_| Error::InvalidInput("failed to read audio file"))?;
        self.parts.push(ContentPart::InputAudio {
            input_audio: InputAudio {
                data: BASE64_STANDARD.encode(&bytes),
                format: format.as_str().to_string(),
            },
        });
        Ok(self)
    }

    /// Read a UTF-8 text file and append its contents as a text part.
    pub fn add_text_file(mut self, path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        check_text_extension(path)?;
        let content = std::fs::read_to_string(path)
            .map_err(|_| Error::InvalidInput("failed to read text file (non-UTF-8?)"))?;
        let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("file");
        self.parts.push(ContentPart::Text {
            text: format_text_file(filename, &content),
        });
        Ok(self)
    }

    /// Append an arbitrary pre-built part (escape hatch).
    pub fn add_part(mut self, part: ContentPart) -> Self {
        self.parts.push(part);
        self
    }

    /// Finalize as a `Message` with the given role.
    pub fn build_message(self, role: Role) -> Message {
        Message {
            role,
            content: Content::Parts(self.parts),
            name: None,
            tool_calls: None,
            tool_call_id: None,
            reasoning: None,
            annotations: None,
        }
    }

    /// Finalize as a raw `Vec<ContentPart>`.
    pub fn build_parts(self) -> Vec<ContentPart> {
        self.parts
    }
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

fn user_message_with_parts(text: impl Into<String>, extras: Vec<ContentPart>) -> Message {
    let text = text.into();
    let mut parts: Vec<ContentPart> = Vec::with_capacity(1 + extras.len());
    if !text.is_empty() {
        parts.push(ContentPart::Text { text });
    }
    parts.extend(extras);
    Message {
        role: Role::User,
        content: Content::Parts(parts),
        name: None,
        tool_calls: None,
        tool_call_id: None,
        reasoning: None,
        annotations: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Annotation, FileAnnotation};
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn image_url_message_shape() {
        let m = create_user_message_with_image("look", "https://x/y.png");
        let v = serde_json::to_value(&m).unwrap();
        assert_eq!(
            v,
            json!({
                "role": "user",
                "content": [
                    {"type": "text", "text": "look"},
                    {"type": "image_url", "image_url": {"url": "https://x/y.png"}},
                ]
            })
        );
    }

    #[test]
    fn image_detail_serializes_lowercase() {
        let m = create_user_message_with_image_detail("x", "https://x/y.png", ImageDetail::High);
        let v = serde_json::to_value(&m).unwrap();
        let parts = v["content"].as_array().unwrap();
        assert_eq!(parts[1]["image_url"]["detail"], json!("high"));
    }

    #[test]
    fn multiple_images_attach_all() {
        let m = create_user_message_with_images("look", ["a", "b", "c"]);
        if let Content::Parts(parts) = &m.content {
            assert_eq!(parts.len(), 4); // 1 text + 3 images
        } else {
            panic!("expected parts");
        }
    }

    #[test]
    fn encode_image_bytes_produces_data_url() {
        let url = encode_image_bytes_to_base64(&[0x89, 0x50, 0x4e, 0x47], "image/png");
        assert!(url.starts_with("data:image/png;base64,"));
        assert!(url.ends_with("iVBORw=="));
    }

    #[test]
    fn unsupported_image_extension_rejected() {
        let err = encode_image_to_base64("/tmp/foo.bmp").unwrap_err();
        match err {
            Error::InvalidInput(m) => assert_eq!(m, "unsupported image format"),
            _ => panic!("expected InvalidInput"),
        }
    }

    #[test]
    fn pdf_url_message_shape() {
        let m = create_user_message_with_pdf("summarize", "https://x/p.pdf", "p.pdf");
        let v = serde_json::to_value(&m).unwrap();
        assert_eq!(
            v,
            json!({
                "role": "user",
                "content": [
                    {"type": "text", "text": "summarize"},
                    {"type": "file", "file": {"filename": "p.pdf", "file_url": "https://x/p.pdf"}},
                ]
            })
        );
    }

    #[test]
    fn file_parser_plugin_pdf_text() {
        let plugin = create_file_parser_plugin(FileParserEngine::PdfText);
        let v = serde_json::to_value(&plugin).unwrap();
        assert_eq!(
            v,
            json!({"id": "file-parser", "pdf": {"engine": "pdf-text"}})
        );
    }

    #[test]
    fn file_parser_plugin_auto_omits_engine() {
        let plugin = create_file_parser_plugin(FileParserEngine::Auto);
        let v = serde_json::to_value(&plugin).unwrap();
        assert_eq!(v, json!({"id": "file-parser"}));
    }

    #[test]
    fn file_parser_plugin_mistral_ocr() {
        let plugin = create_file_parser_plugin(FileParserEngine::MistralOcr);
        let v = serde_json::to_value(&plugin).unwrap();
        assert_eq!(v["pdf"]["engine"], json!("mistral-ocr"));
    }

    #[test]
    fn file_annotation_roundtrip() {
        let ann = Annotation::File {
            file: FileAnnotation {
                filename: "p.pdf".into(),
                file_data: "data:application/pdf;base64,AAA=".into(),
            },
        };
        let v = serde_json::to_value(&ann).unwrap();
        assert_eq!(
            v,
            json!({"type": "file", "file": {"filename": "p.pdf", "file_data": "data:application/pdf;base64,AAA="}})
        );
        let back: Annotation = serde_json::from_value(v).unwrap();
        assert_eq!(back, ann);
    }

    #[test]
    fn url_citation_annotation_still_works() {
        let v = json!({"type": "url_citation", "url_citation": {"url": "https://x"}});
        let a: Annotation = serde_json::from_value(v).unwrap();
        match a {
            Annotation::UrlCitation { url_citation } => assert_eq!(url_citation.url, "https://x"),
            _ => panic!("expected UrlCitation"),
        }
    }

    #[test]
    fn audio_bytes_message_shape() {
        let m = create_user_message_with_audio_bytes("transcribe", &[1, 2, 3], AudioFormat::Wav);
        let v = serde_json::to_value(&m).unwrap();
        let parts = v["content"].as_array().unwrap();
        assert_eq!(parts[1]["type"], json!("input_audio"));
        assert_eq!(parts[1]["input_audio"]["format"], json!("wav"));
        assert_eq!(parts[1]["input_audio"]["data"], json!("AQID"));
    }

    #[test]
    fn text_content_helper_no_io() {
        let m = create_user_message_with_text_content("ctx", "hello", "g.txt");
        let body = m.content_text().unwrap();
        assert!(body.starts_with("ctx\n\n--- filename: g.txt ---\n"));
        assert!(body.ends_with("hello"));
    }

    #[test]
    fn text_content_helper_empty_prefix() {
        let m = create_user_message_with_text_content("", "hello", "g.txt");
        assert_eq!(m.content_text().unwrap(), "--- filename: g.txt ---\nhello");
    }

    #[test]
    fn text_extension_whitelist_rejects_exe() {
        let err = create_user_message_with_text_file("x", "/tmp/foo.exe").unwrap_err();
        match err {
            Error::InvalidInput(m) => assert_eq!(m, "unsupported text file extension"),
            _ => panic!("expected InvalidInput"),
        }
    }

    #[test]
    fn content_builder_chains() {
        let m = ContentBuilder::new()
            .add_text("look at")
            .add_image("https://x/y.png")
            .add_image_with_detail("https://x/z.png", ImageDetail::Low)
            .build_message(Role::User);
        let v = serde_json::to_value(&m).unwrap();
        let parts = v["content"].as_array().unwrap();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0]["type"], json!("text"));
        assert_eq!(parts[1]["type"], json!("image_url"));
        assert_eq!(parts[2]["image_url"]["detail"], json!("low"));
    }

    #[test]
    fn content_builder_pdf_url() {
        let parts = ContentBuilder::new()
            .add_text("summarize")
            .add_pdf("https://x/p.pdf", "p.pdf")
            .build_parts();
        assert_eq!(parts.len(), 2);
        match &parts[1] {
            ContentPart::File { file } => {
                assert_eq!(file.filename.as_deref(), Some("p.pdf"));
                assert_eq!(file.file_url.as_deref(), Some("https://x/p.pdf"));
                assert!(file.file_data.is_none());
            }
            _ => panic!("expected file part"),
        }
    }

    #[test]
    fn multi_files_helper() {
        let m = create_user_message_with_files(
            "compare",
            vec![
                File::from_url("a.pdf", "https://x/a.pdf"),
                File::from_url("b.pdf", "https://x/b.pdf"),
            ],
        );
        if let Content::Parts(parts) = &m.content {
            assert_eq!(parts.len(), 3); // text + 2 files
        } else {
            panic!("expected parts");
        }
    }
}
