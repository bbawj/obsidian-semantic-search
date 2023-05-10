import { App, Loc, Pos, SearchMatchPart, SearchResult, TFile } from "obsidian";
import Fuse from 'fuse.js';

export type WASMSuggestion = {
  name: string
  header: string
}

type Section = {
  text: string;
  start: number;
  end: number;
}

export class Suggestion {
  app: App;
  name: string;
  header: string;
  pos: Pos | undefined;
  file: TFile | undefined;
  match: SearchResult | undefined;
  sectionDelimeterRegex: string;

  constructor(app: App, wasmSuggestion: WASMSuggestion, sectionDelimeterRegex: string) {
    this.app = app;
    this.name = wasmSuggestion.name;
    this.header = wasmSuggestion.header;
    this.sectionDelimeterRegex = sectionDelimeterRegex;
  }

  // Find corresponding suggestion file
  addSuggestionFile() : Suggestion {
    const files = this.app.vault.getMarkdownFiles();
    const matching_file = files.find(file => file.name === this.name);
    this.file = matching_file;
    return this;
  }

  async addSuggestionHeading() {
    if (this.file) {
      const contents = await this.app.vault.cachedRead(this.file);
      const lines = contents.split("\n");
      const section_delimeter_regex = new RegExp(this.sectionDelimeterRegex);
      const sections: Section[] = [];
      let cur_section = "";
      let cur_idx = 0;

      for (let line of lines) {
        if (line.match(section_delimeter_regex) && cur_idx !== 0) {
          sections.push({
            text: cur_section,
            start: cur_idx - cur_section.length,
            end: cur_idx
          })
          cur_section = line;
        } else {
          cur_section += line;
        }
        cur_idx += line.length;
      }

      const options = {
        includeMatches: true,
        includeScore: true,
        ignoreLocation: true,
        minMatchCharLength: 2,
        keys: ["text"]
      }
      const fuse = new Fuse(sections, options);
      const result = fuse.search(this.header);
      if (result.length > 0) {
        const bestMatch = result[0];
        if (bestMatch.matches && bestMatch.score) {
          const matches = bestMatch.matches.slice();
          const indices: SearchMatchPart[] = [];
          for (let match of matches) {
            const match_indices = match.indices.slice();
            indices.concat(match_indices)
          }
          this.match = {
            score: bestMatch.score,
            matches: indices,
          };
        }

        this.pos = {start: getLocFromIndex(contents, bestMatch.item.start), end: getLocFromIndex(contents, bestMatch.item.end)};
      }
    }
  }
}

function getLocFromIndex(
  content: string,
  index: number
): Loc {
  const substr = content.slice(0, index);

  let l = 1;
  let r = -1;
  for (; (r = substr.indexOf("\n", r + 1)) !== -1; l++);

  return { line: l, col: 0, offset: index };
}
