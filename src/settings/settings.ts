import SemanticSearch from "main";
import { App, PluginSettingTab, Setting } from "obsidian";
import { LinkSuggest } from "src/ui/linkSuggest";

export interface semanticSearchSettings {
	apiKey: string;
  sectionDelimeters: string;
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
			.setName('Section Delimeters')
			.setDesc('The type of heading to use to delimit a file into sections by Generate Input Command. Smaller headers are subsets of bigger headers, e.g. the H1 option will also split sections starting with H2, H3 etc.')
			.addDropdown(dropdownComponent => dropdownComponent
        .addOption("#", "H1: #")
        .addOption("##", "H2: ##")
        .addOption("###", "H3: ###")
        .addOption("####", "H4: ####")
        .addOption("#####", "H5: #####")
        .addOption("######", "H6: ######")
				.setValue(this.plugin.settings.sectionDelimeters)
				.onChange(async (value) => {
					this.plugin.settings.sectionDelimeters = value;
					await this.plugin.saveSettings();
				}));

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

