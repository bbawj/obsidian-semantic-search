import { Editor, MarkdownView, Menu, Plugin } from 'obsidian';
import { semanticSearchSettings, SemanticSearchSettingTab } from 'src/settings/settings';
import { GenerateEmbeddingsModal } from 'src/ui/generateEmbeddingsModal';
import { LinkSuggest } from 'src/ui/linkSuggest';
import { LinkSuggestQueryModal, QueryModal } from 'src/ui/queryModal';

import * as plugin from "./pkg/obsidian_rust_plugin.js";
import * as wasmbin from './pkg/obsidian_rust_plugin_bg.wasm';

export default class SemanticSearch extends Plugin {
	settings: semanticSearchSettings;

	async onload() {
		await this.loadSettings();

		this.addRibbonIcon('file-search-2', 'Semantic Search', (_: MouseEvent) => {
      new QueryModal(this.app, this.settings).open();
		});

		this.addCommand({
			id: 'open-query-modal',
			name: 'Open query modal',
			callback: () => {
				new QueryModal(this.app, this.settings).open();
			}
		});

		const linkSuggestQueryCommand = this.addCommand({
			id: 'open-link-suggest-query-modal',
			name: 'Recommend links using current selection',
			editorCallback: (editor: Editor, view: MarkdownView) => {
				new LinkSuggestQueryModal(this.app, this.settings, editor).open();
			}
		});

		this.addCommand({
			id: 'generate-embeddings-modal',
			name: 'Generate Embeddings',
			callback: () => {
				new GenerateEmbeddingsModal(this.app, this.settings).open();
			}
		});

    if (this.settings.enableLinkRecommendationSuggestor) {
      const linksSuggest = new LinkSuggest(this.app, this.settings);
      this.registerEditorSuggest(linksSuggest);
    }

    this.registerEvent(
      this.app.workspace.on("editor-menu", (menu: Menu, editor: Editor) => {
        menu.addItem((item) => {
          item.setTitle(linkSuggestQueryCommand.name)
          .setIcon('file-search-2')
          .onClick(() => {
            //@ts-ignore
            this.app.commands.executeCommandById(linkSuggestQueryCommand.id);
          });
        });
      })
    );

		this.addSettingTab(new SemanticSearchSettingTab(this.app, this));

		// here's the Rust bit
		await plugin.default(Promise.resolve(wasmbin.default));
		plugin.onload(this);
	}

	onunload() {

	}

	async loadSettings() {
    const DEFAULT_SETTINGS: semanticSearchSettings = {
      apiKey: '',
      sectionDelimeters: '##',
      enableLinkRecommendationSuggestor: false
    }

		this.settings = Object.assign({}, DEFAULT_SETTINGS, await this.loadData());
	}

	async saveSettings() {
		await this.saveData(this.settings);
	}
}

