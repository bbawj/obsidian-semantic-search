use std::error::Error;
use crate::EmbeddingRequestBuilderError;
use csv::Writer;
use serde::Deserialize;
use wasm_bindgen::JsValue;

/// Wrapper to deserialize the error object nested in "error" JSON key
#[derive(Debug, Deserialize)]
pub(crate) struct WrappedError {
    pub(crate) error: ApiError,
}

/// OpenAI API returns error object on failure
#[derive(Debug, Deserialize)]
pub struct ApiError {
    pub message: String,
    pub r#type: String,
    pub param: Option<serde_json::Value>,
    pub code: Option<serde_json::Value>,
}

#[derive(Debug)]
pub enum SemanticSearchError {
    ObsidianError(JsValue),
    WriteError(csv::Error),
    ConversionError(Box<dyn std::error::Error>),
    ReqwestError(reqwest::Error),
    JSONDeserialize(serde_json::Error),
    ApiError(ApiError),
    InvalidArgument(String),
    GetEmbeddingsError(String),
}

impl std::fmt::Display for SemanticSearchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SemanticSearchError::ObsidianError(e) => write!(f, "obsidian error; {}", e.as_string().unwrap()),
            SemanticSearchError::WriteError(e) => write!(f, "write error; {:?}", e.source()),
            SemanticSearchError::ConversionError(e) => write!(f, "conversion error; {:?}", e.source()),
            SemanticSearchError::ReqwestError(e) => write!(f, "reqwest error; {}", e),
            SemanticSearchError::JSONDeserialize(e) => write!(f, "JSONDeserialize error: {:?}", e),
            SemanticSearchError::ApiError(e) => write!(f, "API error: {}: {}", e.r#type, e.message),
            SemanticSearchError::InvalidArgument(e) => write!(f, "Invalid argument: {}", e),
            SemanticSearchError::GetEmbeddingsError(e) => write!(f, "GetEmbeddingsError: {}", e),
        }
    }
}

impl From<csv::Error> for SemanticSearchError {
    fn from(value: csv::Error) -> Self {
        Self::WriteError(value)
    }
}

impl From<csv::IntoInnerError<Writer<Vec<u8>>>> for SemanticSearchError {
    fn from(value: csv::IntoInnerError<Writer<Vec<u8>>>) -> Self {
        Self::ConversionError(Box::new(value.into_error()))
    }
}

impl From<std::string::FromUtf8Error> for SemanticSearchError {
    fn from(value: std::string::FromUtf8Error) -> Self {
        Self::ConversionError(Box::new(value))
    }
}

impl From<wasm_bindgen::JsValue> for SemanticSearchError {
    fn from(value: wasm_bindgen::JsValue) -> Self {
        Self::ObsidianError(value)
    }
}

impl From<reqwest::Error> for SemanticSearchError {
    fn from(value: reqwest::Error) -> Self {
        Self::ReqwestError(value)
    }
}

impl From<EmbeddingRequestBuilderError> for SemanticSearchError {
    fn from(value: EmbeddingRequestBuilderError) -> Self {
        Self::InvalidArgument(value.to_string())
    }
}

impl std::error::Error for SemanticSearchError {
}


