use log::debug;
use log::info;
use regex::Regex;
use log::error;
use wasm_bindgen::prelude::*;
use lazy_static::lazy_static;
use anyhow::{Context, Result};

use crate::FileProcessor;
use crate::SemanticSearchError;
use crate::Notice;
use crate::file_processor::InputRow;
use crate::obsidian;
use crate::obsidian::App;
use crate::obsidian::semanticSearchSettings;

#[wasm_bindgen]
pub struct GenerateInputCommand {
    file_processor: FileProcessor,
    ignored_folders: String,
    section_delimeter_regex: String,
}

#[wasm_bindgen]
impl GenerateInputCommand {
    #[wasm_bindgen(constructor)]
    pub fn new(app: App, settings: semanticSearchSettings) -> GenerateInputCommand {
        let file_processor = FileProcessor::new(app.vault());
        let ignored_folders = settings.ignoredFolders();
        let section_delimeter_regex = settings.sectionDelimeterRegex();

        GenerateInputCommand { file_processor, ignored_folders, section_delimeter_regex}
    }

    pub async fn callback(&self) {
		let data: Vec<InputRow>;
        match self.generate_input().await {
			Ok(input) => data = input,
			Err(e) => {
				Notice::new(&format!("An error occurred generating inputs: {}", e));
				error!("{:?}", e);
				return;
			}
		}
		info!("Deleting input.csv");
        match self.file_processor.delete_input().await {
            Ok(()) => (),
            Err(e) => error!("{:?}", e),
        }
		info!("Writing input.csv");
        match self.file_processor.write_input_csv(data).await {
            Ok(()) => (),
            Err(e) => error!("{:?}", e),
        }

        Notice::new("Successfully created input.csv");
    }

    async fn generate_input(&self) -> Result<Vec<InputRow>, SemanticSearchError> {
        let files = self.file_processor.get_vault_markdown_files(self.ignored_folders.clone());
		info!("Found {} files", files.len());
		let mut folded_input: Vec<InputRow> = Vec::new();
        for file in files {
            match self.process_file(file).await {
				Ok(mut extracted) => {
					folded_input.append(&mut extracted);
				},
				Err(e) => error!("{:?}", e),
			}
        }
        Ok(folded_input)
    }

    async fn process_file(&self, file: obsidian::TFile) -> Result<Vec<InputRow>, SemanticSearchError> {
        let name = file.name();
		debug!("processing {}", name);
		let mtime = file.stat().mtime();
        let text = self.file_processor.read_from_file(file).await.context(format!("Failed to read {}", name))?;
		let sections = extract_sections(&name, &mtime.to_string(), &text, &self.section_delimeter_regex)?;
		Ok(sections)
	}
}

fn extract_sections(name: &str, mtime: &str, text: &str, delimeter: &str) -> Result<Vec<InputRow>, SemanticSearchError> {
    let mut output: Vec<InputRow> = Vec::new();
    let mut lines = text.lines().peekable();
    let re = match Regex::new(delimeter) {
        Ok(r) => r,
        Err(_) => {
            Notice::new("Invalid regex used, defaulting to '.'");
            Regex::new(".").unwrap()
        },
    };
    let mut section_header = "".to_string();
    let mut body = String::new();
    while let Some(line) = lines.next() {
        if re.is_match(&line) {
            if !(section_header.trim().is_empty() && body.trim().is_empty()) {
				let section_text = clean_text(&section_header);
				let body_text = clean_text(&body);
				if !(section_text.is_empty() && body_text.is_empty()) {
					output.push(InputRow { name: name.to_string(), mtime: mtime.to_string(), section: section_text, body: body_text});
				}
			}
			section_header = line.to_string();
			body = line.to_string();
		} else {
			if section_header.is_empty() {
				section_header = line.to_string();
			}
			let cleaned_line = clean_text(line);
			if !cleaned_line.is_empty() {
				body.push_str(&" ");
				body.push_str(&cleaned_line);
			}
		}
		if lines.peek().is_none() && !(section_header.trim().is_empty() && body.trim().is_empty()) {
			let section_text = clean_text(&section_header);
			let body_text = clean_text(&body);
			if !(section_text.is_empty() && body_text.is_empty()) {
				output.push(InputRow { name: name.to_string(), mtime: mtime.to_string(), section: section_text, body: body_text});
			}
		}
    }
    Ok(output)
}

fn clean_text(text: &str) -> String {
    const MAX_TOKEN_LENGTH: usize = 8191;
    let mut input = remove_hashtags(text);
    input = remove_links(&input);
    input = input.trim().to_string();

    input.truncate(MAX_TOKEN_LENGTH);
    input
}

fn remove_hashtags(text: &str) -> String {
    text.replace("#", "")
}

fn remove_links(text: &str) -> String {
    lazy_static! {
        static ref LINK_REGEX: Regex = Regex::new(r"!\[.*?\]\(.*?\)").unwrap();
    }
    let res = LINK_REGEX.replace_all(text, "");
    res.to_string()
}
    

#[cfg(test)]
mod tests {
    use super::*;
    const NAME: &str = "test";

    #[test]
    fn single_line() {
        let text = "## Test";
        let section_delimeter = r"^## \S*";

        let res = extract_sections(NAME, &" ", text, &section_delimeter).unwrap();

        assert_eq!(res.len(), 1);
        assert_eq!(res.get(0).unwrap().name, "test");
        assert_eq!(res.get(0).unwrap().section, "Test");
        assert_eq!(res.get(0).unwrap().body, "Test");
    }

	#[test]
	fn empty_section() {
        let text = " ";
        let section_delimeter = r".";

        let res = extract_sections(NAME, &" ", text, &section_delimeter).unwrap();

        assert_eq!(res.len(), 0);
	}

	#[test]
	fn empty_section_inbetween() {
        let text = "Test\n \nTest2\n ";
        let section_delimeter = r".";

        let res = extract_sections(NAME, &" ", text, &section_delimeter).unwrap();

        assert_eq!(res.len(), 2);
        assert_eq!(res.get(0).unwrap().section, "Test");
        assert_eq!(res.get(0).unwrap().body, "Test");
        assert_eq!(res.get(1).unwrap().section, "Test2");
        assert_eq!(res.get(1).unwrap().body, "Test2");
	}

    #[test]
    fn empty_body() {
        let text = "## Test\n ";
        let section_delimeter = r"^## \S*";

        let res = extract_sections(NAME, &" ", text, &section_delimeter).unwrap();

        assert_eq!(res.len(), 1);
        assert_eq!(res.get(0).unwrap().name, "test");
        assert_eq!(res.get(0).unwrap().section, "Test");
        assert_eq!(res.get(0).unwrap().body, "Test");
    }

    #[test]
    fn non_empty_body() {
        let text = "## Test\nThis is a test body.";
        let section_delimeter = r"^## \S*";

        let res = extract_sections(NAME, &" ", text, &section_delimeter).unwrap();

        assert_eq!(res.len(), 1);
        assert_eq!(res.get(0).unwrap().name, "test");
        assert_eq!(res.get(0).unwrap().section, "Test");
        assert_eq!(res.get(0).unwrap().body, "Test This is a test body.");
    }

    #[test]
    fn double_line() {
        let text = "## Test\n## Test2";
        let section_delimeter = r"^## .*";

        let res = extract_sections(NAME, &" ", text, &section_delimeter).unwrap();

        assert_eq!(res.len(), 2);
        assert_eq!(res.get(0).unwrap().name, "test");
        assert_eq!(res.get(0).unwrap().section, "Test");
        assert_eq!(res.get(0).unwrap().body, "Test");
        assert_eq!(res.get(1).unwrap().name, "test");
        assert_eq!(res.get(1).unwrap().section, "Test2");
        assert_eq!(res.get(1).unwrap().body, "Test2");
    }

    #[test]
    fn match_all_headers() {
        let text = "# Test1\ncontent1\n## Test2\ncontent2\n### Test3\ncontent3\n#### Test4\ncontent4\n##### Test5\ncontent5\n###### Test6\ncontent6";
        let section_delimeter = r"^#{1,6} ";

        let res = extract_sections(NAME, &" ", text, &section_delimeter).unwrap();
        println!("{:?}", res);

        assert_eq!(res.len(), 6);
        assert_eq!(res.get(0).unwrap().name, "test");
        assert_eq!(res.get(0).unwrap().section, "Test1");
        assert_eq!(res.get(0).unwrap().body, "Test1 content1");
        assert_eq!(res.get(1).unwrap().name, "test");
        assert_eq!(res.get(1).unwrap().section, "Test2");
        assert_eq!(res.get(1).unwrap().body, "Test2 content2");
        assert_eq!(res.get(2).unwrap().name, "test");
        assert_eq!(res.get(2).unwrap().section, "Test3");
        assert_eq!(res.get(2).unwrap().body, "Test3 content3");
        assert_eq!(res.get(3).unwrap().name, "test");
        assert_eq!(res.get(3).unwrap().section, "Test4");
        assert_eq!(res.get(3).unwrap().body, "Test4 content4");
        assert_eq!(res.get(4).unwrap().name, "test");
        assert_eq!(res.get(4).unwrap().section, "Test5");
        assert_eq!(res.get(4).unwrap().body, "Test5 content5");
        assert_eq!(res.get(5).unwrap().name, "test");
        assert_eq!(res.get(5).unwrap().section, "Test6");
        assert_eq!(res.get(5).unwrap().body, "Test6 content6");
    }

    #[test]
    fn diff_headers() {
        let text = "# Test1\ncontent1\n## Test2\ncontent2\n### Test3\ncontent3\n#### Test4\ncontent4\n##### Test5\ncontent5\n###### Test6\ncontent6";
        let section_delimeter = r"^### \S*";

        let res = extract_sections(NAME, &" ", text, &section_delimeter).unwrap();

        assert_eq!(res.len(), 2);
        assert_eq!(res.get(1).unwrap().name, "test");
        assert_eq!(res.get(1).unwrap().section, "Test3");
        assert_eq!(res.get(1).unwrap().body, "Test3 content3 Test4 content4 Test5 content5 Test6 content6");
    }

    #[test]
    fn remove_http_link() {
        let text = "![](https://test-link)";

        let res = remove_links(text);

        assert_eq!(res.len(), 0);
    }

    #[test]
    fn remove_pasted_image() {
        let text = "![Pasted image 20220415211535](Pics/Pasted%20image%2020220415211535.png)";

        let res = remove_links(text);

        assert_eq!(res.len(), 0);
    }

    #[test]
    fn remove_embedded() {
        let text = "## Test\n![Pasted image 20220415211535](Pics/Pasted%20image%2020220415211535.png)\n### Test2\n![Pasted image 20220415211535](Pics/Pasted%20image%2020220415211535.png)";

        let res = remove_links(text);

        assert_eq!(res, "## Test\n\n### Test2\n");
    }

    #[test]
    fn diff_header_with_links() {
        let text = "## Test\n![Pasted image 20220415211535](Pics/Pasted%20image%2020220415211535.png)\n### Test2\n![Pasted image 20220415211535](Pics/Pasted%20image%2020220415211535.png)";
        let section_delimeter = "^## .*";

        let res = extract_sections(NAME, &" ", text, &section_delimeter).unwrap();
        println!("{:?}", res.get(0));

        assert_eq!(res.len(), 1);
        assert_eq!(res.get(0).unwrap().name, "test");
        assert_eq!(res.get(0).unwrap().section, "Test");
        assert_eq!(res.get(0).unwrap().body, "Test Test2");
    }

    #[test]
    fn sample_note() {
        let text = "## Unreliable Broadcast
Does not guarantee anything. Such events are allowed:
![](https://i.imgur.com/rgh87f2.png)
## Best Effort Broadcast
Guarantees reliability only if sender is correct
- BEB1. Best-effort-Validity: If pi and pj are correct, then any broadcast by pi is eventually delivered by pj  
- BEB2. No duplication: No message delivered more than once  
- BEB3. No creation: No message delivered unless broadcast
![](https://i.imgur.com/LdLrtA0.png)
";
        let section_delimeter = "##";

        let res = extract_sections(NAME, &" ", text, &section_delimeter).unwrap();
        println!("{:?}", res.get(0));

        assert_eq!(res.len(), 2);
        assert_eq!(res.get(0).unwrap().name, "test");
        assert_eq!(res.get(0).unwrap().section, "Unreliable Broadcast");
        assert_eq!(res.get(0).unwrap().body, "Unreliable Broadcast Does not guarantee anything. Such events are allowed:");
        assert_eq!(res.get(1).unwrap().name, "test");
        assert_eq!(res.get(1).unwrap().section, "Best Effort Broadcast");
        assert_eq!(res.get(1).unwrap().body, "Best Effort Broadcast Guarantees reliability only if sender is correct \
- BEB1. Best-effort-Validity: If pi and pj are correct, then any broadcast by pi is eventually delivered by pj \
- BEB2. No duplication: No message delivered more than once \
- BEB3. No creation: No message delivered unless broadcast");
    }

    #[test]
    fn no_delimeter() {
        let text = "## Test\n![Pasted image 20220415211535](Pics/Pasted%20image%2020220415211535.png)\n### Test2\n![Pasted image 20220415211535](Pics/Pasted%20image%2020220415211535.png)";
        let section_delimeter = "";

        let res = extract_sections(NAME, &" ", text, &section_delimeter).unwrap();
        println!("{:?}", res.get(0));

        assert_eq!(res.len(), 2);
        assert_eq!(res.get(0).unwrap().name, "test");
        assert_eq!(res.get(0).unwrap().section, "Test");
        assert_eq!(res.get(0).unwrap().body, "Test");
        assert_eq!(res.get(1).unwrap().name, "test");
        assert_eq!(res.get(1).unwrap().section, "Test2");
        assert_eq!(res.get(1).unwrap().body, "Test2");
    }
}
