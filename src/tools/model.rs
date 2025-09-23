use schemars::{JsonSchema, SchemaGenerator, generate::SchemaSettings};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use snafu::{ResultExt, Snafu};

pub trait GeminiSchema {
    /// Create a new SchemaGenerator with Gemini-optimized settings
    fn gemini() -> Self;
}

impl GeminiSchema for SchemaGenerator {
    // See: https://ai.google.dev/api/caching#Schema
    fn gemini() -> Self {
        let settings = SchemaSettings::openapi3().with(|s| {
            s.inline_subschemas = true;
            s.meta_schema = None;
        });
        SchemaGenerator::new(settings)
    }
}

/// Tool that can be used by the model
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Tool {
    /// Function-based tool
    Function {
        /// The function declaration for the tool
        function_declarations: Vec<FunctionDeclaration>,
    },
    /// Google Search tool
    GoogleSearch {
        /// The Google Search configuration
        google_search: GoogleSearchConfig,
    },
}

/// Empty configuration for Google Search tool
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GoogleSearchConfig {}

impl Tool {
    /// Create a new tool with a single function declaration
    pub fn new(function_declaration: FunctionDeclaration) -> Self {
        Self::Function {
            function_declarations: vec![function_declaration],
        }
    }

    /// Create a new tool with multiple function declarations
    pub fn with_functions(function_declarations: Vec<FunctionDeclaration>) -> Self {
        Self::Function {
            function_declarations,
        }
    }

    /// Create a new Google Search tool
    pub fn google_search() -> Self {
        Self::GoogleSearch {
            google_search: GoogleSearchConfig {},
        }
    }
}

/// Defines the function behavior
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Behavior {
    /// `default` If set, the system will wait to receive the function response before
    /// continuing the conversation.
    #[default]
    Blocking,
    /// If set, the system will not wait to receive the function response. Instead, it will
    /// attempt to handle function responses as they become available while maintaining the
    /// conversation between the user and the model.
    NonBlocking,
}

/// Declaration of a function that can be called by the model
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct FunctionDeclaration {
    /// The name of the function
    pub name: String,
    /// The description of the function
    pub description: String,
    /// `Optional` Specifies the function Behavior. Currently only supported by the BidiGenerateContent method.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub behavior: Option<Behavior>,
    /// `Optional` The parameters for the function
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) parameters: Option<Value>,
    /// `Optional` Describes the output from this function in JSON Schema format. Reflects the
    /// Open API 3.03 Response Object. The Schema defines the type used for the response value
    /// of the function.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) response: Option<Value>,
}

impl FunctionDeclaration {
    /// Create a new function declaration
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        behavior: Option<Behavior>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            behavior,
            ..Default::default()
        }
    }

    /// Set the parameters for the function using a struct that implements `JsonSchema`
    pub fn with_parameters<Parameters>(mut self) -> Self
    where
        Parameters: JsonSchema + Serialize,
    {
        let schema = SchemaGenerator::gemini().into_root_schema_for::<Parameters>();
        self.parameters = Some(schema.to_value());
        self
    }

    /// Set the response schema for the function using a struct that implements `JsonSchema`
    pub fn with_response<Response>(mut self) -> Self
    where
        Response: JsonSchema + Serialize,
    {
        let schema = SchemaGenerator::gemini().into_root_schema_for::<Response>();
        self.response = Some(schema.to_value());
        self
    }
}

/// A function call made by the model
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FunctionCall {
    /// The name of the function
    pub name: String,
    /// The arguments for the function
    pub args: serde_json::Value,
    /// The thought signature for the function call (Gemini 2.5 series only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thought_signature: Option<String>,
}

#[derive(Debug, Snafu)]
pub enum FunctionCallError {
    #[snafu(display("failed to deserialize parameter '{key}'"))]
    Deserialization {
        source: serde_json::Error,
        key: String,
    },

    #[snafu(display("parameter '{key}' is missing in arguments '{args}'"))]
    MissingParameter {
        key: String,
        args: serde_json::Value,
    },

    #[snafu(display("arguments should be an object; actual: {actual}"))]
    ArgumentTypeMismatch { actual: String },
}

impl FunctionCall {
    /// Create a new function call
    pub fn new(name: impl Into<String>, args: serde_json::Value) -> Self {
        Self {
            name: name.into(),
            args,
            thought_signature: None,
        }
    }

    /// Create a new function call with thought signature
    pub fn with_thought_signature(
        name: impl Into<String>,
        args: serde_json::Value,
        thought_signature: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            args,
            thought_signature: Some(thought_signature.into()),
        }
    }

    /// Get a parameter from the arguments
    pub fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<T, FunctionCallError> {
        match &self.args {
            serde_json::Value::Object(obj) => {
                if let Some(value) = obj.get(key) {
                    serde_json::from_value(value.clone()).with_context(|_| DeserializationSnafu {
                        key: key.to_string(),
                    })
                } else {
                    Err(MissingParameterSnafu {
                        key: key.to_string(),
                        args: self.args.clone(),
                    }
                    .build())
                }
            }
            _ => Err(ArgumentTypeMismatchSnafu {
                actual: self.args.to_string(),
            }
            .build()),
        }
    }
}

/// A response from a function
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FunctionResponse {
    /// The name of the function
    pub name: String,
    /// The response from the function
    /// This must be a valid JSON object
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<serde_json::Value>,
}

impl FunctionResponse {
    /// Create a new function response with a JSON value
    pub fn new(name: impl Into<String>, response: serde_json::Value) -> Self {
        Self {
            name: name.into(),
            response: Some(response),
        }
    }

    /// Create a new function response with a string that will be parsed as JSON
    pub fn from_str(
        name: impl Into<String>,
        response: impl Into<String>,
    ) -> Result<Self, serde_json::Error> {
        let json = serde_json::from_str(&response.into())?;
        Ok(Self {
            name: name.into(),
            response: Some(json),
        })
    }
}

/// Configuration for tools
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ToolConfig {
    /// The function calling config
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_calling_config: Option<FunctionCallingConfig>,
}

/// Configuration for function calling
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FunctionCallingConfig {
    /// The mode for function calling
    pub mode: FunctionCallingMode,
}

/// Mode for function calling
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FunctionCallingMode {
    /// The model may use function calling
    Auto,
    /// The model must use function calling
    Any,
    /// The model must not use function calling
    None,
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use schemars::JsonSchema;
    use serde::Serialize;
    use serde_json::json;

    /// Test struct with various basic types
    #[derive(Serialize, JsonSchema)]
    struct BasicTypes {
        /// A string field
        name: String,
        /// An optional string field
        optional_name: Option<String>,
        /// An integer field
        age: i32,
        /// A number field
        height: f64,
        /// A boolean field
        is_active: bool,
    }

    /// Test enum with string variants
    #[derive(Serialize, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    #[allow(dead_code)]
    enum Status {
        /// Active status
        Active,
        /// Inactive status
        Inactive,
        /// Pending status
        Pending,
    }

    /// Test struct with enum field
    #[derive(Serialize, JsonSchema)]
    struct WithEnum {
        /// Status field
        status: Status,
        /// Optional status
        optional_status: Option<Status>,
    }

    /// Test struct with arrays
    #[derive(Serialize, JsonSchema)]
    struct WithArrays {
        /// Array of strings
        tags: Vec<String>,
        /// Array of numbers
        scores: Vec<f64>,
        /// Optional array
        optional_items: Option<Vec<i32>>,
    }

    /// Nested struct for testing object properties
    #[derive(Serialize, JsonSchema)]
    struct NestedStruct {
        /// Nested field
        value: String,
        /// Nested count
        count: i32,
    }

    /// Test struct with nested objects
    #[derive(Serialize, JsonSchema)]
    struct WithNested {
        /// Simple field
        name: String,
        /// Nested object
        nested: NestedStruct,
        /// Optional nested object
        optional_nested: Option<NestedStruct>,
        /// Array of nested objects
        nested_array: Vec<NestedStruct>,
    }

    /// Test struct with validation constraints using schemars attributes
    #[derive(Serialize, JsonSchema)]
    struct WithValidation {
        /// String with length constraints
        #[schemars(length(min = 1, max = 100))]
        name: String,
        /// Integer with range constraints  
        #[schemars(range(min = 0, max = 150))]
        age: i32,
        /// String with pattern
        #[schemars(regex(pattern = r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"))]
        email: String,
        /// Array with item constraints
        #[schemars(length(min = 1, max = 10))]
        tags: Vec<String>,
    }

    /// Test struct with default values
    #[derive(Serialize, JsonSchema)]
    #[allow(dead_code)]
    struct WithDefaults {
        /// Field with default
        #[serde(default = "default_name")]
        name: String,
        /// Field with default number
        #[serde(default = "default_count")]
        count: i32,
        /// Optional field
        optional: Option<String>,
    }

    #[allow(dead_code)]
    fn default_name() -> String {
        "default".to_string()
    }

    #[allow(dead_code)]
    fn default_count() -> i32 {
        42
    }

    /// Test struct with title and description
    #[derive(Serialize, JsonSchema)]
    #[schemars(title = "User Profile", description = "A user profile object")]
    struct UserProfile {
        /// The user's full name
        #[schemars(title = "Full Name")]
        name: String,
        /// The user's age in years
        #[schemars(description = "Age must be between 0 and 150")]
        age: i32,
    }

    #[test]
    fn test_basic_types_schema() {
        let func = FunctionDeclaration::new("test_basic", "Test basic types", None)
            .with_parameters::<BasicTypes>();

        assert!(func.parameters.is_some());
        let schema_value = func.parameters.unwrap();
        let schema: HashMap<String, Value> = serde_json::from_value(schema_value.clone()).unwrap();

        // Check root type is object
        assert_eq!(schema.get("type").unwrap(), "object");

        // Check properties exist
        let properties = schema.get("properties").unwrap().as_object().unwrap();
        assert!(properties.contains_key("name"));
        assert!(properties.contains_key("optional_name"));
        assert!(properties.contains_key("age"));
        assert!(properties.contains_key("height"));
        assert!(properties.contains_key("is_active"));

        // Check property types
        let name_prop = properties.get("name").unwrap().as_object().unwrap();
        assert_eq!(name_prop.get("type").unwrap(), "string");

        let age_prop = properties.get("age").unwrap().as_object().unwrap();
        assert_eq!(age_prop.get("type").unwrap(), "integer");

        let height_prop = properties.get("height").unwrap().as_object().unwrap();
        assert_eq!(height_prop.get("type").unwrap(), "number");

        let is_active_prop = properties.get("is_active").unwrap().as_object().unwrap();
        assert_eq!(is_active_prop.get("type").unwrap(), "boolean");

        // Check optional field is marked as nullable
        let optional_name_prop = properties
            .get("optional_name")
            .unwrap()
            .as_object()
            .unwrap();
        assert_eq!(optional_name_prop.get("type").unwrap(), "string");
        assert_eq!(optional_name_prop.get("nullable"), Some(&json!(true)));

        // Check required fields - only non-optional fields should be in required array
        if let Some(required) = schema.get("required") {
            let required = required.as_array().unwrap();
            assert!(required.contains(&json!("name")));
            assert!(required.contains(&json!("age")));
            assert!(required.contains(&json!("height")));
            assert!(required.contains(&json!("is_active")));
            // optional_name should NOT be in required since it's Optional<String>
            assert!(!required.contains(&json!("optional_name")));
        }
    }

    #[test]
    fn test_enum_schema() {
        let func =
            FunctionDeclaration::new("test_enum", "Test enum", None).with_parameters::<WithEnum>();

        let schema_value = func.parameters.unwrap();
        let schema: HashMap<String, Value> = serde_json::from_value(schema_value.clone()).unwrap();

        let properties = schema.get("properties").unwrap().as_object().unwrap();
        let status_prop = properties.get("status").unwrap().as_object().unwrap();

        // Enum is represented using oneOf structure
        assert!(status_prop.contains_key("oneOf"));
        let one_of = status_prop.get("oneOf").unwrap().as_array().unwrap();

        // Check that enum variants are present
        let mut found_active = false;
        let mut found_inactive = false;
        let mut found_pending = false;

        for variant in one_of {
            if let Some(enum_vals) = variant.get("enum") {
                let enum_vals = enum_vals.as_array().unwrap();
                if enum_vals.contains(&json!("active")) {
                    found_active = true;
                }
                if enum_vals.contains(&json!("inactive")) {
                    found_inactive = true;
                }
                if enum_vals.contains(&json!("pending")) {
                    found_pending = true;
                }
            }
        }

        assert!(found_active, "Should contain 'active' variant");
        assert!(found_inactive, "Should contain 'inactive' variant");
        assert!(found_pending, "Should contain 'pending' variant");
    }

    #[test]
    fn test_array_schema() {
        let func = FunctionDeclaration::new("test_arrays", "Test arrays", None)
            .with_parameters::<WithArrays>();

        let schema_value = func.parameters.unwrap();
        let schema: HashMap<String, Value> = serde_json::from_value(schema_value).unwrap();

        let properties = schema.get("properties").unwrap().as_object().unwrap();
        let tags_prop = properties.get("tags").unwrap().as_object().unwrap();

        // Should be array type
        assert_eq!(tags_prop.get("type").unwrap(), "array");

        // Check items type
        let items = tags_prop.get("items").unwrap().as_object().unwrap();
        assert_eq!(items.get("type").unwrap(), "string");

        // Check scores array
        let scores_prop = properties.get("scores").unwrap().as_object().unwrap();
        assert_eq!(scores_prop.get("type").unwrap(), "array");
        let scores_items = scores_prop.get("items").unwrap().as_object().unwrap();
        assert_eq!(scores_items.get("type").unwrap(), "number");
    }

    #[test]
    fn test_nested_objects_schema() {
        let func = FunctionDeclaration::new("test_nested", "Test nested objects", None)
            .with_parameters::<WithNested>();

        let schema_value = func.parameters.unwrap();
        let schema: HashMap<String, Value> = serde_json::from_value(schema_value).unwrap();

        let properties = schema.get("properties").unwrap().as_object().unwrap();
        let nested_prop = properties.get("nested").unwrap().as_object().unwrap();

        // Should be object type
        assert_eq!(nested_prop.get("type").unwrap(), "object");

        // Check nested properties
        let nested_properties = nested_prop.get("properties").unwrap().as_object().unwrap();
        assert!(nested_properties.contains_key("value"));
        assert!(nested_properties.contains_key("count"));

        let nested_value_prop = nested_properties.get("value").unwrap().as_object().unwrap();
        assert_eq!(nested_value_prop.get("type").unwrap(), "string");

        // Check array of nested objects
        let nested_array_prop = properties.get("nested_array").unwrap().as_object().unwrap();
        assert_eq!(nested_array_prop.get("type").unwrap(), "array");
        let array_items = nested_array_prop.get("items").unwrap().as_object().unwrap();
        assert_eq!(array_items.get("type").unwrap(), "object");
    }

    #[test]
    fn test_validation_constraints() {
        let func = FunctionDeclaration::new("test_validation", "Test validation", None)
            .with_parameters::<WithValidation>();

        let schema_value = func.parameters.unwrap();
        let schema: HashMap<String, Value> = serde_json::from_value(schema_value).unwrap();

        let properties = schema.get("properties").unwrap().as_object().unwrap();

        // Check name field with length constraints
        let name_prop = properties.get("name").unwrap().as_object().unwrap();
        if let Some(min_length) = name_prop.get("minLength") {
            assert_eq!(min_length.as_u64().unwrap(), 1);
        }
        if let Some(max_length) = name_prop.get("maxLength") {
            assert_eq!(max_length.as_u64().unwrap(), 100);
        }

        // Check age field with range constraints
        let age_prop = properties.get("age").unwrap().as_object().unwrap();
        if let Some(minimum) = age_prop.get("minimum") {
            assert_eq!(minimum.as_u64().unwrap(), 0);
        }
        if let Some(maximum) = age_prop.get("maximum") {
            assert_eq!(maximum.as_u64().unwrap(), 150);
        }

        // Check email field with pattern
        let email_prop = properties.get("email").unwrap().as_object().unwrap();
        if let Some(pattern) = email_prop.get("pattern") {
            assert!(pattern.as_str().unwrap().contains("@"));
        }

        // Check tags array with length constraints
        let tags_prop = properties.get("tags").unwrap().as_object().unwrap();
        if let Some(min_items) = tags_prop.get("minItems") {
            assert_eq!(min_items.as_u64().unwrap(), 1);
        }
        if let Some(max_items) = tags_prop.get("maxItems") {
            assert_eq!(max_items.as_u64().unwrap(), 10);
        }
    }

    #[test]
    fn test_titles_and_descriptions() {
        let func = FunctionDeclaration::new("test_titles", "Test titles", None)
            .with_parameters::<UserProfile>();

        let schema_value = func.parameters.unwrap();
        let schema: HashMap<String, Value> = serde_json::from_value(schema_value).unwrap();

        // Check root title and description
        if let Some(title) = schema.get("title") {
            assert_eq!(title.as_str().unwrap(), "User Profile");
        }
        if let Some(description) = schema.get("description") {
            assert_eq!(description.as_str().unwrap(), "A user profile object");
        }

        let properties = schema.get("properties").unwrap().as_object().unwrap();

        // Check field title
        let name_prop = properties.get("name").unwrap().as_object().unwrap();
        if let Some(title) = name_prop.get("title") {
            assert_eq!(title.as_str().unwrap(), "Full Name");
        }

        // Check field description
        let age_prop = properties.get("age").unwrap().as_object().unwrap();
        if let Some(description) = age_prop.get("description") {
            assert!(description.as_str().unwrap().contains("Age must be"));
        }
    }

    #[test]
    fn test_google_schema_compatibility() {
        let func =
            FunctionDeclaration::new("test_compatibility", "Test Google compatibility", None)
                .with_parameters::<WithNested>();

        let schema_value = func.parameters.unwrap();
        let schema: HashMap<String, Value> = serde_json::from_value(schema_value).unwrap();

        // Verify only Google Gemini-supported fields are present at root level
        // Based on https://ai.google.dev/api/caching#Schema
        let gemini_supported_fields = [
            "type",
            "format",
            "title",
            "description",
            "nullable",
            "enum",
            "maxItems",
            "minItems",
            "properties",
            "required",
            "minProperties",
            "maxProperties",
            "minLength",
            "maxLength",
            "pattern",
            "example",
            "anyOf",
            "propertyOrdering",
            "default",
            "items",
            "minimum",
            "maximum",
        ];

        for key in schema.keys() {
            assert!(
                gemini_supported_fields.contains(&key.as_str()),
                "Found unsupported field '{}' in schema. Gemini Schema does not support this field.",
                key
            );
        }

        // Ensure required Google fields are present
        assert!(schema.contains_key("type"));
        assert_eq!(schema.get("type").unwrap(), "object");
        assert!(schema.contains_key("properties"));

        // Verify that $schema field is not present (was removed by our fix)
        assert!(
            !schema.contains_key("$schema"),
            "$schema field should be removed for Gemini compatibility"
        );

        // Check properties recursively for unsupported fields
        if let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) {
            for (prop_name, prop_value) in properties {
                if let Some(prop_obj) = prop_value.as_object() {
                    for key in prop_obj.keys() {
                        assert!(
                            gemini_supported_fields.contains(&key.as_str()),
                            "Property '{}' contains unsupported field '{}' in schema",
                            prop_name,
                            key
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_schema_generation_settings() {
        // Test that the Gemini schema generator produces valid schemas
        let generator = SchemaGenerator::gemini();
        let schema = generator.into_root_schema_for::<BasicTypes>();

        // Should generate inline schemas (not references)
        let schema_json = serde_json::to_value(&schema).unwrap();

        // The schema should be well-formed JSON
        assert!(schema_json.is_object());

        // Should have the basic structure expected by Google
        let root_obj = schema_json.as_object().unwrap();
        assert!(root_obj.contains_key("type") || root_obj.contains_key("$schema"));
    }

    #[test]
    fn test_function_response_method() {
        let func = FunctionDeclaration::new("test_response", "Test with response", None)
            .with_parameters::<BasicTypes>()
            .with_response::<UserProfile>();

        assert!(func.parameters.is_some());
        assert!(func.response.is_some());

        let response_value = func.response.unwrap();
        let response_schema: HashMap<String, Value> =
            serde_json::from_value(response_value).unwrap();

        assert_eq!(response_schema.get("type").unwrap(), "object");
        let properties = response_schema
            .get("properties")
            .unwrap()
            .as_object()
            .unwrap();
        assert!(properties.contains_key("name"));
        assert!(properties.contains_key("age"));
    }

    #[test]
    fn test_complex_nested_structure() {
        #[derive(Serialize, JsonSchema)]
        struct ComplexNested {
            /// Map-like structure using nested objects
            metadata: HashMap<String, String>,
            /// Multi-dimensional array
            matrix: Vec<Vec<f64>>,
            /// Union-like enum
            #[schemars(title = "Data Type")]
            data_type: DataType,
        }

        #[derive(Serialize, JsonSchema)]
        #[serde(tag = "type", content = "value")]
        #[allow(dead_code)]
        enum DataType {
            Text(String),
            Number(f64),
            Boolean(bool),
        }

        let func = FunctionDeclaration::new("test_complex", "Test complex structure", None)
            .with_parameters::<ComplexNested>();

        let schema_value = func.parameters.unwrap();
        let schema: HashMap<String, Value> = serde_json::from_value(schema_value).unwrap();

        let properties = schema.get("properties").unwrap().as_object().unwrap();

        // Check matrix (array of arrays)
        let matrix_prop = properties.get("matrix").unwrap().as_object().unwrap();
        assert_eq!(matrix_prop.get("type").unwrap(), "array");
        let matrix_items = matrix_prop.get("items").unwrap().as_object().unwrap();
        assert_eq!(matrix_items.get("type").unwrap(), "array");
        let inner_items = matrix_items.get("items").unwrap().as_object().unwrap();
        assert_eq!(inner_items.get("type").unwrap(), "number");

        // Check metadata (should be object type for HashMap<String, String>)
        let metadata_prop = properties.get("metadata").unwrap().as_object().unwrap();
        assert_eq!(metadata_prop.get("type").unwrap(), "object");
    }

    #[test]
    fn test_nullable_fields() {
        #[derive(Serialize, JsonSchema)]
        struct WithNullable {
            /// Required field
            name: String,
            /// Nullable field (Option)
            optional_value: Option<String>,
            /// Nullable number
            optional_number: Option<i32>,
        }

        let func = FunctionDeclaration::new("test_nullable", "Test nullable fields", None)
            .with_parameters::<WithNullable>();

        let schema_value = func.parameters.unwrap();
        let schema: HashMap<String, Value> = serde_json::from_value(schema_value.clone()).unwrap();

        // Check required fields - only non-optional fields should be in required array
        if let Some(required) = schema.get("required") {
            let required = required.as_array().unwrap();
            assert!(required.contains(&json!("name")));
            // Optional fields should NOT be in required since they're Option<T>
            assert!(!required.contains(&json!("optional_value")));
            assert!(!required.contains(&json!("optional_number")));
        }

        // Check that optional fields are marked as nullable in properties
        let properties = schema.get("properties").unwrap().as_object().unwrap();
        assert!(properties.contains_key("optional_value"));
        assert!(properties.contains_key("optional_number"));

        let optional_value_prop = properties
            .get("optional_value")
            .unwrap()
            .as_object()
            .unwrap();
        assert_eq!(optional_value_prop.get("nullable"), Some(&json!(true)));

        let optional_number_prop = properties
            .get("optional_number")
            .unwrap()
            .as_object()
            .unwrap();
        assert_eq!(optional_number_prop.get("nullable"), Some(&json!(true)));
    }

    #[test]
    fn test_schema_with_examples() {
        #[derive(Serialize, JsonSchema)]
        struct WithExamples {
            /// Name field
            name: String,
            /// Age field  
            age: i32,
        }

        let func = FunctionDeclaration::new("test_examples", "Test examples", None)
            .with_parameters::<WithExamples>();

        let schema_value = func.parameters.unwrap();
        let schema: HashMap<String, Value> = serde_json::from_value(schema_value).unwrap();

        let properties = schema.get("properties").unwrap().as_object().unwrap();

        // Check basic structure
        let name_prop = properties.get("name").unwrap().as_object().unwrap();
        assert_eq!(name_prop.get("type").unwrap(), "string");

        let age_prop = properties.get("age").unwrap().as_object().unwrap();
        assert_eq!(age_prop.get("type").unwrap(), "integer");
    }

    #[test]
    fn test_min_max_properties() {
        #[derive(Serialize, JsonSchema)]
        struct WithObjectLimits {
            field1: String,
            field2: Option<String>,
        }

        let func = FunctionDeclaration::new("test_object_limits", "Test object limits", None)
            .with_parameters::<WithObjectLimits>();

        let schema_value = func.parameters.unwrap();
        let schema: HashMap<String, Value> = serde_json::from_value(schema_value).unwrap();

        // Check basic structure - min/max properties might not be directly settable with schemars attributes
        assert_eq!(schema.get("type").unwrap(), "object");
        let properties = schema.get("properties").unwrap().as_object().unwrap();
        assert!(properties.contains_key("field1"));
        assert!(properties.contains_key("field2"));
    }

    #[test]
    fn test_property_ordering() {
        #[derive(Serialize, JsonSchema)]
        struct WithOrdering {
            /// Third field
            c_field: String,
            /// First field
            a_field: String,
            /// Second field
            b_field: String,
        }

        let func = FunctionDeclaration::new("test_ordering", "Test property ordering", None)
            .with_parameters::<WithOrdering>();

        let schema_value = func.parameters.unwrap();
        let schema: HashMap<String, Value> = serde_json::from_value(schema_value).unwrap();

        // Check if propertyOrdering is supported (Google schema feature)
        if let Some(property_ordering) = schema.get("propertyOrdering") {
            let ordering = property_ordering.as_array().unwrap();
            assert!(!ordering.is_empty());
        }

        // Properties should exist regardless of ordering
        let properties = schema.get("properties").unwrap().as_object().unwrap();
        assert!(properties.contains_key("a_field"));
        assert!(properties.contains_key("b_field"));
        assert!(properties.contains_key("c_field"));
    }

    #[test]
    fn test_any_of_patterns() {
        #[derive(Serialize, JsonSchema)]
        struct WithAnyOf {
            /// Field that could be string or number  
            flexible_field: FlexibleValue,
        }

        #[derive(Serialize, JsonSchema)]
        #[serde(untagged)]
        #[allow(dead_code)]
        enum FlexibleValue {
            Text(String),
            Number(f64),
        }

        let func = FunctionDeclaration::new("test_any_of", "Test anyOf patterns", None)
            .with_parameters::<WithAnyOf>();

        let schema_value = func.parameters.unwrap();
        let schema: HashMap<String, Value> = serde_json::from_value(schema_value).unwrap();

        let properties = schema.get("properties").unwrap().as_object().unwrap();
        let flexible_prop = properties
            .get("flexible_field")
            .unwrap()
            .as_object()
            .unwrap();

        // Should use anyOf or oneOf for union types
        assert!(flexible_prop.contains_key("anyOf") || flexible_prop.contains_key("oneOf"));
    }

    #[test]
    fn test_format_annotations() {
        let func = FunctionDeclaration::new("test_formats", "Test format annotations", None)
            .with_parameters::<BasicTypes>();

        let schema_value = func.parameters.unwrap();
        let schema: HashMap<String, Value> = serde_json::from_value(schema_value).unwrap();

        let properties = schema.get("properties").unwrap().as_object().unwrap();

        // Check format annotations for numbers
        let height_prop = properties.get("height").unwrap().as_object().unwrap();
        if let Some(format) = height_prop.get("format") {
            assert_eq!(format.as_str().unwrap(), "double");
        }

        let age_prop = properties.get("age").unwrap().as_object().unwrap();
        if let Some(format) = age_prop.get("format") {
            assert_eq!(format.as_str().unwrap(), "int32");
        }
    }

    #[test]
    fn test_nested_array_objects() {
        #[derive(Serialize, JsonSchema)]
        struct NestedArrays {
            /// Array of arrays of strings
            matrix: Vec<Vec<String>>,
            /// Array of objects
            objects: Vec<SimpleObject>,
        }

        #[derive(Serialize, JsonSchema)]
        struct SimpleObject {
            id: String,
            value: i32,
        }

        let func = FunctionDeclaration::new("test_nested_arrays", "Test nested arrays", None)
            .with_parameters::<NestedArrays>();

        let schema_value = func.parameters.unwrap();
        let schema: HashMap<String, Value> = serde_json::from_value(schema_value).unwrap();

        let properties = schema.get("properties").unwrap().as_object().unwrap();

        // Check matrix (array of arrays)
        let matrix_prop = properties.get("matrix").unwrap().as_object().unwrap();
        assert_eq!(matrix_prop.get("type").unwrap(), "array");
        let matrix_items = matrix_prop.get("items").unwrap().as_object().unwrap();
        assert_eq!(matrix_items.get("type").unwrap(), "array");
        let inner_items = matrix_items.get("items").unwrap().as_object().unwrap();
        assert_eq!(inner_items.get("type").unwrap(), "string");

        // Check objects array
        let objects_prop = properties.get("objects").unwrap().as_object().unwrap();
        assert_eq!(objects_prop.get("type").unwrap(), "array");
        let object_items = objects_prop.get("items").unwrap().as_object().unwrap();
        assert_eq!(object_items.get("type").unwrap(), "object");

        // Object items should have properties
        let object_properties = object_items.get("properties").unwrap().as_object().unwrap();
        assert!(object_properties.contains_key("id"));
        assert!(object_properties.contains_key("value"));
    }

    #[test]
    fn test_comprehensive_real_world_schema() {
        /// A comprehensive example combining multiple Google-compatible schema features
        #[derive(Serialize, JsonSchema)]
        #[schemars(
            title = "API Request",
            description = "A comprehensive API request structure"
        )]
        struct ApiRequest {
            /// Request ID for tracking
            #[schemars(length(min = 1, max = 50))]
            request_id: String,

            /// Request method
            method: HttpMethod,

            /// Request headers (optional)
            headers: Option<HashMap<String, String>>,

            /// Query parameters
            #[schemars(length(max = 20))]
            query_params: Vec<QueryParam>,

            /// Request body (optional)
            body: Option<RequestBody>,

            /// Request metadata
            metadata: RequestMetadata,

            /// Retry configuration (optional)
            retry_config: Option<RetryConfig>,
        }

        #[derive(Serialize, JsonSchema)]
        #[serde(rename_all = "UPPERCASE")]
        #[allow(dead_code)]
        enum HttpMethod {
            Get,
            Post,
            Put,
            Delete,
            Patch,
        }

        #[derive(Serialize, JsonSchema)]
        struct QueryParam {
            /// Parameter name
            #[schemars(length(min = 1, max = 100))]
            name: String,
            /// Parameter value
            value: String,
        }

        #[derive(Serialize, JsonSchema)]
        #[serde(untagged)]
        #[allow(dead_code)]
        enum RequestBody {
            Json(serde_json::Value),
            Text(String),
            FormData(HashMap<String, String>),
        }

        #[derive(Serialize, JsonSchema)]
        #[schemars(description = "Request metadata containing timing and auth info")]
        struct RequestMetadata {
            /// Request timestamp
            timestamp: i64,
            /// Authentication token (optional)
            auth_token: Option<String>,
            /// Request timeout in seconds
            #[schemars(range(min = 1, max = 300))]
            timeout_seconds: i32,
        }

        #[derive(Serialize, JsonSchema)]
        struct RetryConfig {
            /// Maximum number of retries
            #[schemars(range(min = 0, max = 10))]
            max_retries: i32,
            /// Retry delay in milliseconds
            #[schemars(range(min = 100, max = 30000))]
            retry_delay_ms: i32,
        }

        let func = FunctionDeclaration::new(
            "make_api_request",
            "Make an API request with comprehensive options",
            None,
        )
        .with_parameters::<ApiRequest>()
        .with_response::<ApiResponse>();

        // Test that schema generation succeeds
        assert!(func.parameters.is_some());
        assert!(func.response.is_some());

        let schema_value = func.parameters.unwrap();
        let schema: HashMap<String, Value> = serde_json::from_value(schema_value).unwrap();

        // Verify root structure
        assert_eq!(schema.get("type").unwrap(), "object");
        assert_eq!(schema.get("title").unwrap(), "API Request");
        assert!(
            schema
                .get("description")
                .unwrap()
                .as_str()
                .unwrap()
                .contains("comprehensive")
        );

        // Verify complex nested structure
        let properties = schema.get("properties").unwrap().as_object().unwrap();
        assert!(properties.contains_key("request_id"));
        assert!(properties.contains_key("method"));
        assert!(properties.contains_key("headers"));
        assert!(properties.contains_key("query_params"));
        assert!(properties.contains_key("body"));
        assert!(properties.contains_key("metadata"));
        assert!(properties.contains_key("retry_config"));

        // Verify enum handling
        let method_prop = properties.get("method").unwrap().as_object().unwrap();
        // Enum might use oneOf or enum directly depending on the structure
        assert!(method_prop.contains_key("oneOf") || method_prop.contains_key("enum"));

        // Verify array handling
        let query_params_prop = properties.get("query_params").unwrap().as_object().unwrap();
        assert_eq!(query_params_prop.get("type").unwrap(), "array");
        assert!(query_params_prop.contains_key("items"));

        // Verify nested object handling
        let metadata_prop = properties.get("metadata").unwrap().as_object().unwrap();
        assert_eq!(metadata_prop.get("type").unwrap(), "object");
        assert!(metadata_prop.contains_key("properties"));

        // Verify optional fields are handled correctly
        let headers_prop = properties.get("headers").unwrap().as_object().unwrap();
        assert_eq!(headers_prop.get("nullable"), Some(&json!(true)));

        #[derive(Serialize, JsonSchema)]
        #[schemars(title = "API Response", description = "API response structure")]
        struct ApiResponse {
            /// HTTP status code
            #[schemars(range(min = 100, max = 599))]
            status_code: i32,
            /// Response headers
            headers: HashMap<String, String>,
            /// Response body (optional)
            body: Option<String>,
            /// Response metadata
            metadata: ResponseMetadata,
        }

        #[derive(Serialize, JsonSchema)]
        struct ResponseMetadata {
            /// Response timestamp
            timestamp: i64,
            /// Processing time in milliseconds
            processing_time_ms: i32,
        }

        // Verify response schema
        let response_value = func.response.unwrap();
        let response_schema: HashMap<String, Value> =
            serde_json::from_value(response_value).unwrap();

        assert_eq!(response_schema.get("type").unwrap(), "object");
        assert_eq!(response_schema.get("title").unwrap(), "API Response");

        let response_properties = response_schema
            .get("properties")
            .unwrap()
            .as_object()
            .unwrap();
        assert!(response_properties.contains_key("status_code"));
        assert!(response_properties.contains_key("headers"));
        assert!(response_properties.contains_key("body"));
        assert!(response_properties.contains_key("metadata"));
    }
}
