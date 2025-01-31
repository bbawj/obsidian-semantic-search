use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::file_processor::InputRow;

#[derive(Debug, Serialize, Clone)]
#[serde(untagged)]
pub enum EmbeddingInput {
    StringArray(Vec<String>),
}

impl Default for EmbeddingInput {
    fn default() -> Self {
        EmbeddingInput::StringArray(vec!["".to_string()])
    }
}

impl From<String> for EmbeddingInput {
    fn from(value: String) -> Self {
        EmbeddingInput::StringArray(vec![value])
    }
}

impl From<Vec<String>> for EmbeddingInput {
    fn from(value: Vec<String>) -> Self {
        EmbeddingInput::StringArray(value)
    }
}

impl From<&[String]> for EmbeddingInput {
    fn from(value: &[String]) -> Self {
        EmbeddingInput::StringArray(value.to_vec())
    }
}

impl From<&[InputRow]> for EmbeddingInput {
    fn from(value: &[InputRow]) -> Self {
        EmbeddingInput::StringArray(value.to_vec().into_iter().map(|row| row.body).collect())
    }
}

impl From<&Vec<InputRow>> for EmbeddingInput {
    fn from(value: &Vec<InputRow>) -> Self {
        EmbeddingInput::StringArray(value.into_iter().map(|row| row.body.to_string()).collect())
    }
}

impl From<&mut Vec<InputRow>> for EmbeddingInput {
    fn from(value: &mut Vec<InputRow>) -> Self {
        EmbeddingInput::StringArray(value.into_iter().map(|row| row.body.to_string()).collect())
    }
}

#[derive(Debug, Serialize, Clone, Default, Builder)]
#[builder(pattern = "mutable")]
pub struct EmbeddingRequest {
    pub model: String,

    /// Input text to get embeddings for, encoded as a string or array of tokens.
    /// To get embeddings for multiple inputs in a single request, pass an array
    /// of strings or array of token arrays. For OpenAI: Each input must not exceed 8192
    /// tokens in length.
    pub input: EmbeddingInput,
}

#[derive(Debug, Clone)]
pub enum SupportedAPIs {
    Ollama,
    OpenAI,
}

impl From<std::string::String> for SupportedAPIs {
    fn from(value: std::string::String) -> Self {
        match value.as_str() {
            "Ollama" => Self::Ollama,
            "OpenAI" => Self::OpenAI,
            _ => todo!(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct OpenAIEmbeddingResponse {
    pub data: Vec<Embedding>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OllamaEmbeddingResponse {
    pub embeddings: Vec<Vec<f32>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Embedding {
    pub embedding: Vec<f32>,
}

impl From<&mut OpenAIEmbeddingResponse> for Vec<Vec<f32>> {
    fn from(value: &mut OpenAIEmbeddingResponse) -> Self {
        std::mem::take(&mut value.data)
            .iter_mut()
            .map(|x| std::mem::take(&mut x.embedding))
            .collect()
    }
}

impl From<&mut OllamaEmbeddingResponse> for Vec<Vec<f32>> {
    fn from(value: &mut OllamaEmbeddingResponse) -> Self {
        std::mem::take(&mut value.embeddings)
    }
}
