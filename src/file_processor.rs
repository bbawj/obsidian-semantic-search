use log::debug;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

use crate::obsidian::TFile;
use crate::SemanticSearchError;
use crate::obsidian::TFolder;
use crate::obsidian::Vault;

#[wasm_bindgen]
pub struct FileProcessor {
    vault: Vault,
}

impl FileProcessor {
    pub fn new(vault: Vault) -> Self {
        Self {vault}
    }

    pub async fn read_from_path(&self, path: &str) -> Result<String, SemanticSearchError> {
        let file: TFile = self.vault.getAbstractFileByPath(path.to_string()).unchecked_into();
        let input = self.vault.cachedRead(file).await?.as_string().expect("file contents is not a string");
        Ok(input)
    }

    pub async fn read_from_file(&self, file: TFile) -> Result<String, SemanticSearchError> {
        let input = self.vault.cachedRead(file).await?.as_string().expect("file contents is not a string");
        Ok(input)
    }

    pub async fn write_to_path(&self, path: &str, data: &str) -> Result<(), SemanticSearchError> {
        let file: TFile = self.vault.getAbstractFileByPath(path.to_string()).unchecked_into();
        if file.is_null() {
            debug!("File: {} does not exist. Creating it now.", path);
            self.vault.create(path.to_string(), data.to_string()).await?;
            return Ok(());
        }
        self.vault.append(file, data.to_string()).await?;
        Ok(())
    }

    pub async fn delete_file_at_path(&self, path: &str) -> Result<(), SemanticSearchError> {
        let file: TFile = self.vault.getAbstractFileByPath(path.to_string()).unchecked_into();
        self.vault.delete(file).await?;
        Ok(())
    }

    pub async fn check_file_exists_at_path(&self, path: &str) -> Result<bool, SemanticSearchError> {
        let file = self.vault.getAbstractFileByPath(path.to_string());
        if file.is_null() {
            return Ok(false);
        }
        Ok(true)
    }

    pub fn get_vault_markdown_files(&self, ignored_folders_setting: String) -> Vec<TFile> {
        let root = self.vault.getRoot();
        let ignored_folders: Vec<String> = ignored_folders_setting.split("\n").map(|x| x.to_string()).collect();
        debug!("Ignored folders: {:?}", &ignored_folders);
    
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
