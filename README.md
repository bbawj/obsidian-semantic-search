# Semantic Search for Obsidian

Find what you are looking for based on what you mean. A new file switcher powered by OpenAI's embedding API and built using WASM and rust.

## Commands
|Command|Description|
|-------|-----------|
|Generate Input|Generate input csv based on sections of your notes. Currently, sections are defined as text blocks between headings (does not include H1). Prepared input is saved as `input.csv` in your root folder.
|Generate Embedding|Obtain embeddings via OpenAI's `text-embedding-ada-002` embedding model. Generaeted embeddings is saved as `embedding.csv` in your root folder.
|Open Query Modal|Semantic search through your notes using generated embeddings.

## Configuration
|Setting|Description|
|-------|-----------|
|API Key| Your OpenAI API key which can be found [here](https://platform.openai.com/account/api-keys)

## Demo
https://user-images.githubusercontent.com/53790951/231014867-ce37c097-3b22-412a-9b1a-74204b0f167c.mp4

## Note
This plugin is very much experimental at the moment, use it at your own risk. Testing is done on Windows.
