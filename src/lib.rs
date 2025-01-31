mod embedding;
mod error;
mod file_processor;
mod generate_input;
mod obsidian;

extern crate console_error_panic_hook;

use crate::embedding::EmbeddingRequestBuilder;
use crate::file_processor::EmbeddingRow;
use crate::file_processor::EMBEDDING_FILE_PATH;
use crate::obsidian::Notice;
use std::panic;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use embedding::EmbeddingRequest;
use embedding::SupportedAPIs;
use error::SemanticSearchError;
use error::WrappedError;
use file_processor::FileProcessor;
use js_sys::JsString;
use log::debug;
use log::info;
use ndarray::Array1;
use obsidian::semanticSearchSettings;
use obsidian::App;
use serde::Deserialize;
use serde::Serialize;
use tiktoken_rs::cl100k_base;
use wasm_bindgen::prelude::*;

use crate::embedding::EmbeddingInput;

#[wasm_bindgen]
pub struct GenerateEmbeddingsCommand {
    file_processor: FileProcessor,
    client: Client,
    num_batches: u32,
}

#[wasm_bindgen]
pub struct NumModifiedResponse {
    pub nfiles: i64,
}

#[wasm_bindgen]
pub struct CostEstimateResponse {
    pub cost: f32,
}

#[wasm_bindgen]
impl GenerateEmbeddingsCommand {
    #[wasm_bindgen(constructor)]
    pub fn new(app: App, settings: &semanticSearchSettings) -> GenerateEmbeddingsCommand {
        let file_processor = FileProcessor::new(app.vault());
        let client = Client::new(settings);
        let num_batches = settings.numBatches();
        GenerateEmbeddingsCommand {
            file_processor,
            client,
            num_batches,
        }
    }

    pub async fn get_embeddings(&self) -> Result<(), SemanticSearchError> {
        let (_, modified_input, mut reusable_embeddings) =
            self.file_processor.read_modified_input().await?;
        self.file_processor.delete_embeddings().await?;

        let mut num_processed = 0;
        let num_batches = self.num_batches;
        let mut batch = 1;
        let num_records = modified_input.len();
        info!("Found {} records.", num_records);
        let batch_size = (num_records as f64 / num_batches as f64).ceil() as usize;
        let mut with_headers = true;

        while num_processed < num_records {
            let num_to_process = if batch == num_batches {
                num_records - num_processed
            } else {
                batch_size
            };

            let records = &modified_input[num_processed..num_processed + num_to_process].to_vec();
            debug!(
                "Processing batch {}: {} to {}",
                batch,
                num_processed,
                num_processed + num_to_process
            );

            let response: Vec<Vec<f32>> = self.client.get_embedding(records.into()).await?;
            info!("Sucessfully obtained {} embeddings", response.len());

            if records.len() != response.len() {
                return Err(SemanticSearchError(anyhow!(
                    "Requested for {} embeddings but got {}",
                    records.len(),
                    response.len()
                )));
            }

            let mut embedding_rows: Vec<EmbeddingRow> = Vec::with_capacity(num_to_process);

            records.into_iter().enumerate().for_each(|(i, record)| {
                let embedding = response
                    .get(i)
                    .map(|res| {
                        res.clone()
                            .into_iter()
                            .map(|f| f.to_string())
                            .collect::<Vec<String>>()
                            .join(",")
                    })
                    .expect("Length of records and response data should be aligned");

                embedding_rows.push(EmbeddingRow {
                    name: record.name.to_string(),
                    mtime: record.mtime.to_string(),
                    header: record.section.to_string(),
                    embedding,
                });
            });

            embedding_rows.append(&mut reusable_embeddings);
            self.file_processor
                .write_embedding_csv(embedding_rows, with_headers)
                .await?;

            num_processed += num_to_process;
            batch += 1;
            with_headers = false;
        }

        info!("Saved embeddings to {}", EMBEDDING_FILE_PATH);
        Ok(())
    }

    pub async fn get_input_n_modified(&self) -> Result<NumModifiedResponse, SemanticSearchError> {
        let (n_modified, _, _) = self.file_processor.read_modified_input().await?;
        Ok(NumModifiedResponse { nfiles: n_modified })
    }

    pub async fn get_input_cost_estimate(
        &self,
    ) -> Result<CostEstimateResponse, SemanticSearchError> {
        let (_, input, _) = self.file_processor.read_modified_input().await?;
        let string_records = input.into_iter().fold(String::new(), |mut acc, x| {
            acc.push_str(&x.body);
            acc
        });
        let estimate = get_query_cost_estimate(&string_records);
        Ok(CostEstimateResponse { cost: estimate })
    }

    pub async fn check_embedding_file_exists(&self) -> bool {
        return self
            .file_processor
            .check_file_exists_at_path(EMBEDDING_FILE_PATH)
            .await;
    }
}

#[wasm_bindgen]
pub struct QueryCommand {
    file_processor: FileProcessor,
    client: Client,
}

#[wasm_bindgen]
impl QueryCommand {
    async fn get_similarity(&self, query: String) -> Result<Vec<Suggestions>, SemanticSearchError> {
        struct Embedding<'a> {
            row: &'a EmbeddingRow,
            score: f32,
        }
        let rows = self.file_processor.read_embedding_csv().await?;
        let response: Vec<Vec<f32>> = self.client.get_embedding(query.into()).await?;
        info!("Sucessfully obtained {} embeddings", response.len());
        let query_embedding = Array1::from_vec(response[0].clone());

        let mut embeddings: Vec<Embedding> = Vec::with_capacity(rows.len());
        for row in &rows {
            let deserialized = deserialize_embeddings(&row.embedding).with_context(|| format!("Failed to deserialize embedding for file: {} and section: {} with embedding: {}", &row.name, &row.header, &row.embedding))?;
            embeddings.push(Embedding {
                score: cosine_similarity(&query_embedding, deserialized),
                row: &row,
            });
        }

        embeddings.sort_unstable_by(|row1: &Embedding, row2: &Embedding| {
            row1.score
                .partial_cmp(&row2.score)
                .expect("scores should be comparable")
        });
        embeddings.reverse();
        let ranked = embeddings
            .iter()
            .map(|e| Suggestions {
                name: e.row.name.to_string(),
                header: e.row.header.to_string(),
            })
            .collect();
        Ok(ranked)
    }
}

fn deserialize_embeddings(embedding: &str) -> Result<Vec<f32>> {
    embedding
        .split(",")
        .map(|s| {
            s.parse::<f32>()
                .context("Embedding should be comma-separated list of f32")
        })
        .collect()
}

fn cosine_similarity(a1: &Array1<f32>, right: Vec<f32>) -> f32 {
    let a2 = Array1::from_vec(right);
    a1.dot(&a2) / a1.dot(a1).sqrt() * a2.dot(&a2).sqrt()
}

#[derive(Deserialize, Serialize)]
pub struct Suggestions {
    name: String,
    header: String,
}

#[wasm_bindgen]
pub async fn get_suggestions(
    app: &obsidian::App,
    settings: &obsidian::semanticSearchSettings,
    query: JsString,
) -> Result<JsValue, JsError> {
    let query_string = query.as_string().unwrap();
    let file_processor = FileProcessor::new(app.vault());
    let client = Client::new(settings);
    let query_cmd = QueryCommand {
        file_processor,
        client,
    };
    let mut ranked_suggestions = query_cmd.get_similarity(query_string).await?;
    ranked_suggestions.truncate(10);
    Ok(serde_wasm_bindgen::to_value(&ranked_suggestions)?)
}

#[wasm_bindgen]
pub fn get_query_cost_estimate(query: &str) -> f32 {
    const TOKEN_COST: f32 = 0.0004 / 1000.0;
    let tokens = cl100k_base().unwrap().encode_with_special_tokens(query);
    let tokens_length = tokens.len() as f32;
    return TOKEN_COST * tokens_length;
}

#[derive(Debug, Clone)]
pub struct Client {
    api_url: String,
    api_key: String,
    model: String,
    api_response: SupportedAPIs,
}

impl Client {
    pub fn api_url(&self) -> &str {
        &self.api_url
    }

    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    fn new(settings: &obsidian::semanticSearchSettings) -> Self {
        Self {
            api_url: settings.apiUrl(),
            api_key: settings.apiKey(),
            model: settings.model(),
            api_response: settings.apiResponseType().into(),
        }
    }

    pub async fn get_embedding(
        &self,
        input: EmbeddingInput,
    ) -> Result<Vec<Vec<f32>>, SemanticSearchError> {
        let request = self.create_embedding_request(input)?;
        let response = self.post_embedding_request(request).await?;
        Ok(response)
    }

    fn create_embedding_request(&self, input: EmbeddingInput) -> Result<EmbeddingRequest> {
        let embedding_request = EmbeddingRequestBuilder::default()
            // TODO: add user param for model
            .model(self.model.clone())
            .input(input)
            .build()
            .context("Failed to build embedding request")?;
        Ok(embedding_request)
    }

    async fn post_embedding_request<I: serde::ser::Serialize>(
        &self,
        request: I,
    ) -> Result<Vec<Vec<f32>>> {
        let request = reqwest::Client::new()
            .post(self.api_url())
            .bearer_auth(self.api_key())
            .json(&request)
            .build()?;

        let reqwest_client = reqwest::Client::new();
        let response = reqwest_client
            .execute(request)
            .await
            .context(format!("Failed POST request to {}", self.api_url()))?;

        let status = response.status();
        let bytes = response.bytes().await?;

        if !status.is_success() {
            let wrapped_error: WrappedError = serde_json::from_slice(bytes.as_ref())?;
            return Err(anyhow!(wrapped_error));
        }

        let response: Vec<Vec<f32>> = match self.api_response {
            SupportedAPIs::Ollama => {
                let mut response: embedding::OllamaEmbeddingResponse =
                    serde_json::from_slice(bytes.as_ref())
                        .context("Failed deserializing Ollama embedding response")?;
                (&mut response).into()
            }
            SupportedAPIs::OpenAI => {
                let mut response: embedding::OpenAIEmbeddingResponse =
                    serde_json::from_slice(bytes.as_ref())
                        .context("Failed deserializing OpenAI embedding response")?;
                (&mut response).into()
            }
        };
        Ok(response)
    }
}

#[wasm_bindgen]
pub fn onload(plugin: &obsidian::Plugin) {
    if plugin.settings().debugMode() {
        console_log::init_with_level(log::Level::Debug).expect("");
    } else {
        console_log::init_with_level(log::Level::Info).expect("");
    }
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    info!("Semantic Search Loaded!");
}
