use futures::TryStream;
use std::sync::Arc;

use crate::{
    cache::CachedContentHandle,
    client::{Error as ClientError, GeminiClient},
    generation::{GenerateContentRequest, SpeakerVoiceConfig, SpeechConfig, ThinkingConfig},
    tools::{FunctionCallingConfig, ToolConfig},
    Content, FunctionCallingMode, FunctionDeclaration, GenerationConfig, GenerationResponse,
    Message, Role, Tool,
};

/// Builder for content generation requests
pub struct ContentBuilder {
    client: Arc<GeminiClient>,
    pub contents: Vec<Content>,
    generation_config: Option<GenerationConfig>,
    tools: Option<Vec<Tool>>,
    tool_config: Option<ToolConfig>,
    system_instruction: Option<Content>,
    cached_content: Option<String>,
}

impl ContentBuilder {
    /// Create a new content builder
    pub(crate) fn new(client: Arc<GeminiClient>) -> Self {
        Self {
            client,
            contents: Vec::new(),
            generation_config: None,
            tools: None,
            tool_config: None,
            system_instruction: None,
            cached_content: None,
        }
    }

    /// Add a system prompt to the request
    pub fn with_system_prompt(self, text: impl Into<String>) -> Self {
        // Create a Content with text parts specifically for system_instruction field
        self.with_system_instruction(text)
    }

    /// Set the system instruction directly (matching the API format in the curl example)
    pub fn with_system_instruction(mut self, text: impl Into<String>) -> Self {
        // Create a Content with text parts specifically for system_instruction field
        let content = Content::text(text);
        self.system_instruction = Some(content);
        self
    }

    /// Add a user message to the request
    pub fn with_user_message(mut self, text: impl Into<String>) -> Self {
        let message = Message::user(text);
        let content = message.content;
        self.contents.push(content);
        self
    }

    /// Add a model message to the request
    pub fn with_model_message(mut self, text: impl Into<String>) -> Self {
        let message = Message::model(text);
        let content = message.content;
        self.contents.push(content);
        self
    }

    /// Add a inline data (blob data) to the request
    pub fn with_inline_data(
        mut self,
        data: impl Into<String>,
        mime_type: impl Into<String>,
    ) -> Self {
        let content = Content::inline_data(mime_type, data).with_role(Role::User);
        self.contents.push(content);
        self
    }

    /// Add a function response to the request using a JSON value
    pub fn with_function_response(
        mut self,
        name: impl Into<String>,
        response: serde_json::Value,
    ) -> Self {
        let content = Content::function_response_json(name, response).with_role(Role::User);
        self.contents.push(content);
        self
    }

    /// Add a function response to the request using a JSON string
    pub fn with_function_response_str(
        mut self,
        name: impl Into<String>,
        response: impl Into<String>,
    ) -> std::result::Result<Self, serde_json::Error> {
        let response_str = response.into();
        let json = serde_json::from_str(&response_str)?;
        let content = Content::function_response_json(name, json).with_role(Role::User);
        self.contents.push(content);
        Ok(self)
    }

    /// Add a message to the request
    pub fn with_message(mut self, message: Message) -> Self {
        let content = message.content.clone();
        match &content.role {
            Some(role) => {
                let role_clone = role.clone();
                self.contents.push(content.with_role(role_clone));
            }
            None => {
                self.contents.push(content.with_role(message.role));
            }
        }
        self
    }

    /// Use cached content for this request.
    /// This allows reusing previously cached system instructions and conversation history.
    pub fn with_cached_content(mut self, cached_content: &CachedContentHandle) -> Self {
        self.cached_content = Some(cached_content.name().to_string());
        self
    }

    /// Add multiple messages to the request
    pub fn with_messages(mut self, messages: impl IntoIterator<Item = Message>) -> Self {
        for message in messages {
            self = self.with_message(message);
        }
        self
    }

    /// Set the generation config for the request
    pub fn with_generation_config(mut self, config: GenerationConfig) -> Self {
        self.generation_config = Some(config);
        self
    }

    /// Set the temperature for the request
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        if self.generation_config.is_none() {
            self.generation_config = Some(GenerationConfig::default());
        }
        if let Some(config) = &mut self.generation_config {
            config.temperature = Some(temperature);
        }
        self
    }

    /// Set the top-p value for the request
    pub fn with_top_p(mut self, top_p: f32) -> Self {
        if self.generation_config.is_none() {
            self.generation_config = Some(GenerationConfig::default());
        }
        if let Some(config) = &mut self.generation_config {
            config.top_p = Some(top_p);
        }
        self
    }

    /// Set the top-k value for the request
    pub fn with_top_k(mut self, top_k: i32) -> Self {
        if self.generation_config.is_none() {
            self.generation_config = Some(GenerationConfig::default());
        }
        if let Some(config) = &mut self.generation_config {
            config.top_k = Some(top_k);
        }
        self
    }

    /// Set the maximum output tokens for the request
    pub fn with_max_output_tokens(mut self, max_output_tokens: i32) -> Self {
        if self.generation_config.is_none() {
            self.generation_config = Some(GenerationConfig::default());
        }
        if let Some(config) = &mut self.generation_config {
            config.max_output_tokens = Some(max_output_tokens);
        }
        self
    }

    /// Set the candidate count for the request
    pub fn with_candidate_count(mut self, candidate_count: i32) -> Self {
        if self.generation_config.is_none() {
            self.generation_config = Some(GenerationConfig::default());
        }
        if let Some(config) = &mut self.generation_config {
            config.candidate_count = Some(candidate_count);
        }
        self
    }

    /// Set the stop sequences for the request
    pub fn with_stop_sequences(mut self, stop_sequences: Vec<String>) -> Self {
        if self.generation_config.is_none() {
            self.generation_config = Some(GenerationConfig::default());
        }
        if let Some(config) = &mut self.generation_config {
            config.stop_sequences = Some(stop_sequences);
        }
        self
    }

    /// Set the response mime type for the request
    pub fn with_response_mime_type(mut self, mime_type: impl Into<String>) -> Self {
        if self.generation_config.is_none() {
            self.generation_config = Some(GenerationConfig::default());
        }
        if let Some(config) = &mut self.generation_config {
            config.response_mime_type = Some(mime_type.into());
        }
        self
    }

    /// Set the response schema for structured output
    pub fn with_response_schema(mut self, schema: serde_json::Value) -> Self {
        if self.generation_config.is_none() {
            self.generation_config = Some(GenerationConfig::default());
        }
        if let Some(config) = &mut self.generation_config {
            config.response_schema = Some(schema);
        }
        self
    }

    /// Add a tool to the request
    pub fn with_tool(mut self, tool: Tool) -> Self {
        if self.tools.is_none() {
            self.tools = Some(Vec::new());
        }
        if let Some(tools) = &mut self.tools {
            tools.push(tool);
        }
        self
    }

    /// Add a function declaration as a tool
    pub fn with_function(mut self, function: FunctionDeclaration) -> Self {
        let tool = Tool::new(function);
        self = self.with_tool(tool);
        self
    }

    /// Set the function calling mode for the request
    pub fn with_function_calling_mode(mut self, mode: FunctionCallingMode) -> Self {
        if self.tool_config.is_none() {
            self.tool_config = Some(ToolConfig {
                function_calling_config: Some(FunctionCallingConfig { mode }),
            });
        } else if let Some(tool_config) = &mut self.tool_config {
            tool_config.function_calling_config = Some(FunctionCallingConfig { mode });
        }
        self
    }

    /// Set the thinking configuration for the request (Gemini 2.5 series only)
    pub fn with_thinking_config(mut self, thinking_config: ThinkingConfig) -> Self {
        if self.generation_config.is_none() {
            self.generation_config = Some(GenerationConfig::default());
        }
        if let Some(config) = &mut self.generation_config {
            config.thinking_config = Some(thinking_config);
        }
        self
    }

    /// Set the thinking budget for the request (Gemini 2.5 series only)
    pub fn with_thinking_budget(mut self, budget: i32) -> Self {
        if self.generation_config.is_none() {
            self.generation_config = Some(GenerationConfig::default());
        }
        if let Some(config) = &mut self.generation_config {
            if config.thinking_config.is_none() {
                config.thinking_config = Some(ThinkingConfig::default());
            }
            if let Some(thinking_config) = &mut config.thinking_config {
                thinking_config.thinking_budget = Some(budget);
            }
        }
        self
    }

    /// Enable dynamic thinking (model decides the budget) (Gemini 2.5 series only)
    pub fn with_dynamic_thinking(self) -> Self {
        self.with_thinking_budget(-1)
    }

    /// Include thought summaries in the response (Gemini 2.5 series only)
    pub fn with_thoughts_included(mut self, include: bool) -> Self {
        if self.generation_config.is_none() {
            self.generation_config = Some(GenerationConfig::default());
        }
        if let Some(config) = &mut self.generation_config {
            if config.thinking_config.is_none() {
                config.thinking_config = Some(ThinkingConfig::default());
            }
            if let Some(thinking_config) = &mut config.thinking_config {
                thinking_config.include_thoughts = Some(include);
            }
        }
        self
    }

    /// Enable audio output (text-to-speech)
    pub fn with_audio_output(mut self) -> Self {
        if self.generation_config.is_none() {
            self.generation_config = Some(GenerationConfig::default());
        }
        if let Some(config) = &mut self.generation_config {
            config.response_modalities = Some(vec!["AUDIO".to_string()]);
        }
        self
    }

    /// Set speech configuration for text-to-speech generation
    pub fn with_speech_config(mut self, speech_config: SpeechConfig) -> Self {
        if self.generation_config.is_none() {
            self.generation_config = Some(GenerationConfig::default());
        }
        if let Some(config) = &mut self.generation_config {
            config.speech_config = Some(speech_config);
        }
        self
    }

    /// Set a single voice for text-to-speech generation
    pub fn with_voice(self, voice_name: impl Into<String>) -> Self {
        let speech_config = SpeechConfig::single_voice(voice_name);
        self.with_speech_config(speech_config).with_audio_output()
    }

    /// Set multi-speaker configuration for text-to-speech generation
    pub fn with_multi_speaker_config(self, speakers: Vec<SpeakerVoiceConfig>) -> Self {
        let speech_config = SpeechConfig::multi_speaker(speakers);
        self.with_speech_config(speech_config).with_audio_output()
    }

    pub fn build(self) -> GenerateContentRequest {
        GenerateContentRequest {
            contents: self.contents,
            generation_config: self.generation_config,
            safety_settings: None,
            tools: self.tools,
            tool_config: self.tool_config,
            system_instruction: self.system_instruction,
            cached_content: self.cached_content,
        }
    }

    /// Execute the request
    pub async fn execute(self) -> Result<GenerationResponse, ClientError> {
        let client = self.client.clone();
        let request = self.build();
        client.generate_content_raw(request).await
    }

    /// Execute the request with streaming
    pub async fn execute_stream(
        self,
    ) -> Result<impl TryStream<Ok = GenerationResponse, Error = ClientError> + Send, ClientError>
    {
        let request = GenerateContentRequest {
            contents: self.contents,
            generation_config: self.generation_config,
            safety_settings: None,
            tools: self.tools,
            tool_config: self.tool_config,
            system_instruction: self.system_instruction,
            cached_content: self.cached_content,
        };

        self.client.generate_content_stream(request).await
    }
}
