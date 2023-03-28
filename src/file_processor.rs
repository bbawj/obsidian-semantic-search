use crate::obsidian::{DataAdapter, self};
use crate::SemanticSearchError;
use crate::obsidian::Vault;

pub struct FileProcessor {
    vault: Vault
}

impl FileProcessor {
    pub fn new(vault: Vault) -> Self {
        Self {vault}
    }
    pub fn adapter(&self) -> DataAdapter {
        self.vault.adapter()
    }

    pub async fn read_from_path(&self, path: &str) -> Result<String, SemanticSearchError> {
        let input = self.adapter().read(path.to_string()).await?.as_string().expect("Input csv is not a string");
        Ok(input)
    }

    pub async fn write_to_path(&self, data: String, path: &str) -> Result<(), SemanticSearchError> {
        self.adapter().append(path.to_string(), data).await?;
        Ok(())
    }

    pub async fn process_files(&self) -> Result<String, SemanticSearchError> {
        let files = self.vault.getMarkdownFiles();
        let mut wtr = csv::Writer::from_writer(vec![]);
        for file in files {
            let extracted = self.extract_sections(file).await.unwrap();
            for (file_name, header, body) in extracted {
                wtr.write_record(&[&file_name, &header, &body])?;
            }
        }
        let data = String::from_utf8(wtr.into_inner()?)?;
        Ok(data)
    }
    
    async fn extract_sections(&self, file: obsidian::TFile) -> std::io::Result<Vec<(String, String, String)>> {
        let mut header_to_content: Vec<(String, String, String)> = Vec::new();
        let name = file.name();
        match self.vault.read(file).await {
            Ok(text) => {
                let text = text.as_string().unwrap();
                let mut header = "".to_string();
                let mut body = "".to_string();
                let mut iterator = text.lines().peekable();
                while let Some(line) = iterator.next() {
                    if line.starts_with("##") {
                        header = line.replace("#", "").trim().to_string();
                        header_to_content.push((name.clone(), header.clone(), body.clone()));
                        body.clear();
                    } else {
                        body += line;
                        if iterator.peek().is_none() && header != "" {
                            header_to_content.push((name.clone(), header.clone(), body.clone()));
                        }
                    }
                }
            },
            Err(_) => todo!(),
        }
        Ok(header_to_content)
    }
    
}

