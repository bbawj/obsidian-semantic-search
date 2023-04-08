export * from 'obsidian';

declare module 'obsidian' {
  export interface View {
    file?: TFile
  }
}

