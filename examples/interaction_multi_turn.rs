use display_error_chain::DisplayErrorChain;
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
    let api_key =
        std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY environment variable not set");
    let client = Gemini::new(api_key)?;

    info!("multi-turn conversation example (stateful)");

    let interaction1 = client
        .create_interaction()
        .with_model("gemini-flash-latest")
        .with_text("I have 2 dogs in my house.")
        .execute()
        .await?;

    info!(response = interaction1.output_text(), "response 1");

    let id1 = interaction1.id().expect("interaction should have an id");

    let interaction2 = client
        .create_interaction()
        .with_model("gemini-flash-latest")
        .with_text("How many paws are in my house?")
        .with_previous_interaction(id1)
        .execute()
        .await?;

    info!(
        response = interaction2.output_text(),
        "response 2 (uses server-side state)"
    );

    info!("multi-turn conversation example (stateless)");

    let history = vec![Step::UserInput {
        content: vec![InteractionContent::text("I have 2 dogs in my house.")],
    }];

    let interaction3 = client
        .create_interaction()
        .with_model("gemini-flash-latest")
        .with_store(false)
        .with_step_input(history.clone())
        .execute()
        .await?;

    info!(
        response = interaction3.output_text(),
        "response 3a (stateless)"
    );

    let mut history2 = history;
    for step in &interaction3.steps {
        history2.push(step.clone());
    }
    history2.push(Step::UserInput {
        content: vec![InteractionContent::text("How many paws are in my house?")],
    });

    let interaction4 = client
        .create_interaction()
        .with_model("gemini-flash-latest")
        .with_store(false)
        .with_step_input(history2)
        .execute()
        .await?;

    info!(
        response = interaction4.output_text(),
        "response 4 (stateless, client-side history)"
    );

    info!("multi-turn example completed");
    Ok(())
}
