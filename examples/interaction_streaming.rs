use display_error_chain::DisplayErrorChain;
use futures::TryStreamExt;
use gemini_rust::prelude::*;
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
    let api_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY environment variable not set");
    let client = Gemini::new(api_key)?;

    info!("interaction streaming example starting");

    let mut stream = client
        .create_interaction()
        .with_model("gemini-flash-latest")
        .with_text("Tell me a short story about a programmer who discovers a magical bug in their code")
        .execute_stream()
        .await?;

    let mut full_response = String::new();
    while let Some(event) = stream.try_next().await? {
        match event {
            InteractionEvent::StepStart { index, step, .. } => {
                info!(step.index = index, step.type_ = std::format!("{:?}", step), "step started");
            }
            InteractionEvent::StepDelta { index, delta, .. } => {
                if let StepDeltaData::Text { text } = delta {
                    info!(step.index = index, text = %text, "delta");
                    full_response.push_str(&text);
                }
            }
            InteractionEvent::StepStop { index, usage, .. } => {
                info!(step.index = index, "step completed");
                if let Some(u) = usage {
                    info!(
                        step.index = index,
                        total_tokens = u.total_tokens,
                        "step usage"
                    );
                }
            }
            InteractionEvent::InteractionCompleted { interaction, .. } => {
                info!(
                    status = interaction.status.as_ref(),
                    "interaction completed"
                );
            }
            _ => {}
        }
    }

    info!(
        response_length = full_response.len(),
        "streaming response completed"
    );

    info!("interaction streaming example completed");
    Ok(())
}
