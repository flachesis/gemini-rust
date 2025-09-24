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
    /// The parameters for the function
    pub parameters: FunctionParameter,
}

impl FunctionDeclaration {
    /// Create a new function declaration
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        // Parameters are properties of an object data type, so we use the builder to ensure that
        parameters: FunctionParameterObjectBuilder,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parameters: parameters.build(),
        }
    }
}

/// Function parameter, used in function declarations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FunctionParameter {
    /// Data type of the parameter
    #[serde(rename = "type")]
    pub data_type: String,

    /// Brief description of the parameter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Enum values, only applicable for enums
    #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<String>>,

    /// Array items, only applicable for arrays
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<FunctionParameter>>,

    /// Object properties, only applicable for objects
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, FunctionParameter>>,

    /// Required properties, only applicable for objects
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
}

impl FunctionParameter {
    /// Create a new string parameter
    pub fn string(description: impl Into<String>) -> Self {
        Self {
            data_type: "string".to_string(),
            description: Some(description.into()),
            enum_values: None,
            items: None,
            properties: None,
            required: None,
        }
    }

    /// Create a new number parameter
    pub fn number(description: impl Into<String>) -> Self {
        Self {
            data_type: "number".to_string(),
            description: Some(description.into()),
            enum_values: None,
            items: None,
            properties: None,
            required: None,
        }
    }

    /// Create a new integer parameter
    pub fn integer(description: impl Into<String>) -> Self {
        Self {
            data_type: "integer".to_string(),
            description: Some(description.into()),
            enum_values: None,
            items: None,
            properties: None,
            required: None,
        }
    }

    /// Create a new boolean parameter
    pub fn boolean(description: impl Into<String>) -> Self {
        Self {
            data_type: "boolean".to_string(),
            description: Some(description.into()),
            enum_values: None,
            items: None,
            properties: None,
            required: None,
        }
    }

    /// Create a new array parameter
    pub fn array(description: impl Into<String>, items: FunctionParameter) -> Self {
        Self {
            data_type: "array".to_string(),
            description: Some(description.into()),
            enum_values: None,
            items: Some(Box::new(items)),
            properties: None,
            required: None,
        }
    }

    /// Create a new enum parameter
    pub fn enum_type(
        description: impl Into<String>,
        enum_values: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            data_type: "string".to_string(),
            description: Some(description.into()),
            enum_values: Some(enum_values.into_iter().map(|s| s.into()).collect()),
            items: None,
            properties: None,
            required: None,
        }
    }

    /// Get a `FunctionParameterObjectBuilder` to create an object parameter
    pub fn object_builder() -> FunctionParameterObjectBuilder {
        // The only reason to use a builder is to have a function to add properties to the object
        // that is not available on the other data types.
        FunctionParameterObjectBuilder::new()
    }
}

/// A builder to construct an object `FunctionParameter`.
pub struct FunctionParameterObjectBuilder {
    properties: HashMap<String, FunctionParameter>,
    required: Vec<String>,
}

impl FunctionParameterObjectBuilder {
    /// Create a new, empty object builder.
    pub fn new() -> Self {
        Self {
            properties: HashMap::new(),
            required: Vec::new(),
        }
    }

    // Add a property to the object.
    pub fn with_property(
        mut self,
        name: impl Into<String>,
        function_parameter: FunctionParameter,
        required: bool,
    ) -> Self {
        let name = name.into();
        self.properties.insert(name.clone(), function_parameter);
        if required {
            self.required.push(name);
        }
        self
    }

    /// Build the object `FunctionParameter`.
    pub fn build(self) -> FunctionParameter {
        FunctionParameter {
            data_type: "object".to_string(),
            description: None,
            properties: Some(self.properties),
            required: Some(self.required),
            items: None,
            enum_values: None,
        }
    }
}

impl Default for FunctionParameterObjectBuilder {
    fn default() -> Self {
        Self::new()
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
