// 範例影片請放在 examples/sample.mp4
// 這個範例會將 mp4 影片內容傳給 Gemini API，請 AI 描述影片內容

use gemini_rust::{Gemini, Content};
use std::fs::File;
use std::io::Read;
use std::env;
use base64::{engine::general_purpose, Engine as _};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 讀取 mp4 影片檔案
    let mut file = File::open("examples/sample.mp4")?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let b64 = general_purpose::STANDARD.encode(&buffer);

    // 取得 API key
    let api_key = env::var("GEMINI_API_KEY")?;
    let gemini = Gemini::pro(api_key);

    // 建立 mp4 blob
    let video_content = Content::inline_data("video/mp4", b64);

    // 呼叫 Gemini API
    let response = gemini
        .generate_content()
        .with_user_message("Please describe this video")
        .with_message(gemini_rust::Message {
            content: video_content,
            role: gemini_rust::Role::User,
        })
        .execute()
        .await?;

    println!("AI 描述: {}", response.text());
    Ok(())
}
