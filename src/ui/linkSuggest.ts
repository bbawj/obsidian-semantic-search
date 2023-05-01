import { App, debounce, Debouncer, Editor, EditorPosition, EditorSuggest, EditorSuggestContext, EditorSuggestTriggerInfo, normalizePath, renderResults, SearchResult, setIcon, TFile } from "obsidian";
import { semanticSearchSettings } from "src/settings/settings";
import { Suggestion, WASMSuggestion } from "./suggestion";

import * as plugin from "../../pkg/obsidian_rust_plugin.js";

export class LinkSuggest extends EditorSuggest<Suggestion> {
    app: App;
    settings: semanticSearchSettings;
    debouncer: Debouncer<[EditorSuggestContext, (suggestions: Suggestion[]) => void], void> | undefined;

    constructor(app: App, settings: semanticSearchSettings) {
      super(app);
      this.app = app;
      this.settings = settings;
    }

    onTrigger(cursor: EditorPosition, editor: Editor, file: TFile): EditorSuggestTriggerInfo | null {
      const line = editor.getLine(cursor.line);
      // using "{{}}" as a way to trigger this suggest
      const rx = /\{\{.*\}\}/;
      const matchedIdx = line.search(rx);

      if (matchedIdx == -1) {
        return null;
      }
      // cursor is not within braces
      if (cursor.ch <= matchedIdx + 1) {
        return null
      }

      return {
        start: {
          ch: matchedIdx, // For multi-word completion
          line: cursor.line,
        },
        end: {
          ch: cursor.ch + 2,
          line: cursor.line,
        },
        query: line.substring(matchedIdx+2, cursor.ch)
      };
    }

    async getSuggestions(context: EditorSuggestContext): Promise<Suggestion[]> {
      if (this.debouncer !== undefined) {
        this.debouncer.cancel();
      }

      this.debouncer = debounce(async (context: EditorSuggestContext, cb: (suggestions: Suggestion[]) => void) => {
        const query = context.query;
        console.log(query);

        if (query === "") {
          return []
        }

        const wasmSuggestions: WASMSuggestion[] = await plugin.get_suggestions(this.app, this.settings.apiKey, query);
        const suggestions: Suggestion[] = wasmSuggestions.map(wasmSuggestion => new Suggestion(this.app, wasmSuggestion));

        suggestions.forEach(suggestion => {
          suggestion.addSuggestionFile().addSuggestionHeading();
        })

        cb(suggestions);
      }, 500, true);


      return new Promise((resolve) => {
        if (this.debouncer !== undefined) {
          this.debouncer(context, (suggestions) => {
            resolve(suggestions)
          })
        }
      })
    }

    renderSuggestion(suggestion: Suggestion, el: HTMLElement): void {
      console.log(suggestion);
      const resultContainer = el.createDiv({cls: ["suggestion-item", "mod-complex" ]})
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

    selectSuggestion(suggestion: Suggestion, evt: MouseEvent | KeyboardEvent): void {
      const linkPath = normalizePath(encodeURI(suggestion.file?.path + "#" + suggestion.header));
      const textToLink = this.context?.query;
      this.context?.editor.replaceRange(`[${textToLink}](${linkPath})`, this.context.start, this.context.end);
    }
}


