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
    pub fn apiKey(this: &semanticSearchSettings) -> String;

    pub type App;

    #[wasm_bindgen(method, getter)]
    pub fn vault(this: &App) -> Vault;

    pub type Vault;

    #[wasm_bindgen(method)]
    pub fn getMarkdownFiles(this: &Vault) -> Vec<TFile>;
    #[wasm_bindgen(method, catch)]
    pub async fn cachedRead(this: &Vault, file: TFile) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(method, getter)]
    pub fn adapter(this: &Vault) -> DataAdapter;

    pub type DataAdapter;

    #[wasm_bindgen(method, catch)]
    pub async fn append(this: &DataAdapter, normalizedPath: String, data: String) -> Result<(), JsValue>;
    #[wasm_bindgen(method, catch)]
    pub async fn read(this: &DataAdapter, normalizedPath: String) -> Result<JsValue, JsValue>;

    pub type TFile;

    #[wasm_bindgen(method, getter)]
    pub fn path(this: &TFile) -> String;
    #[wasm_bindgen(method, getter)]
    pub fn name(this: &TFile) -> String;
    // #[wasm_bindgen(method, catch)]
    // pub fn read(this: &TFile) -> Result<JsValue, JsValue>;

    pub type Notice;

    #[wasm_bindgen(constructor)]
    pub fn new(message: &str) -> Notice;
}
