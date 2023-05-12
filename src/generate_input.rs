use log::debug;
use regex::Regex;
use js_sys::JsString;
use log::error;
use wasm_bindgen::prelude::*;
use lazy_static::lazy_static;

use crate::FileProcessor;
use crate::SemanticSearchError;
use crate::Notice;
use crate::DATA_FILE_PATH;
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
        let data = self.generate_input().await.expect("failed to generate input.csv");
        match self.file_processor.delete_file_at_path(DATA_FILE_PATH).await {
            Ok(()) => (),
            Err(e) => error!("{:?}", e),
        }
        match self.file_processor.write_to_path(DATA_FILE_PATH, &data).await {
            Ok(()) => (),
            Err(e) => error!("{:?}", e),
        }

        Notice::new("Successfully created input.csv");
    }

    async fn generate_input(&self) -> Result<String, SemanticSearchError> {
        let files = self.file_processor.get_vault_markdown_files(self.ignored_folders.clone());
        let mut wtr = csv::Writer::from_writer(vec![]);
        for file in files {
            let extracted = self.process_file(file).await.unwrap();
            for (file_name, header, body) in extracted {
                wtr.write_record(&[&file_name, &header, &body])?;
            }
        }
        let data = String::from_utf8(wtr.into_inner()?)?;
        Ok(data)
    }

    async fn process_file(&self, file: obsidian::TFile) -> Result<Vec<(String, String, String)>, SemanticSearchError> {
        let name = file.name();
        let text = self.file_processor.read_from_file(file).await?;
        let sections = extract_sections(&name, &text, &self.section_delimeter_regex)?;
        Ok(sections)
    }
}

fn extract_sections(name: &str, text: &str, delimeter: &str) -> Result<Vec<(String, String, String)>, SemanticSearchError> {
    let mut header_to_content: Vec<(String, String, String)> = Vec::new();
    let mut lines = text.lines().peekable();
    let re = match Regex::new(delimeter) {
        Ok(r) => r,
        Err(_) => {
            Notice::new("Invalid regex used, defaulting to '.'");
            Regex::new(".").unwrap()
        },
    };
    let mut section_header = "".to_string();
    let mut body = Vec::new();
    while let Some(line) = lines.next() {
        if re.is_match(&line) {
            if body.len() != 0 || section_header != "" {
                header_to_content.push((name.to_string(), clean_text(&section_header), clean_text(&body.join(" "))));
            }
            section_header = line.to_string();
            body = vec![line.to_string()];
        } else {
            if section_header == "" {
                section_header = line.to_string();
            }
            let cleaned_line = clean_text(line);
            if cleaned_line != "" {
                body.push(cleaned_line);
            }
        }
        if lines.peek().is_none() && (section_header != "" || body.len() != 0) {
            header_to_content.push((name.to_string(), clean_text(&section_header), clean_text(&body.join(" "))));
        }
    }
    Ok(header_to_content)
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
    let input = text.clone();
    lazy_static! {
        static ref LINK_REGEX: Regex = Regex::new(r"!\[.*?\]\(.*?\)").unwrap();
    }
    let res = LINK_REGEX.replace_all(input, "");
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

        let res = extract_sections(NAME, text, &section_delimeter).unwrap();

        assert_eq!(res.len(), 1);
        assert_eq!(res.get(0).unwrap().0, "test");
        assert_eq!(res.get(0).unwrap().1, "Test");
        assert_eq!(res.get(0).unwrap().2, "Test");
    }

    #[test]
    fn empty_body() {
        let text = "## Test\n ";
        let section_delimeter = r"^## \S*";

        let res = extract_sections(NAME, text, &section_delimeter).unwrap();

        assert_eq!(res.len(), 1);
        assert_eq!(res.get(0).unwrap().0, "test");
        assert_eq!(res.get(0).unwrap().1, "Test");
        assert_eq!(res.get(0).unwrap().2, "Test");
    }

    #[test]
    fn non_empty_body() {
        let text = "## Test\nThis is a test body.";
        let section_delimeter = r"^## \S*";

        let res = extract_sections(NAME, text, &section_delimeter).unwrap();

        assert_eq!(res.len(), 1);
        assert_eq!(res.get(0).unwrap().0, "test");
        assert_eq!(res.get(0).unwrap().1, "Test");
        assert_eq!(res.get(0).unwrap().2, "Test This is a test body.");
    }

    #[test]
    fn double_line() {
        let text = "## Test\n## Test2";
        let section_delimeter = r"^## .*";

        let res = extract_sections(NAME, text, &section_delimeter).unwrap();

        assert_eq!(res.len(), 2);
        assert_eq!(res.get(0).unwrap().0, "test");
        assert_eq!(res.get(0).unwrap().1, "Test");
        assert_eq!(res.get(0).unwrap().2, "Test");
        assert_eq!(res.get(1).unwrap().0, "test");
        assert_eq!(res.get(1).unwrap().1, "Test2");
        assert_eq!(res.get(1).unwrap().2, "Test2");
    }

    #[test]
    fn match_all_headers() {
        let text = "# Test1\ncontent1\n## Test2\ncontent2\n### Test3\ncontent3\n#### Test4\ncontent4\n##### Test5\ncontent5\n###### Test6\ncontent6";
        let section_delimeter = r"^#{1,6} ";

        let res = extract_sections(NAME, text, &section_delimeter).unwrap();
        println!("{:?}", res);

        assert_eq!(res.len(), 6);
        assert_eq!(res.get(0).unwrap().0, "test");
        assert_eq!(res.get(0).unwrap().1, "Test1");
        assert_eq!(res.get(0).unwrap().2, "Test1 content1");
        assert_eq!(res.get(1).unwrap().0, "test");
        assert_eq!(res.get(1).unwrap().1, "Test2");
        assert_eq!(res.get(1).unwrap().2, "Test2 content2");
        assert_eq!(res.get(2).unwrap().0, "test");
        assert_eq!(res.get(2).unwrap().1, "Test3");
        assert_eq!(res.get(2).unwrap().2, "Test3 content3");
        assert_eq!(res.get(3).unwrap().0, "test");
        assert_eq!(res.get(3).unwrap().1, "Test4");
        assert_eq!(res.get(3).unwrap().2, "Test4 content4");
        assert_eq!(res.get(4).unwrap().0, "test");
        assert_eq!(res.get(4).unwrap().1, "Test5");
        assert_eq!(res.get(4).unwrap().2, "Test5 content5");
        assert_eq!(res.get(5).unwrap().0, "test");
        assert_eq!(res.get(5).unwrap().1, "Test6");
        assert_eq!(res.get(5).unwrap().2, "Test6 content6");
    }

    #[test]
    fn diff_headers() {
        let text = "# Test1\ncontent1\n## Test2\ncontent2\n### Test3\ncontent3\n#### Test4\ncontent4\n##### Test5\ncontent5\n###### Test6\ncontent6";
        let section_delimeter = r"^### \S*";

        let res = extract_sections(NAME, text, &section_delimeter).unwrap();

        assert_eq!(res.len(), 2);
        assert_eq!(res.get(1).unwrap().0, "test");
        assert_eq!(res.get(1).unwrap().1, "Test3");
        assert_eq!(res.get(1).unwrap().2, "Test3 content3 Test4 content4 Test5 content5 Test6 content6");
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

        let res = extract_sections(NAME, text, &section_delimeter).unwrap();
        println!("{:?}", res.get(0));

        assert_eq!(res.len(), 1);
        assert_eq!(res.get(0).unwrap().0, "test");
        assert_eq!(res.get(0).unwrap().1, "Test");
        assert_eq!(res.get(0).unwrap().2, "Test Test2");
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

        let res = extract_sections(NAME, text, &section_delimeter).unwrap();
        println!("{:?}", res.get(0));

        assert_eq!(res.len(), 2);
        assert_eq!(res.get(0).unwrap().0, "test");
        assert_eq!(res.get(0).unwrap().1, "Unreliable Broadcast");
        assert_eq!(res.get(0).unwrap().2, "Unreliable Broadcast Does not guarantee anything. Such events are allowed:");
        assert_eq!(res.get(1).unwrap().0, "test");
        assert_eq!(res.get(1).unwrap().1, "Best Effort Broadcast");
        assert_eq!(res.get(1).unwrap().2, "Best Effort Broadcast Guarantees reliability only if sender is correct \
- BEB1. Best-effort-Validity: If pi and pj are correct, then any broadcast by pi is eventually delivered by pj \
- BEB2. No duplication: No message delivered more than once \
- BEB3. No creation: No message delivered unless broadcast");
    }

    #[test]
    fn no_delimeter() {
        let text = "## Test\n![Pasted image 20220415211535](Pics/Pasted%20image%2020220415211535.png)\n### Test2\n![Pasted image 20220415211535](Pics/Pasted%20image%2020220415211535.png)";
        let section_delimeter = "";

        let res = extract_sections(NAME, text, &section_delimeter).unwrap();
        println!("{:?}", res.get(0));

        assert_eq!(res.len(), 4);
        assert_eq!(res.get(0).unwrap().0, "test");
        assert_eq!(res.get(0).unwrap().1, "Test");
        assert_eq!(res.get(0).unwrap().2, "Test");
        assert_eq!(res.get(1).unwrap().0, "test");
        assert_eq!(res.get(1).unwrap().1, "");
        assert_eq!(res.get(1).unwrap().2, "");
        assert_eq!(res.get(2).unwrap().0, "test");
        assert_eq!(res.get(2).unwrap().1, "Test2");
        assert_eq!(res.get(2).unwrap().2, "Test2");
        assert_eq!(res.get(3).unwrap().0, "test");
        assert_eq!(res.get(3).unwrap().1, "");
        assert_eq!(res.get(3).unwrap().2, "");
    }
}
