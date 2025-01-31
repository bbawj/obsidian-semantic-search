# Semantic Search for Obsidian

Find what you are looking for based on what you mean. A new file switcher built using WASM and Rust.

## Quickstart

1. Setup API configuration in plugin settings
2. Run `Generate Input` command
3. Run `Generate Embedding` command
4. Run `Open Query Modal` command and start semantic searching!

## Demo
https://user-images.githubusercontent.com/53790951/231014867-ce37c097-3b22-412a-9b1a-74204b0f167c.mp4

## Commands
|Command|Description|
|-------|-----------|
|Generate Input|Generate input csv based on sections of your notes. Currently, sections are defined as text blocks between headings. Prepared input is saved as `input.csv` in your root folder.
|Generate Embedding|Obtain embeddings via the configured API URL (this requires that the generate input command was successfully executed). Generated embeddings is saved as `embedding.csv` in your root folder.
|Open Query Modal|Semantic search through your notes using generated embeddings.
|Recommend links using current selection|Uses current editor selection as query input, automatically creating a markdown link with your choice. Can also be triggered in the context menu using the mouse right-click.

## Configuration
|Setting|Description|
|-------|-----------|
|API URL| Any arbitrary url endpoint for obtaining embeddings (but make sure the response JSON is supported).
|API Key| Optional API key that is placed into Bearer Auth HTTP header. This gets stored into `data.json` as per all obsidian plugin settings data so make sure you do not commit this file to a repository.
|Model| The model id, passed in the key "model" of request.
|API response type| The type of response JSON expected to be returned from the URL.
|Section Delimeters| Regex used to determine if the current line is the start of a new section. Sections are used to group related content together. Defaults to `.`, meaning every line starts a new section. E.g. matching every heading: `^#{1,6} `
|Folders to ignore| Folders to ignore when generating input. Enter folder paths separated by newlines.
|Number of batches| Number of batches used to call OpenAI's endpoint. If you have lots of data, and are facing invalid request errors, try increasing this number.
|Enable link recommendation using `{{}}`| Use `{{}}` as a way to trigger semantic search suggestions for file linking.
|Enable cost estimation| Turn on/off input cost estimation that is based on a flat rate of $0.0004 / 1000 tokens.
|Enable debug mode logging| Turn on/off more verbose logging.

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

### Dependencies
1. [Rust and cargo](https://www.rust-lang.org/tools/install)
2. [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)

### Getting Started
1. Clone the repo
2. cd into the newly created folder and run `yarn install`
3. Run `yarn run dev`

## Note
This plugin is very much experimental at the moment, use it at your own risk. Testing is done on Windows.

Thanks to [Robert's blog post](https://reasonabledeviations.com/2023/02/05/gpt-for-second-brain/?utm_source=pocket_saves) for the idea and inspiration!
