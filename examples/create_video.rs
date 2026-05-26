//! Submit a video generation job, poll until done, and download the first
//! output to `out.mp4`.
//!
//! This example actually consumes credits if run against the live API.
//! Without `OPENROUTER_RUN_VIDEO=1` it stops after listing available video
//! models so it's safe to include in `examples/run_all.rs`.
//!
//! Run with:
//!
//! ```bash
//! OPENROUTER_API_KEY=sk-... OPENROUTER_RUN_VIDEO=1 \
//!     cargo run --example create_video
//! ```

use std::time::Duration;

use openrouter::{Client, VideoAspectRatio, VideoGenerationRequest, VideoResolution};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Ok(api_key) = std::env::var("OPENROUTER_API_KEY") else {
        eprintln!("OPENROUTER_API_KEY not set — skipping.");
        return Ok(());
    };
    let client = Client::builder()
        .api_key(api_key)
        .app_name("openrouter-rust create_video example")
        .build()?;

    let models = client.list_video_models().await?;
    println!("Available video models ({}):", models.data.len());
    for m in models.data.iter().take(5) {
        println!("  {:<32}  {}", m.id, m.name);
    }

    if std::env::var("OPENROUTER_RUN_VIDEO").ok().as_deref() != Some("1") {
        eprintln!("\nSet OPENROUTER_RUN_VIDEO=1 to actually submit a job.");
        return Ok(());
    }

    let model = models
        .data
        .first()
        .map(|m| m.id.clone())
        .ok_or("no video models available")?;
    println!("\nSubmitting a job to {model}…");
    let job = client
        .create_video(&VideoGenerationRequest {
            model,
            prompt: "A cat surfing in slow motion, cinematic lighting".into(),
            aspect_ratio: Some(VideoAspectRatio::R16x9),
            resolution: Some(VideoResolution::P720),
            ..Default::default()
        })
        .await?;
    println!("  id: {}  status: {:?}", job.id, job.status);

    println!("\nPolling…");
    let final_resp = client
        .wait_for_video(&job.id, Duration::from_secs(5))
        .await?;
    println!("  final status: {:?}", final_resp.status);
    if let Some(usage) = &final_resp.usage {
        println!("  cost: {:?}  byok: {}", usage.cost, usage.is_byok);
    }

    println!("\nDownloading default output…");
    let content = client.get_video_content(&job.id, 0).await?;
    let out_path = "out.mp4";
    std::fs::write(out_path, &content.content)?;
    println!(
        "  wrote {} bytes (Content-Type: {}) to {out_path}",
        content.content.len(),
        content.content_type.as_deref().unwrap_or("?"),
    );
    Ok(())
}
