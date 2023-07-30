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
    /// ID of the model to use. You can use the
    /// [List models](https://platform.openai.com/docs/api-reference/models/list)
    /// API to see all of your available models, or see our
    /// [Model overview](https://platform.openai.com/docs/models/overview)
    /// for descriptions of them.
    pub model: String,

    /// Input text to get embeddings for, encoded as a string or array of tokens.
    /// To get embeddings for multiple inputs in a single request, pass an array
    /// of strings or array of token arrays. Each input must not exceed 8192
    /// tokens in length.
    pub input: EmbeddingInput,

    /// A unique identifier representing your end-user, which will help OpenAI
    ///  to monitor and detect abuse. [Learn more](https://platform.openai.com/docs/usage-policies/end-user-ids).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EmbeddingResponse {
    pub object: String,
    pub model: String,
    pub data: Vec<Embedding>,
    pub usage: EmbeddingUsage,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Embedding {
    pub index: u32,
    pub object: String,
    pub embedding: Vec<f32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EmbeddingUsage {
    pub prompt_tokens: u32,
    pub total_tokens: u32,
}
