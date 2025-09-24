use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};
use std::collections::HashMap;

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

/// Declaration of a function that can be called by the model
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FunctionDeclaration {
    /// The name of the function
    pub name: String,
    /// The description of the function
    pub description: String,
    /// The parameters for the function (using OpenAPI 3.0 Schema)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Schema>,
    /// JSON Schema format parameters (mutually exclusive with parameters)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters_json_schema: Option<serde_json::Value>,
    /// The response schema (using OpenAPI 3.0 Schema)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<Schema>,
    /// JSON Schema format response (mutually exclusive with response)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_json_schema: Option<serde_json::Value>,
    /// Function behavior (BLOCKING or NON_BLOCKING)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub behavior: Option<FunctionBehavior>,
}

/// Function behavior configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FunctionBehavior {
    /// This value is unused
    Unspecified,
    /// The system will wait to receive the function response before continuing
    Blocking,
    /// The system will not wait to receive the function response
    NonBlocking,
}

impl FunctionDeclaration {
    /// Create a new function declaration
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        parameters: Schema,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parameters: Some(parameters),
            parameters_json_schema: None,
            response: None,
            response_json_schema: None,
            behavior: None,
        }
    }

    /// Create a new function declaration with JSON schema parameters
    pub fn with_json_schema_parameters(
        name: impl Into<String>,
        description: impl Into<String>,
        parameters_json_schema: serde_json::Value,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parameters: None,
            parameters_json_schema: Some(parameters_json_schema),
            response: None,
            response_json_schema: None,
            behavior: None,
        }
    }

    /// Set the response schema
    pub fn with_response(mut self, response: Schema) -> Self {
        self.response = Some(response);
        self
    }

    /// Set the response JSON schema
    pub fn with_response_json_schema(mut self, response_json_schema: serde_json::Value) -> Self {
        self.response_json_schema = Some(response_json_schema);
        self
    }

    /// Set the function behavior
    pub fn with_behavior(mut self, behavior: FunctionBehavior) -> Self {
        self.behavior = Some(behavior);
        self
    }
}

/// Schema data type according to OpenAPI 3.0 specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SchemaType {
    /// Not specified, should not be used
    TypeUnspecified,
    /// String type
    String,
    /// Number type
    Number,
    /// Integer type
    Integer,
    /// Boolean type
    Boolean,
    /// Array type
    Array,
    /// Object type
    Object,
    /// Null type
    Null,
}

/// Schema object following OpenAPI 3.0 specification
/// Represents both function parameters and property details
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Schema {
    /// Required. Data type
    #[serde(rename = "type")]
    pub schema_type: SchemaType,
    /// Optional. The format of the data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    /// Optional. The title of the schema
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Optional. A brief description of the parameter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Optional. Indicates if the value may be null
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nullable: Option<bool>,
    /// Optional. Possible values for enum types
    #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<String>>,
    /// Optional. Schema of array elements
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<Schema>>,
    /// Optional. Maximum number of array elements
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<i64>,
    /// Optional. Minimum number of array elements
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_items: Option<i64>,
    /// Optional. Properties of object type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, Schema>>,
    /// Optional. Required properties of object type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
    /// Optional. Minimum number of object properties
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_properties: Option<i64>,
    /// Optional. Maximum number of object properties
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_properties: Option<i64>,
    /// Optional. Minimum value for integer and number types
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<f64>,
    /// Optional. Maximum value for integer and number types
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<f64>,
    /// Optional. Minimum length for string type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<i64>,
    /// Optional. Maximum length for string type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<i64>,
    /// Optional. Pattern for string type validation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
    /// Optional. Example of the object
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<serde_json::Value>,
    /// Optional. The value should be validated against any of the subschemas
    #[serde(skip_serializing_if = "Option::is_none")]
    pub any_of: Option<Vec<Schema>>,
    /// Optional. Property ordering
    #[serde(skip_serializing_if = "Option::is_none")]
    pub property_ordering: Option<Vec<String>>,
    /// Optional. Default value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
}

impl Schema {
    /// Create a new object schema
    pub fn object() -> Self {
        Self {
            schema_type: SchemaType::Object,
            format: None,
            title: None,
            description: None,
            nullable: None,
            enum_values: None,
            items: None,
            max_items: None,
            min_items: None,
            properties: Some(HashMap::new()),
            required: Some(Vec::new()),
            min_properties: None,
            max_properties: None,
            minimum: None,
            maximum: None,
            min_length: None,
            max_length: None,
            pattern: None,
            example: None,
            any_of: None,
            property_ordering: None,
            default: None,
        }
    }

    /// Create a new string schema
    pub fn string(description: impl Into<String>) -> Self {
        Self {
            schema_type: SchemaType::String,
            format: None,
            title: None,
            description: Some(description.into()),
            nullable: None,
            enum_values: None,
            items: None,
            max_items: None,
            min_items: None,
            properties: None,
            required: None,
            min_properties: None,
            max_properties: None,
            minimum: None,
            maximum: None,
            min_length: None,
            max_length: None,
            pattern: None,
            example: None,
            any_of: None,
            property_ordering: None,
            default: None,
        }
    }

    /// Create a new number schema
    pub fn number(description: impl Into<String>) -> Self {
        Self {
            schema_type: SchemaType::Number,
            format: None,
            title: None,
            description: Some(description.into()),
            nullable: None,
            enum_values: None,
            items: None,
            max_items: None,
            min_items: None,
            properties: None,
            required: None,
            min_properties: None,
            max_properties: None,
            minimum: None,
            maximum: None,
            min_length: None,
            max_length: None,
            pattern: None,
            example: None,
            any_of: None,
            property_ordering: None,
            default: None,
        }
    }

    /// Create a new integer schema
    pub fn integer(description: impl Into<String>) -> Self {
        Self {
            schema_type: SchemaType::Integer,
            format: None,
            title: None,
            description: Some(description.into()),
            nullable: None,
            enum_values: None,
            items: None,
            max_items: None,
            min_items: None,
            properties: None,
            required: None,
            min_properties: None,
            max_properties: None,
            minimum: None,
            maximum: None,
            min_length: None,
            max_length: None,
            pattern: None,
            example: None,
            any_of: None,
            property_ordering: None,
            default: None,
        }
    }

    /// Create a new boolean schema
    pub fn boolean(description: impl Into<String>) -> Self {
        Self {
            schema_type: SchemaType::Boolean,
            format: None,
            title: None,
            description: Some(description.into()),
            nullable: None,
            enum_values: None,
            items: None,
            max_items: None,
            min_items: None,
            properties: None,
            required: None,
            min_properties: None,
            max_properties: None,
            minimum: None,
            maximum: None,
            min_length: None,
            max_length: None,
            pattern: None,
            example: None,
            any_of: None,
            property_ordering: None,
            default: None,
        }
    }

    /// Create a new array schema
    pub fn array(description: impl Into<String>, items: Schema) -> Self {
        Self {
            schema_type: SchemaType::Array,
            format: None,
            title: None,
            description: Some(description.into()),
            nullable: None,
            enum_values: None,
            items: Some(Box::new(items)),
            max_items: None,
            min_items: None,
            properties: None,
            required: None,
            min_properties: None,
            max_properties: None,
            minimum: None,
            maximum: None,
            min_length: None,
            max_length: None,
            pattern: None,
            example: None,
            any_of: None,
            property_ordering: None,
            default: None,
        }
    }

    /// Create a new enum schema
    pub fn enum_type(
        description: impl Into<String>,
        enum_values: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            schema_type: SchemaType::String,
            format: Some("enum".to_string()),
            title: None,
            description: Some(description.into()),
            nullable: None,
            enum_values: Some(enum_values.into_iter().map(|s| s.into()).collect()),
            items: None,
            max_items: None,
            min_items: None,
            properties: None,
            required: None,
            min_properties: None,
            max_properties: None,
            minimum: None,
            maximum: None,
            min_length: None,
            max_length: None,
            pattern: None,
            example: None,
            any_of: None,
            property_ordering: None,
            default: None,
        }
    }

    /// Add a property to an object schema
    pub fn with_property(
        mut self,
        name: impl Into<String>,
        schema: Schema,
        required: bool,
    ) -> Self {
        let name = name.into();
        if let Some(props) = &mut self.properties {
            props.insert(name.clone(), schema);
        }
        if required {
            if let Some(req) = &mut self.required {
                req.push(name);
            }
        }
        self
    }

    /// Set the format field
    pub fn with_format(mut self, format: impl Into<String>) -> Self {
        self.format = Some(format.into());
        self
    }

    /// Set the title field
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set nullable
    pub fn with_nullable(mut self, nullable: bool) -> Self {
        self.nullable = Some(nullable);
        self
    }

    /// Set minimum and maximum for number/integer types
    pub fn with_range(mut self, min: Option<f64>, max: Option<f64>) -> Self {
        self.minimum = min;
        self.maximum = max;
        self
    }

    /// Set min_length and max_length for string types
    pub fn with_length_range(mut self, min_length: Option<i64>, max_length: Option<i64>) -> Self {
        self.min_length = min_length;
        self.max_length = max_length;
        self
    }

    /// Set pattern for string validation
    pub fn with_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.pattern = Some(pattern.into());
        self
    }

    /// Set example value
    pub fn with_example(mut self, example: serde_json::Value) -> Self {
        self.example = Some(example);
        self
    }

    /// Set default value
    pub fn with_default(mut self, default: serde_json::Value) -> Self {
        self.default = Some(default);
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
