use gemini_rust::{
    Content, FunctionCallingMode, FunctionDeclaration, FunctionParameters, Gemini, Part,
    PropertyDetails,
};
use std::env;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber
    tracing_subscriber::fmt::init();
    let api_key = env::var("GEMINI_API_KEY")?;

    // Create client
    let client = Gemini::new(api_key).expect("unable to cheate Gemini API client");

    // Define a weather function
    let get_weather = FunctionDeclaration::new(
        "get_weather",
        "Get the current weather for a location",
        FunctionParameters::object()
            .with_property(
                "location",
                PropertyDetails::string("The city and state, e.g., San Francisco, CA"),
                true,
            )
            .with_property(
                "unit",
                PropertyDetails::enum_type("The unit of temperature", ["celsius", "fahrenheit"]),
                false,
            ),
    );

    // Create a request with function calling
    info!("sending function call request");
    let response = client
        .generate_content()
        .with_user_message("What's the weather like in Tokyo right now?")
        .with_function(get_weather)
        .with_function_calling_mode(FunctionCallingMode::Any)
        .execute()
        .await?;

    // Check if there are function calls
    if let Some(function_call) = response.function_calls().first() {
        info!(
            function_name = function_call.name,
            args = ?function_call.args,
            "function call received"
        );

        // Get parameters from the function call
        let location: String = function_call.get("location")?;
        let unit = function_call
            .get::<String>("unit")
            .unwrap_or_else(|_| String::from("celsius"));

        info!(
            location = location,
            unit = unit,
            "function parameters extracted"
        );

        // Simulate function execution (in a real app, this would call a weather API)
        // Create a JSON response object
        let weather_response = serde_json::json!({
            "temperature": 22,
            "unit": unit,
            "condition": "sunny",
            "location": location
        });

        // Continue the conversation with the function result
        // We need to replay the entire conversation with the function response
        info!("sending function response");

        // First, need to recreate the original prompt and the model's response
        let mut final_request = client
            .generate_content()
            .with_user_message("What's the weather like in Tokyo right now?");

        // Add the function call from the model's response
        let call_content = Content {
            parts: Some(vec![Part::FunctionCall {
                function_call: (*function_call).clone(),
                thought_signature: None,
            }]),
            ..Default::default()
        };
        final_request.contents.push(call_content);

        // Now add the function response using the JSON value
        final_request = final_request.with_function_response("get_weather", weather_response);

        // Execute the request
        let final_response = final_request.execute().await?;

        info!(response = final_response.text(), "final response received");
    } else {
        info!("no function calls in response");
        info!(response = response.text(), "direct response received");
    }

    Ok(())
}
