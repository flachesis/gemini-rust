use gemini_rust::Gemini;
use serde_json::json;
use std::env;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber
    tracing_subscriber::fmt::init();

    // Get API key from environment variable
    let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY environment variable not set");

    // Create client
    let client = Gemini::new(api_key).expect("unable to create Gemini API client");

    // Using response_schema for structured output
    info!("starting structured response example");

    // Define a JSON schema for the response
    let schema = json!({
        "type": "object",
        "properties": {
            "name": {
                "type": "string",
                "description": "Name of the programming language"
            },
            "year_created": {
                "type": "integer",
                "description": "Year the programming language was created"
            },
            "creator": {
                "type": "string",
                "description": "Person or organization who created the language"
            },
            "key_features": {
                "type": "array",
                "items": {
                    "type": "string"
                },
                "description": "Key features of the programming language"
            },
            "popularity_score": {
                "type": "integer",
                "description": "Subjective popularity score from 1-10"
            }
        },
        "required": ["name", "year_created", "creator", "key_features", "popularity_score"]
    });

    let response = client
        .generate_content()
        .with_system_prompt("You provide information about programming languages in JSON format.")
        .with_user_message("Tell me about the Rust programming language.")
        .with_response_mime_type("application/json")
        .with_response_schema(schema)
        .execute()
        .await?;

    info!(
        response = response.text(),
        "structured json response received"
    );

    // Parse the JSON response
    let json_response: serde_json::Value = serde_json::from_str(&response.text())?;

    info!(
        language = json_response["name"].as_str().unwrap_or("unknown"),
        year = json_response["year_created"].as_i64().unwrap_or(0),
        creator = json_response["creator"].as_str().unwrap_or("unknown"),
        popularity = json_response["popularity_score"].as_i64().unwrap_or(0),
        "parsed structured response fields"
    );

    if let Some(features) = json_response["key_features"].as_array() {
        for (i, feature) in features.iter().enumerate() {
            info!(
                index = i + 1,
                feature = feature.as_str().unwrap_or("unknown"),
                "key feature"
            );
        }
    }

    Ok(())
}
