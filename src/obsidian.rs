use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "obsidian")]
extern "C" {
    pub type Plugin;

    #[wasm_bindgen(structural, method)]
    pub fn addCommand(this: &Plugin, command: JsValue);
    #[wasm_bindgen(method, getter)]
    pub fn app(this: &Plugin) -> App;
    #[wasm_bindgen(method, getter)]
    pub fn settings(this: &Plugin) -> semanticSearchSettings;

    pub type semanticSearchSettings;

    #[wasm_bindgen(method, getter)]
    pub fn apiUrl(this: &semanticSearchSettings) -> String;
    #[wasm_bindgen(method, getter)]
    pub fn apiKey(this: &semanticSearchSettings) -> String;
    #[wasm_bindgen(method, getter)]
    pub fn model(this: &semanticSearchSettings) -> String;
    #[wasm_bindgen(method, getter)]
    pub fn apiResponseType(this: &semanticSearchSettings) -> String;
    #[wasm_bindgen(method, getter)]
    pub fn debugMode(this: &semanticSearchSettings) -> bool;
    #[wasm_bindgen(method, getter)]
    pub fn ignoredFolders(this: &semanticSearchSettings) -> String;
    #[wasm_bindgen(method, getter)]
    pub fn sectionDelimeterRegex(this: &semanticSearchSettings) -> String;
    #[wasm_bindgen(method, getter)]
    pub fn numBatches(this: &semanticSearchSettings) -> u32;
    #[wasm_bindgen(method, getter)]
    pub fn maxTokenLength(this: &semanticSearchSettings) -> u32;

    #[derive(Clone)]
    pub type App;

    #[wasm_bindgen(method, getter)]
    pub fn vault(this: &App) -> Vault;

    pub type Vault;

    #[wasm_bindgen(method)]
    pub fn getRoot(this: &Vault) -> TFolder;
    #[wasm_bindgen(method)]
    pub fn getMarkdownFiles(this: &Vault) -> Vec<TFile>;
    #[wasm_bindgen(method, catch)]
    pub async fn cachedRead(this: &Vault, file: TFile) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(method, catch)]
    pub async fn append(this: &Vault, file: TFile, data: String) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(method, catch)]
    pub async fn create(this: &Vault, path: String, data: String) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(method, catch)]
    pub async fn delete(this: &Vault, file: TFile) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(method)]
    pub fn getAbstractFileByPath(this: &Vault, path: String) -> TAbstractFile;

    #[derive(Debug)]
    pub type TAbstractFile;

    #[derive(Debug)]
    pub type FileStats;
    #[wasm_bindgen(method, getter)]
    pub fn mtime(this: &FileStats) -> f64;

    #[derive(Debug)]
    #[wasm_bindgen(extends = TAbstractFile)]
    pub type TFile;

    #[wasm_bindgen(method, getter)]
    pub fn path(this: &TFile) -> String;
    #[wasm_bindgen(method, getter)]
    pub fn name(this: &TFile) -> String;
    #[wasm_bindgen(method, getter)]
    pub fn stat(this: &TFile) -> FileStats;
    #[wasm_bindgen(method, getter)]
    pub fn extension(this: &TFile) -> String;

    #[derive(Debug)]
    #[wasm_bindgen(extends = TAbstractFile)]
    pub type TFolder;
    #[wasm_bindgen(method, getter)]
    pub fn path(this: &TFolder) -> String;

    #[wasm_bindgen(method, getter)]
    pub fn children(this: &TFolder) -> Vec<TAbstractFile>;

    pub type Notice;

    #[wasm_bindgen(constructor)]
    pub fn new(message: &str) -> Notice;
}

#[wasm_bindgen(module = "main")]
extern "C" {
    pub type GenerateEmbeddingsModal;
    #[wasm_bindgen(constructor)]
    pub fn new(app: App) -> GenerateEmbeddingsModal;
    #[wasm_bindgen(method, getter)]
    pub fn isConfirmed(this: &GenerateEmbeddingsModal) -> bool;
    #[wasm_bindgen(method, getter)]
    pub fn isOpen(this: &GenerateEmbeddingsModal) -> bool;
    #[wasm_bindgen(method)]
    pub fn open(this: &GenerateEmbeddingsModal) -> bool;
}
