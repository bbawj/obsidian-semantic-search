import SemanticSearch from "main";
import { App, PluginSettingTab, Setting } from "obsidian";

export interface semanticSearchSettings {
	apiKey: string;
  ignoredFolders: string;
  sectionDelimeterRegex: string;
  numBatches: number;
  enableLinkRecommendationSuggestor: boolean;
}

export class SemanticSearchSettingTab extends PluginSettingTab {
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

		new Setting(containerEl)
			.setName('Section Header Delimeter Regex')
			.setDesc("Regex used to determine if the current line is the start of a new section. Sections are used to group related content together. \
               Defaults to '.', meaning every line starts a new section. E.g. matching every heading: '^#{1,6}'. Requires reload.")
			.addText(text => text
				.setValue(this.plugin.settings.sectionDelimeterRegex)
				.onChange(async (value) => {
					this.plugin.settings.sectionDelimeterRegex = value;
					await this.plugin.saveSettings();
				}));

		new Setting(containerEl)
			.setName('Folders to ignore')
			.setDesc('Folders to ignore when generating input. Enter folder paths separated by newlines. Requires reload.')
			.addTextArea(text => text
				.setValue(this.plugin.settings.ignoredFolders)
				.onChange(async (value) => {
					this.plugin.settings.ignoredFolders = value;
					await this.plugin.saveSettings();
				}));

		new Setting(containerEl)
			.setName('Number of batches')
			.setDesc("Number of batches used to call OpenAI's endpoint. If you have lots of data, and are facing invalid request errors, try increasing this number.")
			.addSlider(slider => slider
				.setValue(this.plugin.settings.numBatches)
				.onChange(async (value) => {
					this.plugin.settings.numBatches = value;
					await this.plugin.saveSettings();
        })
        .setLimits(1, 100, 1)
        .setDynamicTooltip()
        .showTooltip());

    new Setting(containerEl)
    .setName("Enable link recommendation using {{}}")
    .setDesc("Typing '{{}}' will generate link recommendations for the text within the braces. Requires reload.")
    .addToggle(toggleComponent => toggleComponent
               .setValue(this.plugin.settings.enableLinkRecommendationSuggestor)
               .onChange(async (value) => {
                 this.plugin.settings.enableLinkRecommendationSuggestor = value;
                 await this.plugin.saveSettings();
               }));
	}
}

