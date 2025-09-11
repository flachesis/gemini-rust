use gemini_rust::{prelude::*, tools::model::FunctionCallingMode};
use serde_json::json;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get API key from environment variable
    let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY environment variable not set");

    // Create client
    let client = Gemini::new(api_key).expect("unable to create Gemini API client");

    println!("--- Meeting Scheduler Function Calling example ---");

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
        println!(
            "Function call: {} with args: {}",
            function_call.name, function_call.args
        );

        // Handle the schedule_meeting function
        if function_call.name == "schedule_meeting" {
            let attendees: Vec<String> = function_call.get("attendees")?;
            let date: String = function_call.get("date")?;
            let time: String = function_call.get("time")?;
            let topic: String = function_call.get("topic")?;

            println!("Scheduling meeting:");
            println!("  Attendees: {:?}", attendees);
            println!("  Date: {}", date);
            println!("  Time: {}", time);
            println!("  Topic: {}", topic);

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

            println!("Final response: {}", final_response.text());
        } else {
            println!("Unknown function call: {}", function_call.name);
        }
    } else {
        println!("No function calls in the response.");
        println!("Direct response: {}", response.text());
    }

    Ok(())
}
