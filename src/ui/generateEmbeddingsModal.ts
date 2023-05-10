import { App, Modal, Notice } from "obsidian";
import { semanticSearchSettings } from "src/settings/settings.js";

import * as plugin from "../../pkg/obsidian_rust_plugin.js";

export class GenerateEmbeddingsModal extends Modal {
  wasmGenerateEmbeddingsCommand : plugin.GenerateEmbeddingsCommand;

  constructor(app: App, settings: semanticSearchSettings) {
    super(app);
    this.wasmGenerateEmbeddingsCommand = new plugin.GenerateEmbeddingsCommand(app, settings);
  }

  async onOpen() {
     const contentEl = this.contentEl;
     const estimate_container = contentEl.createDiv({cls: "ss-estimate-container"});
     const exists_container = contentEl.createDiv();
     const estimate_text = estimate_container.createDiv();
     estimate_text.setText("Estimated cost of query: ...");

     try {
       const cost = await this.wasmGenerateEmbeddingsCommand.get_input_cost_estimate();
       const exists = await this.wasmGenerateEmbeddingsCommand.check_embedding_file_exists();
       if (exists) {
         exists_container.createSpan({text: "Warning: the file 'embedding.csv' already exists.", cls: "ss-exists-text"})
       }
       estimate_text.setText("Estimated cost of query: $" + cost);
     } catch (error) {
       console.error(error)
     }

     const confirm_button = contentEl.createEl("button", {text: "Generate Embeddings"})
     confirm_button.onclick = async () => {
       this.close();
       try {
         await this.wasmGenerateEmbeddingsCommand.get_embeddings();
         new Notice("Successfully generated embeddings in 'embedding.csv'");
       } catch (error) {
         console.error(error);
         new Notice("Failed to create embeddings. Error: ", error);
       }
     }
  }

  onClose() {
    let { contentEl } = this;
    contentEl.empty();
  }
}

