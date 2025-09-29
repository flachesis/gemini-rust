use display_error_chain::DisplayErrorChain;
use gemini_rust::{
    Content, FunctionCall, FunctionCallingMode, FunctionDeclaration, FunctionParameters, Gemini,
    Message, PropertyDetails, Role, Tool,
};
use serde_json::json;
use std::env;
use std::process::ExitCode;
use tracing::info;

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing::level_filters::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    match do_main().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            let error_chain = DisplayErrorChain::new(e.as_ref());
            tracing::error!(error.debug = ?e, error.chained = %error_chain, "execution failed");
            ExitCode::FAILURE
        }
    }
}

async fn do_main() -> Result<(), Box<dyn std::error::Error>> {
    // Get API key from environment variable
    let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY environment variable not set");

    // Create client
    let client = Gemini::new(api_key).expect("unable to create Gemini API client");

    info!("starting meeting scheduler function calling example");

    // Define a meeting scheduler function that matches the curl example
    let schedule_meeting = FunctionDeclaration::new(
        "schedule_meeting",
        "Schedules a meeting with specified attendees at a given time and date.",
        FunctionParameters::object()
            .with_property(
                "attendees",
                PropertyDetails::array(
                    "List of people attending the meeting.",
                    PropertyDetails::string("Attendee name"),
                ),
                true,
            )
            .with_property(
                "date",
                PropertyDetails::string("Date of the meeting (e.g., '2024-07-29')"),
                true,
            )
            .with_property(
                "time",
                PropertyDetails::string("Time of the meeting (e.g., '15:00')"),
                true,
            )
            .with_property(
                "topic",
                PropertyDetails::string("The subject or topic of the meeting."),
                true,
            ),
    );

    // Create function tool
    let function_tool = Tool::new(schedule_meeting);

    // Create a request with the tool - matching the curl example
    let response = client
        .generate_content()
        .with_user_message("Schedule a meeting with Bob and Alice for 03/27/2025 at 10:00 AM about the Q3 planning.")
        .with_tool(function_tool.clone())
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

        // Handle the schedule_meeting function
        if function_call.name == "schedule_meeting" {
            let attendees: Vec<String> = function_call.get("attendees")?;
            let date: String = function_call.get("date")?;
            let time: String = function_call.get("time")?;
            let topic: String = function_call.get("topic")?;

            info!(
                attendees = ?attendees,
                date = date,
                time = time,
                topic = topic,
                "scheduling meeting"
            );

            // Simulate scheduling the meeting
            let meeting_id = format!(
                "meeting_{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            );

            let function_response = json!({
                "success": true,
                "meeting_id": meeting_id,
                "message": format!("Meeting '{}' scheduled for {} at {} with {:?}", topic, date, time, attendees)
            });

            // Create conversation with function response
            let mut conversation = client.generate_content();

            // 1. Add original user message
            conversation = conversation
                .with_user_message("Schedule a meeting with Bob and Alice for 03/27/2025 at 10:00 AM about the Q3 planning.");

            // 2. Add model message with function call
            let model_function_call =
                FunctionCall::new("schedule_meeting", function_call.args.clone());
            let model_content = Content::function_call(model_function_call).with_role(Role::Model);
            let model_message = Message {
                content: model_content,
                role: Role::Model,
            };
            conversation = conversation.with_message(model_message);

            // 3. Add function response
            conversation =
                conversation.with_function_response("schedule_meeting", function_response);

            // Execute final request
            let final_response = conversation.execute().await?;

            info!(response = final_response.text(), "final response received");
        } else {
            info!(
                function_name = function_call.name,
                "unknown function call received"
            );
        }
    } else {
        info!("no function calls in response");
        info!(response = response.text(), "direct response received");
    }

    Ok(())
}
