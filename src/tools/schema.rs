use schemars::{generate::SchemaSettings, JsonSchema, SchemaGenerator};
use serde::Serialize;
use serde_json::Value;

trait GeminiSchemaSettings {
    /// Create SchemaSettings with Gemini-optimized settings
    /// See: https://ai.google.dev/api/caching#Schema
    fn gemini() -> Self;
}

impl GeminiSchemaSettings for SchemaSettings {
    fn gemini() -> Self {
        SchemaSettings::openapi3().with(|s| {
            s.inline_subschemas = true;
            s.meta_schema = None;
        })
    }
}

pub(super) trait GeminiSchemaGenerator {
    /// Returns JSON Schema for the given response
    fn generate_parameters_schema<Parameters>() -> Value
    where
        Parameters: JsonSchema + Serialize;
}

impl GeminiSchemaGenerator for SchemaGenerator {
    fn generate_parameters_schema<Parameters>() -> Value
    where
        Parameters: JsonSchema + Serialize,
    {
        let schema_generator = SchemaGenerator::new(SchemaSettings::gemini());
        let mut schema = schema_generator.into_root_schema_for::<Parameters>();

        // Root schemas always include a title field, which we don't want or need
        schema.remove("title");

        schema.to_value()
    }
}
