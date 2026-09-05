use std::sync::Arc;
use tracing::{instrument, Span};

use crate::client::{Error as ClientError, GeminiClient};
use crate::interactions::model::*;
use crate::interactions::stream::InteractionStream;

/// Fluent builder for constructing and executing interaction requests.
#[derive(Clone)]
pub struct InteractionBuilder {
    client: Arc<GeminiClient>,
    model: Option<String>,
    agent: Option<String>,
    input: Option<InteractionInput>,
    system_instruction: Option<String>,
    tools: Vec<InteractionTool>,
    response_format: Option<ResponseFormat>,
    store: Option<bool>,
    background: bool,
    generation_config: Option<InteractionGenerationConfig>,
    agent_config: Option<AgentConfig>,
    previous_interaction_id: Option<String>,
    environment: Option<EnvironmentConfigOrString>,
    cached_content: Option<String>,
    response_modalities: Vec<ResponseModality>,
    service_tier: Option<ServiceTier>,
    webhook_config: Option<WebhookConfig>,
}

impl InteractionBuilder {
    pub(crate) fn new(client: Arc<GeminiClient>) -> Self {
        Self {
            client,
            model: None,
            agent: None,
            input: None,
            system_instruction: None,
            tools: Vec::new(),
            response_format: None,
            store: None,
            background: false,
            generation_config: None,
            agent_config: None,
            previous_interaction_id: None,
            environment: None,
            cached_content: None,
            response_modalities: Vec::new(),
            service_tier: None,
            webhook_config: None,
        }
    }

    // ===== Model / Agent =====

    /// Set the model (mutually exclusive with agent).
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self.agent = None;
        self
    }

    /// Set the agent (mutually exclusive with model).
    pub fn with_agent(mut self, agent: impl Into<String>) -> Self {
        self.agent = Some(agent.into());
        self.model = None;
        self
    }

    // ===== Input =====

    /// Set input from anything that implements `Into<InteractionInput>`.
    pub fn with_input(mut self, input: impl Into<InteractionInput>) -> Self {
        self.input = Some(input.into());
        self
    }

    /// Set input as plain text.
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.input = Some(InteractionInput::Text(text.into()));
        self
    }

    /// Set input as a content array (multimodal).
    pub fn with_content_input(mut self, content: Vec<InteractionContent>) -> Self {
        self.input = Some(InteractionInput::ContentArray(content));
        self
    }

    /// Set input as a step array (stateless multi-turn).
    pub fn with_step_input(mut self, steps: Vec<Step>) -> Self {
        self.input = Some(InteractionInput::StepArray(steps));
        self
    }

    /// Add image input.
    pub fn with_image(mut self, data: impl Into<String>, mime_type: ImageMimeType) -> Self {
        let content = InteractionContent::image(data, mime_type);
        self.input = match self.input {
            Some(InteractionInput::ContentArray(mut arr)) => {
                arr.push(content);
                Some(InteractionInput::ContentArray(arr))
            }
            _ => Some(InteractionInput::ContentArray(vec![content])),
        };
        self
    }

    /// Add image input from a URI.
    pub fn with_image_uri(mut self, uri: impl Into<String>, mime_type: ImageMimeType) -> Self {
        let content = InteractionContent::image_uri(uri, mime_type);
        self.input = match self.input {
            Some(InteractionInput::ContentArray(mut arr)) => {
                arr.push(content);
                Some(InteractionInput::ContentArray(arr))
            }
            _ => Some(InteractionInput::ContentArray(vec![content])),
        };
        self
    }

    /// Add audio input.
    pub fn with_audio(mut self, data: impl Into<String>, mime_type: AudioMimeType) -> Self {
        let content = InteractionContent::audio(data, mime_type);
        self.input = match self.input {
            Some(InteractionInput::ContentArray(mut arr)) => {
                arr.push(content);
                Some(InteractionInput::ContentArray(arr))
            }
            _ => Some(InteractionInput::ContentArray(vec![content])),
        };
        self
    }

    /// Add video input from a URI.
    pub fn with_video(mut self, uri: impl Into<String>) -> Self {
        let content = InteractionContent::video_uri(uri);
        self.input = match self.input {
            Some(InteractionInput::ContentArray(mut arr)) => {
                arr.push(content);
                Some(InteractionInput::ContentArray(arr))
            }
            _ => Some(InteractionInput::ContentArray(vec![content])),
        };
        self
    }

    /// Add document input.
    pub fn with_document(mut self, data: impl Into<String>, mime_type: DocumentMimeType) -> Self {
        let content = InteractionContent::document(data, mime_type);
        self.input = match self.input {
            Some(InteractionInput::ContentArray(mut arr)) => {
                arr.push(content);
                Some(InteractionInput::ContentArray(arr))
            }
            _ => Some(InteractionInput::ContentArray(vec![content])),
        };
        self
    }

    // ===== System Instruction =====

    /// Set the system instruction.
    pub fn with_system_instruction(mut self, instruction: impl Into<String>) -> Self {
        self.system_instruction = Some(instruction.into());
        self
    }

    // ===== Tools =====

    /// Add a tool.
    pub fn with_tool(mut self, tool: InteractionTool) -> Self {
        self.tools.push(tool);
        self
    }

    /// Add tools.
    pub fn with_tools(mut self, tools: Vec<InteractionTool>) -> Self {
        self.tools.extend(tools);
        self
    }

    /// Add a function declaration tool.
    pub fn with_function(
        mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        parameters: serde_json::Value,
    ) -> Self {
        self.tools
            .push(InteractionTool::function(name, description, parameters));
        self
    }

    /// Enable Google Search.
    pub fn with_google_search(mut self) -> Self {
        self.tools.push(InteractionTool::google_search());
        self
    }

    /// Enable code execution.
    pub fn with_code_execution(mut self) -> Self {
        self.tools.push(InteractionTool::code_execution());
        self
    }

    /// Enable URL context.
    pub fn with_url_context(mut self) -> Self {
        self.tools.push(InteractionTool::url_context());
        self
    }

    /// Enable Google Maps.
    pub fn with_google_maps(mut self) -> Self {
        self.tools.push(InteractionTool::google_maps());
        self
    }

    /// Enable file search.
    pub fn with_file_search(mut self, store_names: Vec<String>) -> Self {
        self.tools.push(InteractionTool::file_search(store_names));
        self
    }

    /// Enable MCP Server.
    pub fn with_mcp_server(mut self, name: impl Into<String>, url: impl Into<String>) -> Self {
        self.tools.push(InteractionTool::mcp_server(name, url));
        self
    }

    /// Enable computer use.
    pub fn with_computer_use(mut self, environment: ComputerUseEnvironment) -> Self {
        self.tools.push(InteractionTool::computer_use(environment));
        self
    }

    /// Enable retrieval tool.
    pub fn with_retrieval(mut self, retrieval_types: Vec<RetrievalType>) -> Self {
        self.tools.push(InteractionTool::retrieval(retrieval_types));
        self
    }

    // ===== Response Format =====

    /// Set the response format.
    pub fn with_response_format(mut self, format: ResponseFormat) -> Self {
        self.response_format = Some(format);
        self
    }

    /// Set JSON structured output with a schema.
    pub fn with_json_schema(mut self, schema: serde_json::Value) -> Self {
        self.response_format = Some(ResponseFormat::json_schema(schema));
        self
    }

    // ===== Generation Config =====

    pub fn with_temperature(mut self, temperature: f64) -> Self {
        self.generation_config
            .get_or_insert_with(Default::default)
            .temperature = Some(temperature);
        self
    }

    pub fn with_top_p(mut self, top_p: f64) -> Self {
        self.generation_config
            .get_or_insert_with(Default::default)
            .top_p = Some(top_p);
        self
    }

    pub fn with_seed(mut self, seed: i64) -> Self {
        self.generation_config
            .get_or_insert_with(Default::default)
            .seed = Some(seed);
        self
    }

    pub fn with_stop_sequences(mut self, sequences: Vec<String>) -> Self {
        self.generation_config
            .get_or_insert_with(Default::default)
            .stop_sequences = sequences;
        self
    }

    pub fn with_max_output_tokens(mut self, tokens: i64) -> Self {
        self.generation_config
            .get_or_insert_with(Default::default)
            .max_output_tokens = Some(tokens);
        self
    }

    pub fn with_thinking_level(mut self, level: InteractionThinkingLevel) -> Self {
        self.generation_config
            .get_or_insert_with(Default::default)
            .thinking_level = Some(level);
        self
    }

    pub fn with_thinking_summaries(mut self, summaries: ThinkingSummaries) -> Self {
        self.generation_config
            .get_or_insert_with(Default::default)
            .thinking_summaries = Some(summaries);
        self
    }

    pub fn with_presence_penalty(mut self, penalty: f64) -> Self {
        self.generation_config
            .get_or_insert_with(Default::default)
            .presence_penalty = Some(penalty);
        self
    }

    pub fn with_frequency_penalty(mut self, penalty: f64) -> Self {
        self.generation_config
            .get_or_insert_with(Default::default)
            .frequency_penalty = Some(penalty);
        self
    }

    pub fn with_tool_choice(mut self, choice: ToolChoiceConfig) -> Self {
        self.generation_config
            .get_or_insert_with(Default::default)
            .tool_choice = Some(choice);
        self
    }

    pub fn with_speech_config(mut self, config: InteractionSpeechConfig) -> Self {
        self.generation_config
            .get_or_insert_with(Default::default)
            .speech_config
            .push(config);
        self
    }

    pub fn with_video_config(mut self, config: VideoConfig) -> Self {
        self.generation_config
            .get_or_insert_with(Default::default)
            .video_config = Some(config);
        self
    }

    /// Set the full generation config.
    pub fn with_generation_config(mut self, config: InteractionGenerationConfig) -> Self {
        self.generation_config = Some(config);
        self
    }

    // ===== Agent Config =====

    /// Set the agent config.
    pub fn with_agent_config(mut self, config: AgentConfig) -> Self {
        self.agent_config = Some(config);
        self
    }

    // ===== Interaction Options =====

    /// Set `previous_interaction_id` for server-side state management.
    pub fn with_previous_interaction(mut self, id: impl Into<String>) -> Self {
        self.previous_interaction_id = Some(id.into());
        self
    }

    /// Set `store` flag. Use `store(false)` for stateless mode.
    pub fn with_store(mut self, store: bool) -> Self {
        self.store = Some(store);
        self
    }

    /// Enable background execution.
    pub fn with_background(mut self) -> Self {
        self.background = true;
        self
    }

    /// Set the environment configuration.
    pub fn with_environment(mut self, env: EnvironmentConfig) -> Self {
        self.environment = Some(EnvironmentConfigOrString::Config(env));
        self
    }

    /// Set the environment by ID (reuse an existing environment).
    pub fn with_environment_id(mut self, env_id: impl Into<String>) -> Self {
        self.environment = Some(EnvironmentConfigOrString::Id(env_id.into()));
        self
    }

    /// Set cached content.
    pub fn with_cached_content(mut self, cached_content: impl Into<String>) -> Self {
        self.cached_content = Some(cached_content.into());
        self
    }

    /// Set response modalities.
    pub fn with_response_modalities(mut self, modalities: Vec<ResponseModality>) -> Self {
        self.response_modalities = modalities;
        self
    }

    /// Set the service tier.
    pub fn with_service_tier(mut self, tier: ServiceTier) -> Self {
        self.service_tier = Some(tier);
        self
    }

    /// Set the webhook configuration.
    pub fn with_webhook_config(mut self, config: WebhookConfig) -> Self {
        self.webhook_config = Some(config);
        self
    }

    // ===== Build & Execute =====

    /// Build the request.
    pub fn build(self) -> Result<CreateInteractionRequest, ClientError> {
        let input = self.input.ok_or_else(|| ClientError::InvalidResourceName {
            name: "input is required for interaction".to_string(),
        })?;

        let model = if self.model.is_none() && self.agent.is_none() {
            Some(
                self.client
                    .model
                    .as_str()
                    .trim_start_matches("models/")
                    .to_string(),
            )
        } else {
            self.model
        };

        Ok(CreateInteractionRequest {
            model,
            agent: self.agent,
            input,
            system_instruction: self.system_instruction,
            tools: self.tools,
            response_format: self.response_format,
            stream: None,
            store: self.store,
            background: if self.background { Some(true) } else { None },
            generation_config: self.generation_config,
            agent_config: self.agent_config,
            previous_interaction_id: self.previous_interaction_id,
            environment: self.environment,
            cached_content: self.cached_content,
            response_modalities: self.response_modalities,
            service_tier: self.service_tier,
            webhook_config: self.webhook_config,
        })
    }

    /// Execute the interaction (non-streaming).
    #[instrument(skip_all, fields(
        model = self.model.as_deref().unwrap_or(""),
        agent = self.agent.as_deref().unwrap_or(""),
        tools.count = self.tools.len(),
        system.instruction.present = self.system_instruction.is_some(),
        background = self.background,
        previous.interaction.present = self.previous_interaction_id.is_some(),
        status.code,
        usage.total_tokens,
    ))]
    pub async fn execute(self) -> Result<Interaction, ClientError> {
        let client = self.client.clone();
        let request = self.build()?;
        let response = client.create_interaction(request).await?;

        Span::current().record("status.code", response.status.as_ref());

        if let Some(usage) = &response.usage {
            Span::current().record("usage.total_tokens", usage.total_tokens);
        }

        Ok(response)
    }

    /// Execute the interaction (streaming).
    #[instrument(skip_all, fields(
        model = self.model.as_deref().unwrap_or(""),
        agent = self.agent.as_deref().unwrap_or(""),
        tools.count = self.tools.len(),
    ))]
    pub async fn execute_stream(self) -> Result<InteractionStream, ClientError> {
        let client = self.client.clone();
        let request = self.build()?;
        client.create_interaction_stream(request).await
    }
}
