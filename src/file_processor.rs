use regex::Regex;
use lazy_static::lazy_static;

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

    pub async fn generate_input(&self) -> Result<String, SemanticSearchError> {
        let files = self.vault.getMarkdownFiles();
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
    
    async fn process_file(&self, file: obsidian::TFile) -> std::io::Result<Vec<(String, String, String)>> {
        let name = file.name();
        let res = match self.vault.cachedRead(file).await {
            Ok(text) => extract_sections(&name, &text.as_string().unwrap())?,
            Err(_) => todo!(),
        };
        Ok(res)
    }

}

fn extract_sections(name: &str, text: &str) -> std::io::Result<Vec<(String, String, String)>> {
    let mut header_to_content: Vec<(String, String, String)> = Vec::new();
    let text = clean_text(text);
    let mut header = "".to_string();
    let mut body = "".to_string();
    let mut iterator = text.lines().peekable();
    while let Some(line) = iterator.next() {
        let line = line.trim();

        if line.starts_with("##") {
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

        let res = extract_sections(NAME, text).unwrap();

        assert_eq!(res.len(), 1);
        assert_eq!(res.get(0).unwrap().0, "test");
        assert_eq!(res.get(0).unwrap().1, "Test");
        assert_eq!(res.get(0).unwrap().2, "");
    }

    #[test]
    fn empty_body() {
        let text = "## Test\n ";

        let res = extract_sections(NAME, text).unwrap();

        assert_eq!(res.len(), 1);
        assert_eq!(res.get(0).unwrap().0, "test");
        assert_eq!(res.get(0).unwrap().1, "Test");
        assert_eq!(res.get(0).unwrap().2, "");
    }

    #[test]
    fn non_empty_body() {
        let text = "## Test\nThis is a test body.";

        let res = extract_sections(NAME, text).unwrap();

        assert_eq!(res.len(), 1);
        assert_eq!(res.get(0).unwrap().0, "test");
        assert_eq!(res.get(0).unwrap().1, "Test");
        assert_eq!(res.get(0).unwrap().2, "This is a test body.");
    }

    #[test]
    fn double_line() {
        let text = "## Test\n## Test2";

        let res = extract_sections(NAME, text).unwrap();

        assert_eq!(res.len(), 2);
        assert_eq!(res.get(0).unwrap().0, "test");
        assert_eq!(res.get(0).unwrap().1, "Test");
        assert_eq!(res.get(0).unwrap().2, "");
        assert_eq!(res.get(1).unwrap().0, "test");
        assert_eq!(res.get(1).unwrap().1, "Test2");
        assert_eq!(res.get(1).unwrap().2, "");
    }

    #[test]
    fn header_three() {
        let text = "## Test\n### Test2";

        let res = extract_sections(NAME, text).unwrap();

        assert_eq!(res.len(), 2);
        assert_eq!(res.get(0).unwrap().0, "test");
        assert_eq!(res.get(0).unwrap().1, "Test");
        assert_eq!(res.get(0).unwrap().2, "");
        assert_eq!(res.get(1).unwrap().0, "test");
        assert_eq!(res.get(1).unwrap().1, "Test2");
        assert_eq!(res.get(1).unwrap().2, "");
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

        let res = extract_sections(NAME, text).unwrap();
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

        let res = extract_sections(NAME, text).unwrap();
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

        let res = extract_sections(NAME, text).unwrap();
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

