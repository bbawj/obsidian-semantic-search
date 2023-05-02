# Semantic Search for Obsidian

Find what you are looking for based on what you mean. A new file switcher powered by OpenAI's embedding API and built using WASM and Rust.

## Commands
|Command|Description|
|-------|-----------|
|Generate Input|Generate input csv based on sections of your notes. Currently, sections are defined as text blocks between headings. Prepared input is saved as `input.csv` in your root folder.
|Generate Embedding|Obtain embeddings via OpenAI's `text-embedding-ada-002` embedding model (this requires that the generate input command was successfully executed). Generated embeddings is saved as `embedding.csv` in your root folder.
|Open Query Modal|Semantic search through your notes using generated embeddings.
|Recommend links using current selection|Uses current editor selection as query input, automatically creating a markdown link with your choice. Can also be triggered in the context menu using the mouse right-click.

## Configuration
|Setting|Description|
|-------|-----------|
|API Key| Your OpenAI API key which can be found [here](https://platform.openai.com/account/api-keys). This gets stored into `data.json` as per all obsidian plugin settings data so make sure you do not commit this file to a repository.
|Section Delimeters| The type of heading to use to delimit a file into sections by Generate Input Command. Smaller headers are subsets of bigger headers, e.g. the H1 option will also split sections starting with H2, H3 etc. 
|Enable link recommendation using `{{}}`| Use `{{}}` as a way to trigger semantic search suggestions for file linking.

## Demo
https://user-images.githubusercontent.com/53790951/231014867-ce37c097-3b22-412a-9b1a-74204b0f167c.mp4

## Installing

From Obsidian v1.0.0, this plugin can be activated from within Obsidian:

1. Open Settings > Third-party plugin
2. Make sure Safe mode is off
3. Click Browse community plugins
4. Search for "Semantic Search"
5. Click the "Install" button
6. Once installed, close the community plugins window
7. Under the "Installed plugins" section, enable Semantic Search

From Github
1. Download the [latest release distribution](https://github.com/bbawj/obsidian-semantic-search/releases)
2. Extract the the contents of the distribution zip file to your vault's plugins folder: <vault>/.obsidian/plugins/ Note: On MacOs the .obsidian folder may be hidden by default.
3. Reload Obsidian
4. Open Settings, third-party plugins, make sure safe mode is off and enable "Semantic Search" from there.

## Contributing

Contributions are welcome!

### Getting Started
1. Clone the repo
2. cd into the newly created folder and run `yarn install`
3. Run `yarn run dev`

## Note
This plugin is very much experimental at the moment, use it at your own risk. Testing is done on Windows.

Thanks to [Robert's blog post](https://reasonabledeviations.com/2023/02/05/gpt-for-second-brain/?utm_source=pocket_saves) for the idea and inspiration!
