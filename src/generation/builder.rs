use std::sync::Arc;
use tracing::instrument;

use crate::{
    cache::CachedContentHandle,
    client::{Error as ClientError, GeminiClient, GenerationStream},
    files::Error as FilesError,
    generation::{
        GenerateContentRequest, ImageConfig, MediaResolutionLevel, SpeakerVoiceConfig,
        SpeechConfig, ThinkingConfig, ThinkingLevel,
    },
    tools::{FunctionCallingConfig, ToolConfig},
    Content, FileHandle, FunctionCallingMode, FunctionDeclaration, GenerationConfig,
    GenerationResponse, Message, Role, SafetySetting, Tool,
};

/// Builder for content generation requests
#[derive(Clone)]
pub struct ContentBuilder {
    client: Arc<GeminiClient>,
    pub contents: Vec<Content>,
    generation_config: Option<GenerationConfig>,
    safety_settings: Option<Vec<SafetySetting>>,
    tools: Option<Vec<Tool>>,
    tool_config: Option<ToolConfig>,
    system_instruction: Option<Content>,
    cached_content: Option<String>,
}

impl ContentBuilder {
    /// Creates a new `ContentBuilder`.
    pub(crate) fn new(client: Arc<GeminiClient>) -> Self {
        Self {
            client,
            contents: Vec::new(),
            generation_config: None,
            safety_settings: None,
            tools: None,
            tool_config: None,
            system_instruction: None,
            cached_content: None,
        }
    }

    /// Sets the safety settings for the request.
    pub fn with_safety_settings(mut self, safety_settings: Vec<SafetySetting>) -> Self {
        self.safety_settings = Some(safety_settings);
        self
    }

    /// Sets the system prompt for the request.
    ///
    /// This is an alias for [`with_system_instruction()`](Self::with_system_instruction).
    pub fn with_system_prompt(self, text: impl Into<String>) -> Self {
        self.with_system_instruction(text)
    }

    /// Sets the system instruction for the request.
    ///
    /// System instructions are used to provide high-level guidance to the model, such as
    /// setting a persona, providing context, or defining the desired output format.
    pub fn with_system_instruction(mut self, text: impl Into<String>) -> Self {
        let content = Content::text(text);
        self.system_instruction = Some(content);
        self
    }

    /// Adds a user message to the conversation history.
    pub fn with_user_message(mut self, text: impl Into<String>) -> Self {
        let message = Message::user(text);
        self.contents.push(message.content);
        self
    }

    /// Adds a user message, together with coordinates for a previously uploaded file.
    ///
    /// Uploading a file and using it avoids encoding large files and sending them, in particular
    /// when this would need to happen more than once with a file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file metadata is incomplete (missing MIME type or URI).
    pub fn with_user_message_and_file(
        mut self,
        text: impl Into<String>,
        file_handle: &FileHandle,
    ) -> Result<Self, FilesError> {
        let content = Content::text_with_file(text, file_handle)?.with_role(Role::User);
        self.contents.push(content);
        Ok(self)
    }

    /// Adds a model message to the conversation history.
    pub fn with_model_message(mut self, text: impl Into<String>) -> Self {
        let message = Message::model(text);
        self.contents.push(message.content);
        self
    }

    /// Adds inline data (e.g., an image) to the request.
    ///
    /// The data should be base64-encoded.
    pub fn with_inline_data(
        mut self,
        data: impl Into<String>,
        mime_type: impl Into<String>,
    ) -> Self {
        let content = Content::inline_data(mime_type, data).with_role(Role::User);
        self.contents.push(content);
        self
    }

    /// Adds inline data with explicit media resolution control.
    ///
    /// This allows fine-grained control over the resolution used for processing
    /// the inline data, which affects both quality and token consumption.
    /// This method is useful for optimizing token usage.
    /// The data should be base64-encoded.
    pub fn with_inline_data_and_resolution(
        mut self,
        data: impl Into<String>,
        mime_type: impl Into<String>,
        resolution: MediaResolutionLevel,
    ) -> Self {
        let content =
            Content::inline_data_with_resolution(mime_type, data, resolution).with_role(Role::User);
        self.contents.push(content);
        self
    }

    /// Adds a function response to the request using a `Serialize` response.
    ///
    /// This is used to provide the model with the result of a function call it has requested.
    pub fn with_function_response<Response>(
        mut self,
        name: impl Into<String>,
        response: Response,
    ) -> std::result::Result<Self, serde_json::Error>
    where
        Response: serde::Serialize,
    {
        let content = Content::function_response_json(name, serde_json::to_value(response)?)
            .with_role(Role::User);
        self.contents.push(content);
        Ok(self)
    }

    /// Adds a function response to the request using a JSON string.
    ///
    /// This is a convenience method that parses the string into a `serde_json::Value`.
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

    /// Adds a `Message` to the conversation history.
    pub fn with_message(mut self, message: Message) -> Self {
        let content = message.content.clone();
        let role = content.role.clone().unwrap_or(message.role);
        self.contents.push(content.with_role(role));
        self
    }

    /// Uses cached content for this request.
    ///
    /// This allows reusing previously cached system instructions and conversation history,
    /// which can reduce latency and cost.
    pub fn with_cached_content(mut self, cached_content: &CachedContentHandle) -> Self {
        self.cached_content = Some(cached_content.name().to_string());
        self
    }

    /// Adds multiple messages to the conversation history.
    pub fn with_messages(mut self, messages: impl IntoIterator<Item = Message>) -> Self {
        for message in messages {
            self = self.with_message(message);
        }
        self
    }

    /// Sets the generation configuration for the request.
    pub fn with_generation_config(mut self, config: GenerationConfig) -> Self {
        self.generation_config = Some(config);
        self
    }

    /// Sets the temperature for the request.
    ///
    /// Temperature controls the randomness of the output. Higher values (e.g., 1.0) produce
    /// more creative results, while lower values (e.g., 0.2) produce more deterministic results.
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.generation_config
            .get_or_insert_with(Default::default)
            .temperature = Some(temperature);
        self
    }

    /// Sets the top-p value for the request.
    ///
    /// Top-p is a sampling method that selects the next token from a cumulative probability
    /// distribution. It can be used to control the diversity of the output.
    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.generation_config
            .get_or_insert_with(Default::default)
            .top_p = Some(top_p);
        self
    }

    /// Sets the top-k value for the request.
    ///
    /// Top-k is a sampling method that selects the next token from the `k` most likely candidates.
    pub fn with_top_k(mut self, top_k: i32) -> Self {
        self.generation_config
            .get_or_insert_with(Default::default)
            .top_k = Some(top_k);
        self
    }

    /// Sets the maximum number of output tokens for the request.
    pub fn with_max_output_tokens(mut self, max_output_tokens: i32) -> Self {
        self.generation_config
            .get_or_insert_with(Default::default)
            .max_output_tokens = Some(max_output_tokens);
        self
    }

    /// Sets the number of candidate responses to generate.
    pub fn with_candidate_count(mut self, candidate_count: i32) -> Self {
        self.generation_config
            .get_or_insert_with(Default::default)
            .candidate_count = Some(candidate_count);
        self
    }

    /// Sets the stop sequences for the request.
    ///
    /// The model will stop generating text when it encounters one of these sequences.
    pub fn with_stop_sequences(mut self, stop_sequences: Vec<String>) -> Self {
        self.generation_config
            .get_or_insert_with(Default::default)
            .stop_sequences = Some(stop_sequences);
        self
    }

    /// Sets the response MIME type for the request.
    ///
    /// This can be used to request structured output, such as JSON.
    pub fn with_response_mime_type(mut self, mime_type: impl Into<String>) -> Self {
        self.generation_config
            .get_or_insert_with(Default::default)
            .response_mime_type = Some(mime_type.into());
        self
    }

    /// Sets the response schema for structured output.
    ///
    /// When used with a JSON MIME type, this schema will be used to validate the model's
    /// output.
    pub fn with_response_schema(mut self, schema: serde_json::Value) -> Self {
        self.generation_config
            .get_or_insert_with(Default::default)
            .response_schema = Some(schema);
        self
    }

    /// Adds a tool to the request.
    ///
    /// Tools allow the model to interact with external systems, such as APIs or databases.
    pub fn with_tool(mut self, tool: Tool) -> Self {
        self.tools.get_or_insert_with(Vec::new).push(tool);
        self
    }

    /// Adds a function declaration as a tool.
    ///
    /// This is a convenience method for creating a `Tool` from a `FunctionDeclaration`.
    pub fn with_function(mut self, function: FunctionDeclaration) -> Self {
        let tool = Tool::new(function);
        self = self.with_tool(tool);
        self
    }

    /// Sets the function calling mode for the request.
    pub fn with_function_calling_mode(mut self, mode: FunctionCallingMode) -> Self {
        self.tool_config
            .get_or_insert_with(Default::default)
            .function_calling_config = Some(FunctionCallingConfig { mode });
        self
    }

    /// Sets the tool configuration for the request.
    pub fn with_tool_config(mut self, tool_config: ToolConfig) -> Self {
        self.tool_config = Some(tool_config);
        self
    }

    /// Sets the thinking configuration for the request (Gemini 2.5 series only).
    pub fn with_thinking_config(mut self, thinking_config: ThinkingConfig) -> Self {
        self.generation_config
            .get_or_insert_with(Default::default)
            .thinking_config = Some(thinking_config);
        self
    }

    /// Sets the thinking budget for the request (Gemini 2.5 series only).
    ///
    /// A budget of -1 enables dynamic thinking.
    /// This is mutually exclusive with `thinking_level` (Gemini 3 models).
    pub fn with_thinking_budget(mut self, budget: i32) -> Self {
        let config = self
            .generation_config
            .get_or_insert_with(Default::default)
            .thinking_config
            .get_or_insert_with(Default::default);
        config.thinking_budget = Some(budget);
        config.thinking_level = None;
        self
    }

    /// Enables dynamic thinking, which allows the model to decide its own thinking budget
    /// (Gemini 2.5 series only).
    ///
    /// Note: This only enables the *capability*. To receive thoughts in the response,
    /// you must also call `[.with_thoughts_included(true)](Self::with_thoughts_included)`.
    pub fn with_dynamic_thinking(self) -> Self {
        self.with_thinking_budget(-1)
    }

    /// Includes thought summaries in the response (Gemini 2.5 series only).
    ///
    /// This requires `with_dynamic_thinking()` or `with_thinking_budget()` to be enabled.
    pub fn with_thoughts_included(mut self, include: bool) -> Self {
        self.generation_config
            .get_or_insert_with(Default::default)
            .thinking_config
            .get_or_insert_with(Default::default)
            .include_thoughts = Some(include);
        self
    }

    /// Sets the thinking level for Gemini 3 Pro.
    ///
    /// This controls the depth of reasoning the model applies. Use `Low` for simpler
    /// queries requiring faster responses, or `High` for complex problems requiring
    /// deeper analysis.
    ///
    /// Note: This is mutually exclusive with `thinking_budget` (used by Gemini 2.5 models).
    /// Setting this will be ignored by Gemini 2.5 models.
    pub fn with_thinking_level(mut self, level: ThinkingLevel) -> Self {
        let config = self
            .generation_config
            .get_or_insert_with(Default::default)
            .thinking_config
            .get_or_insert_with(Default::default);
        config.thinking_level = Some(level);
        config.thinking_budget = None;
        self
    }

    /// Sets the global media resolution level.
    ///
    /// This controls the token usage for all images and PDFs in the request.
    /// Individual parts can override this setting using `with_inline_data_and_resolution()`.
    /// Higher resolutions provide better quality but consume more tokens.
    pub fn with_media_resolution(mut self, level: MediaResolutionLevel) -> Self {
        self.generation_config
            .get_or_insert_with(Default::default)
            .media_resolution = Some(level);
        self
    }

    /// Adds the code execution tool to the request.
    ///
    /// This allows the model to generate and execute Python code as part of the
    /// generation process. Useful for mathematical calculations, data analysis,
    /// and other computational tasks. Currently supports Python only.
    pub fn with_code_execution(self) -> Self {
        self.with_tool(Tool::code_execution())
    }

    /// Enables audio output (text-to-speech).
    pub fn with_audio_output(mut self) -> Self {
        self.generation_config
            .get_or_insert_with(Default::default)
            .response_modalities = Some(vec!["AUDIO".to_string()]);
        self
    }

    /// Sets the image generation configuration.
    pub fn with_image_config(mut self, image_config: ImageConfig) -> Self {
        self.generation_config
            .get_or_insert_with(Default::default)
            .image_config = Some(image_config);
        self
    }

    /// Sets the speech configuration for text-to-speech generation.
    pub fn with_speech_config(mut self, speech_config: SpeechConfig) -> Self {
        self.generation_config
            .get_or_insert_with(Default::default)
            .speech_config = Some(speech_config);
        self
    }

    /// Sets a single voice for text-to-speech generation.
    pub fn with_voice(self, voice_name: impl Into<String>) -> Self {
        let speech_config = SpeechConfig::single_voice(voice_name);
        self.with_speech_config(speech_config).with_audio_output()
    }

    /// Sets multi-speaker configuration for text-to-speech generation.
    pub fn with_multi_speaker_config(self, speakers: Vec<SpeakerVoiceConfig>) -> Self {
        let speech_config = SpeechConfig::multi_speaker(speakers);
        self.with_speech_config(speech_config).with_audio_output()
    }

    /// Builds the `GenerateContentRequest`.
    pub fn build(self) -> GenerateContentRequest {
        GenerateContentRequest {
            contents: self.contents,
            generation_config: self.generation_config,
            safety_settings: self.safety_settings,
            tools: self.tools,
            tool_config: self.tool_config,
            system_instruction: self.system_instruction,
            cached_content: self.cached_content,
        }
    }

    /// Executes the content generation request.
    #[instrument(skip_all, fields(
        messages.parts.count = self.contents.len(),
        tools.present = self.tools.is_some(),
        system.instruction.present = self.system_instruction.is_some(),
        cached.content.present = self.cached_content.is_some(),
    ))]
    pub async fn execute(self) -> Result<GenerationResponse, ClientError> {
        let client = self.client.clone();
        let request = self.build();
        client.generate_content_raw(request).await
    }

    /// Executes the content generation request as a stream.
    #[instrument(skip_all, fields(
        messages.parts.count = self.contents.len(),
        tools.present = self.tools.is_some(),
        system.instruction.present = self.system_instruction.is_some(),
        cached.content.present = self.cached_content.is_some(),
    ))]
    pub async fn execute_stream(self) -> Result<GenerationStream, ClientError> {
        let client = self.client.clone();
        let request = self.build();
        client.generate_content_stream(request).await
    }

    /// Counts the number of tokens in the content generation request.
    #[instrument(skip_all, fields(
        messages.parts.count = self.contents.len(),
    ))]
    pub async fn count_tokens(self) -> Result<super::model::CountTokensResponse, ClientError> {
        let client = self.client.clone();
        let request = self.build();
        client.count_tokens(request).await
    }
}
