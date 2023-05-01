import { App, Pos, prepareSimpleSearch, SearchResult, TFile } from "obsidian";

export type WASMSuggestion = {
  name: string
  header: string
}

export class Suggestion {
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
