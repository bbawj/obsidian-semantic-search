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

#[wasm_bindgen]
pub struct GenerateInputCommand {
    id: JsString,
    name: JsString,
    file_processor: FileProcessor,
    section_delimeter: JsString,
}

#[wasm_bindgen]
impl GenerateInputCommand {
    pub fn build(id: JsString, name: JsString, file_processor: FileProcessor, section_delimeter: JsString) -> GenerateInputCommand {
        return GenerateInputCommand { id, name, file_processor, section_delimeter }
    }

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
        let files = self.file_processor.get_vault_markdown_files();
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
        let sections = extract_sections(&name, &text, &self.section_delimeter.as_string().unwrap())?;
        Ok(sections)
    }
}

fn extract_sections(name: &str, text: &str, delimeter: &str) -> Result<Vec<(String, String, String)>, SemanticSearchError> {
    let mut header_to_content: Vec<(String, String, String)> = Vec::new();
    let text = clean_text(text);
    let mut header = "".to_string();
    let mut body = "".to_string();
    let mut iterator = text.lines().peekable();
    while let Some(line) = iterator.next() {
        let line = line.trim();

        if line.starts_with(delimeter) {
            if header != "" {
                header_to_content.push((name.to_string(), header.clone(), body.clone()));
            }
            header = line.replace("#", "").trim().to_string();
            body.clear();
        } else {
            body += line;
        }

        if iterator.peek().is_none() && header != "" {
            header_to_content.push((name.to_string(), header.clone(), body.clone()));
        }
    }
    Ok(header_to_content)
}

fn clean_text(text: &str) -> String {
    let mut input = remove_links(text);
    input = input.trim().to_string();
    input
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
        let section_delimeter = "##";

        let res = extract_sections(NAME, text, &section_delimeter).unwrap();

        assert_eq!(res.len(), 1);
        assert_eq!(res.get(0).unwrap().0, "test");
        assert_eq!(res.get(0).unwrap().1, "Test");
        assert_eq!(res.get(0).unwrap().2, "");
    }

    #[test]
    fn empty_body() {
        let text = "## Test\n ";
        let section_delimeter = "##";

        let res = extract_sections(NAME, text, &section_delimeter).unwrap();

        assert_eq!(res.len(), 1);
        assert_eq!(res.get(0).unwrap().0, "test");
        assert_eq!(res.get(0).unwrap().1, "Test");
        assert_eq!(res.get(0).unwrap().2, "");
    }

    #[test]
    fn non_empty_body() {
        let text = "## Test\nThis is a test body.";
        let section_delimeter = "##";

        let res = extract_sections(NAME, text, &section_delimeter).unwrap();

        assert_eq!(res.len(), 1);
        assert_eq!(res.get(0).unwrap().0, "test");
        assert_eq!(res.get(0).unwrap().1, "Test");
        assert_eq!(res.get(0).unwrap().2, "This is a test body.");
    }

    #[test]
    fn double_line() {
        let text = "## Test\n## Test2";
        let section_delimeter = "##";

        let res = extract_sections(NAME, text, &section_delimeter).unwrap();

        assert_eq!(res.len(), 2);
        assert_eq!(res.get(0).unwrap().0, "test");
        assert_eq!(res.get(0).unwrap().1, "Test");
        assert_eq!(res.get(0).unwrap().2, "");
        assert_eq!(res.get(1).unwrap().0, "test");
        assert_eq!(res.get(1).unwrap().1, "Test2");
        assert_eq!(res.get(1).unwrap().2, "");
    }

    #[test]
    fn header_one() {
        let text = "# Test1\n## Test2\n### Test3\n#### Test4\n##### Test5\n###### Test6";
        let section_delimeter = "#";

        let res = extract_sections(NAME, text, &section_delimeter).unwrap();

        assert_eq!(res.len(), 6);
        assert_eq!(res.get(0).unwrap().0, "test");
        assert_eq!(res.get(0).unwrap().1, "Test1");
        assert_eq!(res.get(0).unwrap().2, "");
        assert_eq!(res.get(1).unwrap().0, "test");
        assert_eq!(res.get(1).unwrap().1, "Test2");
        assert_eq!(res.get(1).unwrap().2, "");
        assert_eq!(res.get(2).unwrap().0, "test");
        assert_eq!(res.get(2).unwrap().1, "Test3");
        assert_eq!(res.get(2).unwrap().2, "");
        assert_eq!(res.get(3).unwrap().0, "test");
        assert_eq!(res.get(3).unwrap().1, "Test4");
        assert_eq!(res.get(3).unwrap().2, "");
        assert_eq!(res.get(4).unwrap().0, "test");
        assert_eq!(res.get(4).unwrap().1, "Test5");
        assert_eq!(res.get(4).unwrap().2, "");
        assert_eq!(res.get(5).unwrap().0, "test");
        assert_eq!(res.get(5).unwrap().1, "Test6");
        assert_eq!(res.get(5).unwrap().2, "");
    }

    #[test]
    fn header_three() {
        let text = "# Test1\n## Test2\n### Test3";
        let section_delimeter = "###";

        let res = extract_sections(NAME, text, &section_delimeter).unwrap();

        assert_eq!(res.len(), 1);
        assert_eq!(res.get(0).unwrap().0, "test");
        assert_eq!(res.get(0).unwrap().1, "Test3");
        assert_eq!(res.get(0).unwrap().2, "");
    }


    #[test]
    fn header_four() {
        let text = "# Test1\n## Test2\n### Test3\n#### Test4";
        let section_delimeter = "####";

        let res = extract_sections(NAME, text, &section_delimeter).unwrap();

        assert_eq!(res.len(), 1);
        assert_eq!(res.get(0).unwrap().0, "test");
        assert_eq!(res.get(0).unwrap().1, "Test4");
        assert_eq!(res.get(0).unwrap().2, "");
    }

    #[test]
    fn header_five() {
        let text = "# Test1\n## Test2\n### Test3\n#### Test4\n##### Test5";
        let section_delimeter = "#####";

        let res = extract_sections(NAME, text, &section_delimeter).unwrap();

        assert_eq!(res.len(), 1);
        assert_eq!(res.get(0).unwrap().0, "test");
        assert_eq!(res.get(0).unwrap().1, "Test5");
        assert_eq!(res.get(0).unwrap().2, "");
    }
    #[test]
    fn header_six() {
        let text = "# Test1\n## Test2\n### Test3\n#### Test4\n##### Test5\n###### Test6";
        let section_delimeter = "######";

        let res = extract_sections(NAME, text, &section_delimeter).unwrap();

        assert_eq!(res.len(), 1);
        assert_eq!(res.get(0).unwrap().0, "test");
        assert_eq!(res.get(0).unwrap().1, "Test6");
        assert_eq!(res.get(0).unwrap().2, "");
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
    fn diff_header_with_content() {
        let text = "## Test\nTest content.\n### Test2\nTest content 2.";
        let section_delimeter = "##";

        let res = extract_sections(NAME, text, &section_delimeter).unwrap();
        println!("{:?}", res.get(0));

        assert_eq!(res.len(), 2);
        assert_eq!(res.get(0).unwrap().0, "test");
        assert_eq!(res.get(0).unwrap().1, "Test");
        assert_eq!(res.get(0).unwrap().2, "Test content.");
        assert_eq!(res.get(1).unwrap().0, "test");
        assert_eq!(res.get(1).unwrap().1, "Test2");
        assert_eq!(res.get(1).unwrap().2, "Test content 2.");
    }

    #[test]
    fn diff_header_with_links() {
        let text = "## Test\n![Pasted image 20220415211535](Pics/Pasted%20image%2020220415211535.png)\n### Test2\n![Pasted image 20220415211535](Pics/Pasted%20image%2020220415211535.png)";
        let section_delimeter = "##";

        let res = extract_sections(NAME, text, &section_delimeter).unwrap();
        println!("{:?}", res.get(0));

        assert_eq!(res.len(), 2);
        assert_eq!(res.get(0).unwrap().0, "test");
        assert_eq!(res.get(0).unwrap().1, "Test");
        assert_eq!(res.get(0).unwrap().2, "");
        assert_eq!(res.get(1).unwrap().0, "test");
        assert_eq!(res.get(1).unwrap().1, "Test2");
        assert_eq!(res.get(1).unwrap().2, "");
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
        assert_eq!(res.get(0).unwrap().2, "Does not guarantee anything. Such events are allowed:");
        assert_eq!(res.get(1).unwrap().0, "test");
        assert_eq!(res.get(1).unwrap().1, "Best Effort Broadcast");
        assert_eq!(res.get(1).unwrap().2, "Guarantees reliability only if sender is correct\
- BEB1. Best-effort-Validity: If pi and pj are correct, then any broadcast by pi is eventually delivered by pj\
- BEB2. No duplication: No message delivered more than once\
- BEB3. No creation: No message delivered unless broadcast");
    }
}
