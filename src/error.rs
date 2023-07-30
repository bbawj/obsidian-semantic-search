use anyhow::anyhow;
use serde::Deserialize;
use wasm_bindgen::JsValue;

/// Wrapper to deserialize the error object nested in "error" JSON key
#[derive(Debug, Deserialize)]
pub(crate) struct WrappedError {
    pub(crate) error: ApiError,
}

impl std::fmt::Display for WrappedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.error)
    }
}

/// OpenAI API returns error object on failure
#[derive(Debug, Deserialize)]
pub struct ApiError {
    pub message: String,
    pub r#type: String,
    pub param: Option<serde_json::Value>,
    pub code: Option<serde_json::Value>,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Call to OpenAI failed with code: {:?}, message: {}, type: {}, param: {:?}",
            self.code, self.message, self.r#type, self.param
        )
    }
}

#[derive(Debug)]
pub struct SemanticSearchError(pub anyhow::Error);

impl std::error::Error for SemanticSearchError {}

impl std::fmt::Display for SemanticSearchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<anyhow::Error> for SemanticSearchError {
    fn from(value: anyhow::Error) -> Self {
        SemanticSearchError(value)
    }
}

impl From<wasm_bindgen::JsValue> for SemanticSearchError {
    fn from(value: wasm_bindgen::JsValue) -> Self {
        SemanticSearchError(anyhow!("{:?}", value))
    }
}

impl Into<wasm_bindgen::JsValue> for SemanticSearchError {
    fn into(self) -> wasm_bindgen::JsValue {
        JsValue::from_str(&format!("{:?}", self))
    }
}
