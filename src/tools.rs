use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tool that can be used by the model
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDeclaration {
    /// The name of the function
    pub name: String,
    /// The description of the function
    pub description: String,
    /// The parameters for the function
    pub parameters: FunctionParameters,
}

impl FunctionDeclaration {
    /// Create a new function declaration
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        parameters: FunctionParameters,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parameters,
        }
    }
}

/// Parameters for a function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionParameters {
    /// The type of the parameters
    #[serde(rename = "type")]
    pub param_type: String,
    /// The properties of the parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, PropertyDetails>>,
    /// The required properties
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
}

impl FunctionParameters {
    /// Create a new object parameter set
    pub fn object() -> Self {
        Self {
            param_type: "object".to_string(),
            properties: Some(HashMap::new()),
            required: Some(Vec::new()),
        }
    }

    /// Add a property to the parameters
    pub fn with_property(
        mut self,
        name: impl Into<String>,
        details: PropertyDetails,
        required: bool,
    ) -> Self {
        let name = name.into();
        if let Some(props) = &mut self.properties {
            props.insert(name.clone(), details);
        }
        if required {
            if let Some(req) = &mut self.required {
                req.push(name);
            }
        }
        self
    }
}

/// Details about a property
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyDetails {
    /// The type of the property
    #[serde(rename = "type")]
    pub property_type: String,
    /// The description of the property
    pub description: String,
    /// The enum values if the property is an enum
    #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<String>>,
    /// The items if the property is an array
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<PropertyDetails>>,
}

impl PropertyDetails {
    /// Create a new string property
    pub fn string(description: impl Into<String>) -> Self {
        Self {
            property_type: "string".to_string(),
            description: description.into(),
            enum_values: None,
            items: None,
        }
    }

    /// Create a new number property
    pub fn number(description: impl Into<String>) -> Self {
        Self {
            property_type: "number".to_string(),
            description: description.into(),
            enum_values: None,
            items: None,
        }
    }

    /// Create a new integer property
    pub fn integer(description: impl Into<String>) -> Self {
        Self {
            property_type: "integer".to_string(),
            description: description.into(),
            enum_values: None,
            items: None,
        }
    }

    /// Create a new boolean property
    pub fn boolean(description: impl Into<String>) -> Self {
        Self {
            property_type: "boolean".to_string(),
            description: description.into(),
            enum_values: None,
            items: None,
        }
    }

    /// Create a new array property
    pub fn array(description: impl Into<String>, items: PropertyDetails) -> Self {
        Self {
            property_type: "array".to_string(),
            description: description.into(),
            enum_values: None,
            items: Some(Box::new(items)),
        }
    }

    /// Create a new enum property
    pub fn enum_type(
        description: impl Into<String>,
        enum_values: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            property_type: "string".to_string(),
            description: description.into(),
            enum_values: Some(enum_values.into_iter().map(|s| s.into()).collect()),
            items: None,
        }
    }
}
/// A function call made by the model
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FunctionCall {
    /// The name of the function
    pub name: String,
    /// The arguments for the function
    pub args: serde_json::Value,
}

impl FunctionCall {
    /// Create a new function call
    pub fn new(name: impl Into<String>, args: serde_json::Value) -> Self {
        Self {
            name: name.into(),
            args,
        }
    }

    /// Get a parameter from the arguments
    pub fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> crate::Result<T> {
        match &self.args {
            serde_json::Value::Object(obj) => {
                if let Some(value) = obj.get(key) {
                    serde_json::from_value(value.clone()).map_err(|e| {
                        crate::Error::FunctionCallError(format!(
                            "Error deserializing parameter {key}: {e}"
                        ))
                    })
                } else {
                    Err(crate::Error::FunctionCallError(format!(
                        "Missing parameter: {key}"
                    )))
                }
            }
            _ => Err(crate::Error::FunctionCallError(
                "Arguments are not an object".to_string(),
            )),
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
