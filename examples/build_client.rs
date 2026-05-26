//! Phase 1 smoke example: construct a client and print its debug view.
//! No network calls — endpoints land in Phase 2.

#![allow(clippy::result_large_err)]

fn main() -> openrouter::Result<()> {
    let client = openrouter::Client::builder()
        .api_key("sk-test-placeholder")
        .app_name("openrouter-rust-example")
        .referer("https://github.com/hra42/openrouter-rust")
        .build()?;
    println!("{client:#?}");
    Ok(())
}
