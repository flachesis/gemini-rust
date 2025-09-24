use gemini_rust::{Content, FunctionCallingMode, FunctionDeclaration, Gemini, Part, Schema};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("GEMINI_API_KEY")?;

    // Create client
    let client = Gemini::new(api_key).expect("unable to cheate Gemini API client");

    // Define a weather function
    let get_weather = FunctionDeclaration::new(
        "get_weather",
        "Get the current weather for a location",
        Schema::object()
            .with_property(
                "location",
                Schema::string("The city and state, e.g., San Francisco, CA"),
                true,
            )
            .with_property(
                "unit",
                Schema::enum_type("The unit of temperature", ["celsius", "fahrenheit"]),
                false,
            ),
    );

    // Create a request with function calling
    println!("Sending function call request...");
    let response = client
        .generate_content()
        .with_user_message("What's the weather like in Tokyo right now?")
        .with_function(get_weather)
        .with_function_calling_mode(FunctionCallingMode::Any)
        .execute()
        .await?;

    // Check if there are function calls
    if let Some(function_call) = response.function_calls().first() {
        println!(
            "Function call received: {} with args: {}",
            function_call.name, function_call.args
        );

        // Get parameters from the function call
        let location: String = function_call.get("location")?;
        let unit = function_call
            .get::<String>("unit")
            .unwrap_or_else(|_| String::from("celsius"));

        println!("Location: {}, Unit: {}", location, unit);

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
        println!("Sending function response...");

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

        println!("Final response: {}", final_response.text());
    } else {
        println!("No function calls in the response.");
        println!("Response text: {}", response.text());
    }

    Ok(())
}
