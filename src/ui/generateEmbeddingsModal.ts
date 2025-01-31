import { App, Modal, Notice } from "obsidian";
import { semanticSearchSettings } from "src/settings/settings.js";

import * as plugin from "../../pkg/obsidian_rust_plugin.js";

export class GenerateEmbeddingsModal extends Modal {
  wasmGenerateEmbeddingsCommand : plugin.GenerateEmbeddingsCommand;
  settings: semanticSearchSettings;

  constructor(app: App, settings: semanticSearchSettings) {
    super(app);
    this.wasmGenerateEmbeddingsCommand = new plugin.GenerateEmbeddingsCommand(app, settings);
	this.settings = settings;
  }

  async onOpen() {
     const contentEl = this.contentEl;
     const estimate_container = contentEl.createDiv({cls: "ss-estimate-container"});
     const exists_container = contentEl.createDiv();
     const estimate_text = estimate_container.createDiv();
     const nfiles_text = estimate_container.createDiv();
     estimate_text.setText("Processing estimated cost of query: ...");

     try {
       const exists = await this.wasmGenerateEmbeddingsCommand.check_embedding_file_exists();

	   const { nfiles } = await this.wasmGenerateEmbeddingsCommand.get_input_n_modified();
	   if (nfiles == 0) {
		   nfiles_text.setText(`Detected 0 files that are new or modified.`)
		   exists_container.createDiv({text: "Make sure to run 'Generate Input' after modifications.", cls: "ss-exists-text"})
	   } else {
		   if (this.settings.costEstimation) {
			   const { cost } = await this.wasmGenerateEmbeddingsCommand.get_input_cost_estimate();
			   estimate_text.setText(`Estimated cost of query: ${cost}`);
		   }
		   nfiles_text.setText(`Detected ${nfiles == -1 ? 'all' : nfiles} file(s) that are new or modified`)
		   const confirm_button = contentEl.createEl("button", {text: "Generate Embeddings"})
		   confirm_button.onclick = async () => {
			   this.close();
			   try {
				   await this.wasmGenerateEmbeddingsCommand.get_embeddings();
				   new Notice("Successfully generated embeddings in 'embedding.csv'");
			   } catch (error) {
				   console.error(error);
				   new Notice(`Failed to create embeddings. Error: ${error}`);
			   }
		   }
	   }
       if (exists) {
         exists_container.createDiv({text: "Warning: the file 'embedding.csv' already exists.", cls: "ss-exists-text"})
       }
     } catch (error) {
       this.close();
       console.error(error)
	   new Notice(`Failed to create embeddings. Error: ${error}`);
     }
  }

  onClose() {
    let { contentEl } = this;
    contentEl.empty();
  }
}

