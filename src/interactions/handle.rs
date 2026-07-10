use std::{sync::Arc, time::Duration};

use tokio::time::sleep;
use tracing::instrument;

use crate::client::{Error, GeminiClient};
use crate::interactions::model::*;
use crate::interactions::stream::InteractionStream;

/// Handle to an Interaction, usable for get / cancel / delete / poll operations.
#[derive(Clone)]
pub struct InteractionHandle {
    id: String,
    client: Arc<GeminiClient>,
}

impl InteractionHandle {
    pub(crate) fn new(id: String, client: Arc<GeminiClient>) -> Self {
        Self { id, client }
    }

    /// Get the interaction ID.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get the full interaction resource.
    #[instrument(skip(self))]
    pub async fn get(&self) -> Result<Interaction, Error> {
        self.client.get_interaction(&self.id).await
    }

    /// Get the interaction in streaming mode (can resume from last_event_id).
    #[instrument(skip(self), fields(last_event_id = last_event_id.unwrap_or("")))]
    pub async fn get_stream(
        &self,
        last_event_id: Option<&str>,
    ) -> Result<InteractionStream, Error> {
        self.client
            .get_interaction_stream(&self.id, last_event_id)
            .await
    }

    /// Cancel the interaction (only applicable to background executions).
    #[instrument(skip(self))]
    pub async fn cancel(&self) -> Result<Interaction, Error> {
        self.client.cancel_interaction(&self.id).await
    }

    /// Delete the interaction.
    #[instrument(skip(self))]
    pub async fn delete(&self) -> Result<(), Error> {
        self.client.delete_interaction(&self.id).await
    }

    /// Poll until the interaction reaches a terminal state.
    ///
    /// Continuously calls `get()` until the status becomes
    /// completed / failed / cancelled / incomplete / budget_exceeded.
    /// Suitable for background interactions.
    #[instrument(skip(self), fields(poll.interval = ?interval))]
    pub async fn poll_until_completed(
        &self,
        interval: Duration,
    ) -> Result<Interaction, Error> {
        loop {
            let interaction = self.get().await?;

            if interaction.status.is_terminal() {
                return Ok(interaction);
            }

            sleep(interval).await;
        }
    }

    /// Get this interaction's ID for use as `previous_interaction_id` in the next turn.
    pub fn as_previous_interaction_id(&self) -> &str {
        &self.id
    }
}
