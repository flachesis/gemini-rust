use display_error_chain::DisplayErrorChain;
use gemini_rust::prelude::*;
use std::process::ExitCode;
use std::time::Duration;
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

    info!("antigravity managed agent example starting");

    let interaction1 = client
        .create_interaction()
        .with_agent("antigravity-preview-05-2026")
        .with_text("Write a Python script that generates the first 20 Fibonacci numbers and saves them to fibonacci.txt. Then read the file and print its contents.")
        .with_environment_id("remote")
        .with_background()
        .execute()
        .await?;

    let id1 = interaction1.id().expect("interaction should have an id");
    let env_id = interaction1
        .environment_id
        .as_deref()
        .expect("interaction should have an environment_id");

    info!(
        interaction_id = id1,
        environment_id = env_id,
        status = interaction1.status.as_ref(),
        "first interaction started (provisioning sandbox)"
    );

    let handle1 = client.interaction(id1);
    let result1 = handle1.poll_until_completed(Duration::from_secs(5)).await?;

    info!(
        status = result1.status.as_ref(),
        "first interaction completed"
    );
    info!(response = result1.output_text(), "first response");

    info!(
        previous_interaction_id = id1,
        environment_id = env_id,
        "starting second interaction (reusing sandbox)"
    );

    let interaction2 = client
        .create_interaction()
        .with_agent("antigravity-preview-05-2026")
        .with_text("Now plot the Fibonacci sequence as a line chart and save it as chart.png.")
        .with_previous_interaction(id1)
        .with_environment_id(env_id)
        .with_background()
        .execute()
        .await?;

    let id2 = interaction2.id().expect("interaction should have an id");
    info!(
        interaction_id = id2,
        status = interaction2.status.as_ref(),
        "second interaction started"
    );

    let handle2 = client.interaction(id2);
    let result2 = handle2.poll_until_completed(Duration::from_secs(5)).await?;

    info!(
        status = result2.status.as_ref(),
        "second interaction completed"
    );
    info!(response = result2.output_text(), "second response");

    info!("antigravity managed agent example completed");
    Ok(())
}
