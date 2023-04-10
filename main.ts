import { App, Modal, normalizePath, Notice, OpenViewState, PaneType, Plugin, PluginSettingTab, Pos, prepareSimpleSearch, renderResults, SearchResult, setIcon, Setting, SplitDirection, TFile, WorkspaceLeaf } from 'obsidian';

import * as plugin from "./pkg/obsidian_rust_plugin.js";
import * as wasmbin from './pkg/obsidian_rust_plugin_bg.wasm';

interface semanticSearchSettings {
	apiKey: string;
}

const DEFAULT_SETTINGS: semanticSearchSettings = {
	apiKey: ''
}

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

		this.addCommand({
			id: 'generate-embeddings-modal',
			name: 'Generate Embeddings',
			callback: () => {
				new GenerateEmbeddingsModal(this.app, this.settings).open();
			}
		});

		this.addSettingTab(new SettingTab(this.app, this));

		// here's the Rust bit
		await plugin.default(Promise.resolve(wasmbin.default));
		plugin.onload(this);
	}

	onunload() {

	}

	async loadSettings() {
		this.settings = Object.assign({}, DEFAULT_SETTINGS, await this.loadData());
	}

	async saveSettings() {
		await this.saveData(this.settings);
	}
}

type WASMSuggestion = {
  name: string
  header: string
}

class Suggestion {
  app: App;
  name: string;
  header: string;
  pos: Pos | undefined;
  file: TFile | undefined;
  match: SearchResult | undefined;

  constructor(app: App, wasmSuggestion: WASMSuggestion) {
    this.app = app;
    this.name = wasmSuggestion.name;
    this.header = wasmSuggestion.header;
  }

  // Find corresponding suggestion file
  addSuggestionFile() : Suggestion {
    const files = this.app.vault.getMarkdownFiles();
    const matching_file = files.find(file => file.name === this.name);
    this.file = matching_file;
    return this;
  }

  addSuggestionHeading() {
    const { metadataCache } = this.app;
    if (this.file) {
      const headingList = metadataCache.getFileCache(this.file)?.headings ?? [];
      const search = prepareSimpleSearch(this.header);
      headingList.forEach(heading => {
        if (heading.heading === this.header) {
          this.pos = heading.position;
          const match = search(heading.heading);
          if (match) {
            this.match = match;
          }
        }
      })
    }
    return this;
  }
}

export class QueryModal extends Modal {
  settings: semanticSearchSettings = DEFAULT_SETTINGS;
  estimatedCost = 0;
  timerId: number;
  delay = 200;

  constructor(app: App, settings: semanticSearchSettings) {
    super(app);
    this.settings = settings;
  }

  onOpen(): void {
      const contentEl = this.modalEl;
      this.modalEl.removeClass("modal");
      this.modalEl.addClass("prompt");
      this.modalEl.querySelector(".modal-close-button")?.remove();

      const inputContainer = contentEl.createDiv({cls: "prompt-input-container"})
      const input = inputContainer.createEl("input", {cls: "prompt-input"});

      const estimate_container = contentEl.createDiv({cls: "prompt-instructions"});
      const estimate_text = estimate_container.createDiv({cls: "prompt-instruction"});
      estimate_text.setText("Estimated cost of query: $0");
      input.addEventListener("input", (e) => {
        this.debounce(() => this.update_query_cost_estimate(e, estimate_text), this.delay);
      })

      const button = inputContainer.createEl("button", {text: "Submit", cls: "ss-query-submit-button"});
      const resultsDiv = contentEl.createDiv({cls: "prompt-results"});
      button.onclick = async () => {
        resultsDiv.replaceChildren();
        setIcon(resultsDiv, "loader");
        const suggestions: Suggestion[] = await this.getSuggestions(input.value);
        resultsDiv.replaceChildren();
        suggestions.forEach(suggestion => {
          this.renderSuggestion(suggestion, resultsDiv);
        })
      }
  }

  update_query_cost_estimate(e: Event, estimate_text: HTMLElement) {
    if (e.target) {
      const input = e.target as HTMLInputElement;
      this.estimatedCost = plugin.get_query_cost_estimate(input.value);
    }
    estimate_text.setText("Estimated cost of query: $" + this.estimatedCost);
  }

  debounce(fn: Function, delay_in_ms: number) {
    clearTimeout(this.timerId);
    this.timerId = setTimeout(fn, delay_in_ms);
  }

  onClose() {
    let { contentEl } = this;
    contentEl.empty();
  }

  // Returns all available suggestions.
  async getSuggestions(query: string): Promise<Suggestion[]> {
    const wasmSuggestions: WASMSuggestion[] = await plugin.get_suggestions(this.app, this.settings.apiKey, query);
    const suggestions: Suggestion[] = wasmSuggestions.map(wasmSuggestion => new Suggestion(this.app, wasmSuggestion));

    suggestions.forEach(suggestion => {
      suggestion.addSuggestionFile().addSuggestionHeading();
    })

    return suggestions;
  }

  // Renders each suggestion item.
  renderSuggestion(suggestion: Suggestion, el: HTMLElement) {
    const resultContainer = el.createDiv({cls: ["suggestion-item", "mod-complex", "ss-suggestion-item"]})
    resultContainer.onclick = async () => await this.onChooseSuggestion(suggestion);
    if (suggestion.match && suggestion.file) {
      const div = this.renderContent(resultContainer, suggestion.header, suggestion.match);
      this.renderPath(div, suggestion.file, suggestion.match);
    }
  }

  renderContent(
    parentEl: HTMLElement,
    content: string,
    match: SearchResult,
    offset?: number,
  ): HTMLDivElement {
    const contentEl = parentEl.createDiv({
      cls: 'suggestion-content',
    });

    const titleEl = contentEl.createDiv({
      cls: 'suggestion-title',
    });

    renderResults(titleEl, content, match, offset);

    return contentEl;
  }

  renderPath(
    parentEl: HTMLElement,
    file: TFile,
    match: SearchResult,
  ): void {
    if (parentEl && file) {
      const isRoot = file.parent.isRoot();
      let hidePath = isRoot;

      if (!hidePath) {
        const wrapperEl = parentEl.createDiv({ cls: 'suggestion-note' });
        const path = this.getPathDisplayText(file);

        const iconEl = wrapperEl.createSpan();
        setIcon(iconEl, 'folder');

        const pathEl = wrapperEl.createSpan();
        renderResults(pathEl, path, match);
      }
    }
  }

  getPathDisplayText(
    file: TFile,
  ): string {
    let text = '';

    if (file) {
      const { parent } = file;
      const dirname = parent.name;
      const isRoot = parent.isRoot();
      text = isRoot ? `${file.name}` : normalizePath(`${dirname}/${file.name}`);
    }

    return text;
  }

  // Perform action on the selected suggestion.
  async onChooseSuggestion(suggestion: Suggestion) {
    this.close();
    const isMatch = (candidateLeaf: WorkspaceLeaf) => {
      let val = false;

      if (candidateLeaf?.view) {
        val = candidateLeaf.view.file === suggestion.file;
      }

      return val;
    };
    const leaves: WorkspaceLeaf[] = [];
    this.app.workspace.iterateAllLeaves(leaf => leaves.push(leaf));
    const matchingLeaf = leaves.find(isMatch);

    const eState = {
      active: true,
      focus: true,
      startLoc: suggestion.pos?.start,
      endLoc: suggestion.pos?.end,
      cursor: {
        from: {line: suggestion.pos?.start.line, ch: suggestion.pos?.start.col },
        to: {line: suggestion.pos?.start.line, ch: suggestion.pos?.start.col },
      }
    }

    if (matchingLeaf === undefined) {
      if (suggestion.file) {
        await this.openFileInLeaf(suggestion.file, "tab", "vertical", {
          active: true,
          eState
        })
      }
    } else {
      this.app.workspace.setActiveLeaf(matchingLeaf, {focus: true});
      matchingLeaf.view.setEphemeralState(eState);
    }
  }

  async openFileInLeaf(file: TFile, navType: PaneType, splitDirection: SplitDirection = "vertical", openState: OpenViewState) {
    const { workspace } = this.app;
    const leaf = navType === "split" ? workspace.getLeaf(navType, splitDirection) : workspace.getLeaf(navType)
    await leaf.openFile(file, openState);
  }
}

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
         const exists_text = exists_container.createSpan({text: "Warning: the file 'embedding.csv' already exists.", cls: "ss-exists-text"})
       }
       estimate_text.setText("Estimated cost of query: $" + cost);
     } catch (error) {
       console.error(error)
     }

     const confirm_button = contentEl.createEl("button", {text: "Generate Embeddings"})
     confirm_button.onclick = async () => {
       this.close();
       await this.wasmGenerateEmbeddingsCommand.get_embeddings();
       new Notice("Successfully generated embeddings in 'embedding.csv'");
     }
  }

  onClose() {
    let { contentEl } = this;
    contentEl.empty();
  }
}

class SettingTab extends PluginSettingTab {
	plugin: SemanticSearch;

	constructor(app: App, plugin: SemanticSearch) {
		super(app, plugin);
		this.plugin = plugin;
	}

	display(): void {
		const {containerEl} = this;

		containerEl.empty();

		containerEl.createEl('h2', {text: 'Obsidian Semantic Search'});

		new Setting(containerEl)
			.setName('OpenAI API Key')
			.setDesc('https://platform.openai.com/account/api-keys')
			.addText(text => text
				.setPlaceholder('Enter your secret')
				.setValue(this.plugin.settings.apiKey)
				.onChange(async (value) => {
					this.plugin.settings.apiKey = value;
					await this.plugin.saveSettings();
				}));
	}
}
