use std::{collections::VecDeque, env};

use gemini_rust::{
    Content, FunctionCall, FunctionCallingMode, FunctionDeclaration, FunctionResponse, Gemini,
    Part, Role, ThinkingConfig,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, JsonSchema)]
struct Command {
    /// The command to run
    command: String,
    /// The command arguments
    arguments: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, JsonSchema)]
struct RootCommander {
    /// The current step number (starts at 1)
    attempt: i64,
    /// The command to use
    command: Command,
}

#[derive(Deserialize, Serialize, Debug, JsonSchema)]
struct StatusResponse {
    /// The status of the operation
    status: bool,
    /// Additional details about the operation
    detail: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("GEMINI_API_KEY")?;
    let client = Gemini::pro(api_key).expect("unable to cheate Gemini API client");

    let commander_tool = FunctionDeclaration::new(
        "execute_command",
        "Execute a system command with parameters",
        None,
    )
    .with_parameters::<RootCommander>()
    .with_response::<StatusResponse>();

    println!("Sending function response...",);

    let response = client
        .generate_content()
        .with_thinking_config(ThinkingConfig::dynamic_thinking())
        .with_temperature(0.1)
        .with_top_p(0.95)
        .with_function(commander_tool.clone())
        .with_function_calling_mode(FunctionCallingMode::Any)
        .with_user_message(
            "I need you to run a system command 'bleep' with parameters 'boop' and 'bop'.",
        )
        .execute()
        .await?;

    let contents = response
        .candidates
        .into_iter()
        .map(|c| c.content)
        .collect::<Vec<_>>();

    let mut function_queue = VecDeque::<FunctionCall>::new();
    for content in &contents {
        if let Some(parts) = &content.parts {
            for part in parts {
                if let Part::FunctionCall { function_call, .. } = part {
                    function_queue.push_front(function_call.clone());
                }
                if let Part::FunctionResponse { function_response } = part {
                    if let Some(last_call) = function_queue.pop_front() {
                        if last_call.name != function_response.name {
                            eprintln!(
                                "Warning: Function response name '{}' does not match last function call name '{}'",
                                function_response.name, last_call.name
                            );
                        }
                    } else {
                        eprintln!(
                            "Warning: Function response name '{}' has no matching function call",
                            function_response.name
                        );
                    }
                }
            }
        }
    }

    let mut reply = client.generate_content();

    reply.contents.extend(contents);

    for function_call in function_queue {
        println!(
            "Function call received: {} with args:\n{}",
            function_call.name,
            serde_json::to_string_pretty(&function_call.args)?
        );
        let result = serde_json::from_value::<RootCommander>(function_call.args.clone())?;

        // Simulate command execution
        let Command { command, arguments } = result.command;
        let status = StatusResponse {
            status: true,
            detail: format!(
                "Command '{}' executed successfully with arguments: {:?}",
                command, arguments
            ),
        };

        let content = Content::function_response(FunctionResponse::from_schema(
            function_call.name.clone(),
            status,
        )?)
        .with_role(Role::User);
        reply.contents.push(content);
    }

    println!("Sending function response...",);

    let final_response = reply.execute().await?;

    println!("Final response from model: {}", final_response.text(),);

    Ok(())
}
