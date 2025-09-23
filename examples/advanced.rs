use gemini_rust::{
    Content, FunctionCallingMode, FunctionDeclaration, Gemini, Part, tools::GeminiSchema as _,
};
use schemars::{JsonSchema, SchemaGenerator};
use serde::{Deserialize, Serialize};
use std::env;
use termion::color;

#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(description = "The unit of temperature")]
#[serde(rename_all = "lowercase")]
enum Unit {
    #[default]
    Celsius,
    Fahrenheit,
}

impl std::fmt::Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Unit::Celsius => write!(f, "Celsius"),
            Unit::Fahrenheit => write!(f, "Fahrenheit"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
struct Weather {
    /// The city and state, e.g., San Francisco, CA
    location: String,
    /// The unit of temperature
    unit: Option<Unit>,
}

impl Default for Weather {
    fn default() -> Self {
        Weather {
            location: "".to_string(),
            unit: Some(Unit::Celsius),
        }
    }
}

impl std::fmt::Display for Weather {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let schema = SchemaGenerator::gemini().root_schema_for::<Self>();
        write!(f, "{}", schema.to_value())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct WeatherResponse {
    temperature: i32,
    unit: Unit,
    condition: String,
    location: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("GEMINI_API_KEY")?;

    // Create client
    let client = Gemini::new(api_key).expect("unable to cheate Gemini API client");

    // Define a weather function
    let get_weather = FunctionDeclaration::new(
        "get_weather",
        "Get the current weather for a location",
        None,
    )
    .with_parameters::<Weather>()
    .with_response::<WeatherResponse>();

    // Create a request with function calling
    println!("Sending function call request...");
    let response = client
        .generate_content()
        .with_user_message("What's the weather like in Tokyo right now?")
        .with_function(get_weather)
        .with_function_calling_mode(FunctionCallingMode::Any)
        .execute()
        .await?;

    let Some(function_call) = response.function_calls().first().cloned() else {
        eprintln!("No function calls in the response.");
        eprintln!(
            "{}Response:{} {}",
            color::Fg(color::LightBlack),
            color::Fg(color::Reset),
            response.text(),
        );
        return Ok(());
    };

    let result = serde_json::from_value::<Weather>(function_call.args.clone())?;

    println!(
        "{}Function call received:{} {} {}with args:{}\n{}",
        color::Fg(color::LightBlack),
        color::Fg(color::Reset),
        function_call.name,
        color::Fg(color::LightBlack),
        color::Fg(color::Reset),
        serde_json::to_string_pretty(&result)?
    );

    // Get parameters from the function call
    let location = result.location;
    let unit = result.unit.unwrap_or_default();

    // Simulate function execution (in a real app, this would call a weather API)
    // Create a JSON response object
    let weather_response = WeatherResponse {
        temperature: 22,
        unit,
        condition: "sunny".to_string(),
        location: location.clone(),
    };

    // Continue the conversation with the function result
    // We need to replay the entire conversation with the function response
    println!(
        "{}Sending function response...{}",
        color::Fg(color::LightBlack),
        color::Fg(color::Reset)
    );

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

    println!(
        "{}Final response:{} {}",
        color::Fg(color::LightBlack),
        color::Fg(color::Reset),
        final_response.text()
    );

    Ok(())
}
