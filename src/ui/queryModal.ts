import { App, Editor, Modal, normalizePath, Notice, OpenViewState, PaneType, renderResults, SearchResult, setIcon, SplitDirection, TFile, WorkspaceLeaf } from "obsidian";
import { semanticSearchSettings } from "src/settings/settings";
import { Suggestion, WASMSuggestion } from "./suggestion";

import * as plugin from "../../pkg/obsidian_rust_plugin.js";

export class QueryModal extends Modal {
  settings: semanticSearchSettings;
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
    const suggestions: Suggestion[] = wasmSuggestions.map(wasmSuggestion => new Suggestion(this.app, wasmSuggestion, this.settings.sectionDelimeterRegex));

    suggestions.forEach(async suggestion => {
      await suggestion.addSuggestionFile().addSuggestionHeading();
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

export class LinkSuggestQueryModal extends QueryModal {
  editor: Editor;

  constructor(app: App, settings: semanticSearchSettings, editor: Editor) {
    super(app, settings);
    this.editor = editor;
  }

  onOpen(): void {
    const selection = this.editor.getSelection();
    if (selection === "") {
      new Notice("No selection found");
      this.close();
      return
    }

    super.onOpen();
    const input: HTMLInputElement | null = this.modalEl.querySelector(".prompt-input");

    if (input) {
      input.value = this.editor.getSelection();
      // trigger the input event which calculates estimated cost
      input.dispatchEvent(new InputEvent("input"));
    }
  }

  async onChooseSuggestion(suggestion: Suggestion) {
    this.close();
    const linkPath = normalizePath(encodeURI(suggestion.file?.path + "#" + suggestion.header));
    const textToLink = this.editor.getSelection();
    this.editor.replaceSelection(`[${textToLink}](${linkPath})`);
  }
}
