use std::collections::HashMap;
use std::convert::TryInto;
use anyhow::{Context, Result};

use csv::ReaderBuilder;
use log::debug;
use log::info;
use serde::Deserialize;
use serde::Serialize;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

use crate::error::SemanticSearchError;
use crate::obsidian::TFile;
use crate::obsidian::TFolder;
use crate::obsidian::Vault;

pub const INPUT_FILE_PATH: &str = "input.csv";
pub const EMBEDDING_FILE_PATH: &str = "embedding.csv";

#[wasm_bindgen]
pub struct FileProcessor {
    vault: Vault,
}

#[derive(Serialize)]
pub(crate) struct WrittenInputRow<'a> {
	pub name: &'a str,
	pub mtime: &'a str,
	pub section: &'a str,
	pub body: &'a str
}

#[derive(Debug, Clone, Deserialize)]
pub struct InputRow {
	pub name: String,
	pub mtime: String,
	pub section: String,
	pub body: String
}

#[derive(Serialize)]
struct WrittenEmbeddingRow<'a> {
	name: &'a str,
	mtime: &'a str,
	header: &'a str,
	embedding: &'a str
}

#[derive(Debug, Deserialize)]
pub struct EmbeddingRow {
	pub name: String,
	pub mtime: String,
	pub header: String,
	pub embedding: String
}

impl FileProcessor {
    pub fn new(vault: Vault) -> Self {
        Self {vault}
    }

	pub async fn read_input_csv(&self) -> Result<Vec<InputRow>> {
		let input = self.read_from_path(INPUT_FILE_PATH).await.context(format!("Failed to read {}", INPUT_FILE_PATH))?;
		let mut reader = ReaderBuilder::new().trim(csv::Trim::All).flexible(false)
			.from_reader(input.as_bytes());
		let records = reader.deserialize().collect::<Result<Vec<InputRow>, csv::Error>>().context(format!("Failed to deserialize input.csv"))?;
		Ok(records)
	}

	pub async fn read_embedding_csv(&self) -> Result<Vec<EmbeddingRow>> {
		let input = self.read_from_path(EMBEDDING_FILE_PATH).await.context(format!("Failed to read {}", EMBEDDING_FILE_PATH))?;
		let mut reader = ReaderBuilder::new().trim(csv::Trim::All).flexible(false)
			.from_reader(input.as_bytes());
		let records = reader.deserialize().collect::<Result<Vec<EmbeddingRow>, csv::Error>>().context("Failed to deserialize embedding.csv")?;
		Ok(records)
	}

	// TODO: return a struct instead
	pub async fn read_modified_input(&self) -> Result<(i64, Vec<InputRow>, Vec<EmbeddingRow>)> {
        let mut input = self.read_input_csv().await.context("Failed to read input.csv. Try running 'Generate Input' first")?;

		if !self.check_file_exists_at_path(EMBEDDING_FILE_PATH).await {
			return Ok((-1, input, Vec::new()));
		}

		let prev_embeddings = self.read_embedding_csv().await.context("Failed to obtain previous embeddings")?;
		let mut name_to_modified: HashMap<String, (String, String)> = HashMap::new();
		prev_embeddings.into_iter().for_each(|e| {
			name_to_modified.insert(e.name, (e.mtime, e.embedding));
		});

		let mut embedding_rows: Vec<EmbeddingRow> = Vec::new();

		input.retain(|r| {
			if let Some((prev_mtime, prev_embedding)) = name_to_modified.get(&r.name) {
				if prev_mtime == &r.mtime {
					embedding_rows.push(EmbeddingRow { name: r.name.to_string(), mtime: r.mtime.to_string(), header: r.section.to_string(), embedding: prev_embedding.to_string() });
					return false;
				}
			}
			true
		});
		Ok((input.len().try_into().expect("Too many files"), input, embedding_rows))
	}

	pub async fn write_input_csv(&self, embeddings: Vec<InputRow>) -> Result<()> {
		let mut wtr = csv::Writer::from_writer(vec![]);
		for row in embeddings {
			wtr.serialize(WrittenInputRow {
				name: &row.name,
				mtime: &row.mtime,
				section: &row.section,
				body: &row.body
			})?;
		}
		let data = String::from_utf8(wtr.into_inner()?)?;
		self.write_to_path(INPUT_FILE_PATH, &data).await.context(format!("Failed to write to {}", INPUT_FILE_PATH))?;
		Ok(())
	}

	pub async fn write_embedding_csv(&self, embeddings: Vec<EmbeddingRow>, with_header: bool) -> Result<()> {
		let mut wtr = csv::WriterBuilder::new().has_headers(with_header).from_writer(vec![]);
		for row in embeddings {
			wtr.serialize(WrittenEmbeddingRow {
				name: &row.name,
				mtime: &row.mtime,
				header: &row.header,
				embedding: &row.embedding,
			}).context("Failed to serialize embedding row")?;
		}
		let data = String::from_utf8(wtr.into_inner()?)?;
		self.write_to_path(EMBEDDING_FILE_PATH, &data).await.context(format!("Failed to write to {}", EMBEDDING_FILE_PATH))?;
		Ok(())
	}

    async fn read_from_path(&self, path: &str) -> Result<String, SemanticSearchError> {
        let file: TFile = self.vault.getAbstractFileByPath(path.to_string()).unchecked_into();
        let input = self.vault.cachedRead(file).await?.as_string().expect("file contents is not a string");
        Ok(input)
    }

    pub async fn read_from_file(&self, file: TFile) -> Result<String, SemanticSearchError> {
        let input = self.vault.cachedRead(file).await?.as_string().expect("file contents is not a string");
        Ok(input)
    }

    async fn write_to_path(&self, path: &str, data: &str) -> Result<(), SemanticSearchError> {
        let file: TFile = self.vault.getAbstractFileByPath(path.to_string()).unchecked_into();
        if file.is_null() {
            debug!("File: {} does not exist. Creating it now.", path);
            self.vault.create(path.to_string(), data.to_string()).await?;
            return Ok(());
        }
        self.vault.append(file, data.to_string()).await?;
        Ok(())
    }

	pub async fn delete_input(&self) -> Result<()> {
		self.delete_file_at_path(INPUT_FILE_PATH).await.context(format!("Failed to delete {}", INPUT_FILE_PATH))?;
		Ok(())
	}

	pub async fn delete_embeddings(&self) -> Result<()> {
		self.delete_file_at_path(EMBEDDING_FILE_PATH).await.context(format!("Failed to delete {}", EMBEDDING_FILE_PATH))?;
		Ok(())
	}

    async fn delete_file_at_path(&self, path: &str) -> Result<(), SemanticSearchError> {
        let file: TFile = self.vault.getAbstractFileByPath(path.to_string()).unchecked_into();
        self.vault.delete(file).await?;
        Ok(())
    }

    pub async fn check_file_exists_at_path(&self, path: &str) -> bool {
        let file = self.vault.getAbstractFileByPath(path.to_string());
        if file.is_null() {
            return false;
        }
        true
    }

    pub fn get_vault_markdown_files(&self, ignored_folders_setting: String) -> Vec<TFile> {
        let root = self.vault.getRoot();
        let ignored_folders: Vec<String> = ignored_folders_setting.split("\n").map(|x| x.to_string()).collect();
        info!("Ignored folders: {:?}", &ignored_folders);
    
        return self.search_for_markdown_files(root, &ignored_folders);
    }

    fn search_for_markdown_files(&self, root: TFolder, ignored_folders: &Vec<String>) -> Vec<TFile> {
        let mut markdown_files: Vec<TFile> = Vec::new();

        for child in root.children() {
            if child.has_type::<TFolder>() {
                let folder = child.dyn_into::<TFolder>().expect("Folder should have TFolder type");
                if ignored_folders.contains(&folder.path()) {
                    continue;
                }
                markdown_files.extend(self.search_for_markdown_files(folder, &ignored_folders));
            } else {
                let file = child.dyn_into::<TFile>().expect("File should have TFile type");
                if file.extension() == "md" {
                    markdown_files.push(file);
                }
            }
        }

        return markdown_files;
    }
}
