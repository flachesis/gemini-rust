pub mod model;
pub use model::*;

impl FunctionDeclaration {
    /// Sets the parameter schema for this function directly from a JSON schema
    /// [`serde_json::Value`].
    ///
    /// This is a convenience for interop with crates (e.g. the Model Context
    /// Protocol SDK) that expose tool schemas as a [`serde_json::Value`] rather
    /// than a type implementing `schemars::JsonSchema`.
    pub fn with_parameters_value(mut self, schema: serde_json::Value) -> Self {
        self.parameters = Some(schema);
        self
    }

    /// Sets the parameter schema for this function directly from a JSON schema
    /// [`serde_json::Value`].
    ///
    /// Alias for [`Self::with_parameters_value`].
    pub fn with_parameters_json(mut self, schema: serde_json::Value) -> Self {
        self.with_parameters_value(schema)
    }

    /// Sets the response schema for this function directly from a JSON schema
    /// [`serde_json::Value`].
    ///
    /// This is a convenience for interop with crates (e.g. the Model Context
    /// Protocol SDK) that expose tool schemas as a [`serde_json::Value`] rather
    /// than a type implementing `schemars::JsonSchema`.
    pub fn with_response_value(mut self, schema: serde_json::Value) -> Self {
        self.response_schema = Some(schema);
        self
    }

    /// Sets the response schema for this function directly from a JSON schema
    /// [`serde_json::Value`].
    ///
    /// Alias for [`Self::with_response_value`].
    pub fn with_response_json(mut self, schema: serde_json::Value) -> Self {
        self.with_response_value(schema)
    }
}
