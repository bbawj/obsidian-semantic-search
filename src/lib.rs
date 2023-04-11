mod obsidian;
mod embedding;
mod file_processor;
mod error;

use crate::embedding::EmbeddingRequestBuilderError;
use crate::embedding::EmbeddingRequestBuilder;

use csv::{ReaderBuilder, StringRecord};
use embedding::EmbeddingRequest;
use embedding::EmbeddingResponse;
use error::SemanticSearchError;
use error::WrappedError;
use file_processor::FileProcessor;
use js_sys::JsString;
use log::{debug, error};
use ndarray::Array1;
use obsidian::App;
use obsidian::semanticSearchSettings;
use reqwest::header::HeaderMap;
use serde::Deserialize;
use serde::Serialize;
use tiktoken_rs::cl100k_base;
use wasm_bindgen::prelude::*;

use crate::embedding::EmbeddingInput;

const DATA_FILE_PATH: &str = "input.csv";
const EMBEDDING_FILE_PATH: &str = "embedding.csv";

#[wasm_bindgen]
pub struct GenerateInputCommand {
    id: JsString,
    name: JsString,
    file_processor: FileProcessor,
}

#[wasm_bindgen]
impl GenerateInputCommand {
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> JsString {
        self.id.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_id(&mut self, id: &str) {
        self.id = JsString::from(id)
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> JsString {
        self.name.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_name(&mut self, name: &str) {
        self.name = JsString::from(name)
    }

    pub async fn callback(&self) {
        let data = self.file_processor.generate_input().await.expect("failed to generate input.csv");
        match self.file_processor.delete_file_at_path(DATA_FILE_PATH).await {
            Ok(()) => (),
            Err(e) => error!("{:?}", e),
        }
        match self.file_processor.write_to_path(DATA_FILE_PATH, &data).await {
            Ok(()) => (),
            Err(e) => error!("{:?}", e),
        }
    }
}

#[wasm_bindgen]
pub struct GenerateEmbeddingsCommand {
    file_processor: FileProcessor,
    client: Client,
}

#[wasm_bindgen]
impl GenerateEmbeddingsCommand {
    #[wasm_bindgen(constructor)]
    pub fn new(app: App, settings: semanticSearchSettings) -> GenerateEmbeddingsCommand {
        let file_processor = FileProcessor::new(app.vault());
        let client = Client::new(settings.apiKey());
        GenerateEmbeddingsCommand { file_processor, client }
    }

    pub async fn get_embeddings(&self) -> Result<(), SemanticSearchError> {
        self.file_processor.delete_file_at_path(EMBEDDING_FILE_PATH).await?;
        let input = self.file_processor.read_from_path(DATA_FILE_PATH).await?;
        let string_records = self.get_content_to_embed(input.clone())?;
        let request = self.client.create_embedding_request(string_records.into())?;
        let response = self.client.post_embedding_request(&request).await?;
        debug!("Sucessfully obtained {} embeddings", response.data.len());

        let filename_header = self.get_filename_header(input.clone())?;
        let mut wtr = csv::Writer::from_writer(vec![]);
        match request.input {
            EmbeddingInput::StringArray(arr) => {
                for (i, input) in arr.iter().enumerate() {
                    let filename_header = match filename_header.get(i) {
                        None => return Err(SemanticSearchError::GetEmbeddingsError(format!("Cannot find matching filename and header for input index {}", i)).into()),
                        Some(filename_header) => filename_header
                    };
                    let filename = &filename_header.0;
                    let header = &filename_header.1;
                    let embedding = match &response.data.get(i) {
                        None => return Err(SemanticSearchError::GetEmbeddingsError(format!("Cannot find matching embedding for filename: {}, header: {}", filename, header)).into()),
                        Some(embedding) => {
                            let vec: Vec<String> = embedding.embedding.clone().into_iter().map(|f| f.to_string()).collect();
                            vec.join(",")
                        }
                    };
                    wtr.write_record(&[filename, header, &embedding])?;
                }
            }
            EmbeddingInput::String(_) => unreachable!()
        }

        let data = String::from_utf8(wtr.into_inner()?)?;
        self.file_processor.write_to_path(EMBEDDING_FILE_PATH, &data).await?;
        debug!("Saved embeddings to {}", EMBEDDING_FILE_PATH);
        Ok(())
    }

    pub async fn get_input_cost_estimate(&self) -> Result<f32, SemanticSearchError> {
        let input = self.file_processor.read_from_path(DATA_FILE_PATH).await?;
        let string_records = self.get_content_to_embed(input)?;
        let combined_string = string_records.join("");
        let estimate = get_query_cost_estimate(&combined_string);
        Ok(estimate)
    }

    pub async fn check_embedding_file_exists(&self) -> Result<bool, SemanticSearchError> {
        let exists = self.file_processor.check_file_exists_at_path(EMBEDDING_FILE_PATH).await?;
        Ok(exists)
    }

    fn get_content_to_embed(&self, input: String) -> Result<Vec<String>, SemanticSearchError> {
        let mut reader = ReaderBuilder::new().trim(csv::Trim::All).flexible(false)
            .from_reader(input.as_bytes());
        let records = reader.records().collect::<Result<Vec<StringRecord>, csv::Error>>()?;
        let string_records = records.iter().map(|record| {
            let mut s = format!("{}. {}", record.get(1).unwrap(), record.get(2).unwrap());
            s.truncate(MAX_TOKEN_LENGTH);
            s
        }).collect();
        Ok(string_records)
    }

    fn get_filename_header(&self, input: String) -> Result<Vec<(String, String)>, SemanticSearchError> {
        let mut reader = ReaderBuilder::new().trim(csv::Trim::All).flexible(false)
            .from_reader(input.as_bytes());
        let records = reader.records().collect::<Result<Vec<StringRecord>, csv::Error>>()?;
        let filename_header = records.iter().map(|record| 
                           (record.get(0).unwrap().to_string(), record.get(1).unwrap().to_string())
                          ).collect();
        Ok(filename_header)
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
        let mut rows = self.get_embedding_rows().await?;
        let response = self.client.get_embedding(query.into()).await?;
        debug!("Sucessfully obtained {} embeddings", response.data.len());
        let query_embedding = response.data[0].clone().embedding;
        rows.sort_unstable_by(|row1, row2| cosine_similarity(query_embedding.clone(), row1.clone().2).partial_cmp(&cosine_similarity(query_embedding.to_owned(), row2.clone().2)).unwrap());
        rows.reverse();
        let ranked = rows.iter().map(|(name, header, _)| Suggestions { name: name.to_string(), header: header.to_string() }).collect();
        Ok(ranked)
    }

    async fn get_embedding_rows(&self) -> Result<Vec<(String, String, Vec<f32>)>, SemanticSearchError> {
        let input = self.file_processor.read_from_path(EMBEDDING_FILE_PATH).await?;
        let mut reader = ReaderBuilder::new().trim(csv::Trim::All).flexible(false)
            .from_reader(input.as_bytes());
        let records = reader.records().collect::<Result<Vec<StringRecord>, csv::Error>>()?;
        let rows = records.iter().map(|record| 
                           (record.get(0).unwrap().to_string(), 
                            record.get(1).unwrap().to_string(),
                            record.get(2).unwrap().to_string().split(",").map(|s| s.parse::<f32>().unwrap()).collect())
                          ).collect();
        Ok(rows)
    }
}

fn cosine_similarity(left: Vec<f32>, right: Vec<f32>) -> f32 {
    let a1  = Array1::from_vec(left);
    let a2 = Array1::from_vec(right);
    a1.dot(&a2) / a1.dot(&a1).sqrt() * a2.dot(&a2).sqrt()
}

#[derive(Deserialize, Serialize)]
pub struct Suggestions {
    name: String,
    header: String,
}

#[wasm_bindgen]
pub async fn get_suggestions(app: &obsidian::App, api_key: JsString, query: JsString) -> Result<JsValue, JsError> {
    let query_string = query.as_string().unwrap();
    let file_processor = FileProcessor::new(app.vault());
    let client = Client::new(api_key.as_string().unwrap());
    let query_cmd = QueryCommand { file_processor, client };
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
/// Client is a container for api key, base url, organization id
pub struct Client {
    api_key: String,
    api_base: String,
    org_id: String,
}

/// Default v1 API base url
pub const API_BASE: &str = "https://api.openai.com/v1";
/// Name for organization header
pub const ORGANIZATION_HEADER: &str = "OpenAI-Organization";
/// Maximum token length
pub const MAX_TOKEN_LENGTH: usize = 8192;

impl Client {
    pub fn api_base(&self) -> &str {
        &self.api_base
    }

    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    fn new(api_key: String) -> Self{
        Self { api_key, api_base: API_BASE.to_string(), org_id: Default::default() }
    }

    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        if !self.org_id.is_empty() {
            headers.insert(ORGANIZATION_HEADER, self.org_id.as_str().parse().unwrap());
        }
        headers
    }

    pub async fn get_embedding(&self, input: EmbeddingInput) -> Result<EmbeddingResponse, SemanticSearchError> {
        let request = self.create_embedding_request(input)?;
        let response = self.post_embedding_request(request).await?;
        Ok(response)
    }

    fn create_embedding_request(&self, input: EmbeddingInput) -> Result<EmbeddingRequest, SemanticSearchError> {
        let embedding_request = EmbeddingRequestBuilder::default()
            .model("text-embedding-ada-002".to_string())
            .input(input)
            .user(None)
            .build()?;
        Ok(embedding_request)
    }

    async fn post_embedding_request<I: serde::ser::Serialize>(&self, request: I) -> Result<EmbeddingResponse, SemanticSearchError> {
        let path = "/embeddings";

        let request = reqwest::Client::new()
            .post(format!("{}{path}", self.api_base()))
            .bearer_auth(self.api_key())
            .headers(self.headers())
            .json(&request)
            .build()?;

        let reqwest_client = reqwest::Client::new();
        let response = reqwest_client.execute(request).await?;

        let status = response.status();
        let bytes = response.bytes().await?;

        if !status.is_success() {
            let wrapped_error: WrappedError =
                serde_json::from_slice(bytes.as_ref()).map_err(SemanticSearchError::JSONDeserialize)?;

            return Err(SemanticSearchError::ApiError(wrapped_error.error));
        }

        let response: EmbeddingResponse =
            serde_json::from_slice(bytes.as_ref()).map_err(SemanticSearchError::JSONDeserialize)?;
        Ok(response)
    }
}

#[wasm_bindgen]
pub fn onload(plugin: &obsidian::Plugin) {
    console_log::init_with_level(log::Level::Debug).expect("");
    let generate_input_cmd = build_prepare_cmd(plugin);
    plugin.addCommand(JsValue::from(generate_input_cmd));
    debug!("Semantic Search Loaded!");
}

fn build_prepare_cmd(plugin: &obsidian::Plugin) -> GenerateInputCommand {
    let file_processor = FileProcessor::new(plugin.app().vault());
    GenerateInputCommand {
        id: JsString::from("generate-input"),
        name: JsString::from("Generate Input"),
        file_processor
    }
}
