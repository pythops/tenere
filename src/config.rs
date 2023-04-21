use toml;

use dirs;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct AppConfig {
    #[serde(default)]
    pub key_bindings: KeyBindings,

    #[serde(default)]
    pub gpt: GPT,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GPT {
    pub openai_api_key: Option<String>,

    #[serde(default = "GPT::default_model")]
    pub model: String,
}

impl Default for GPT {
    fn default() -> Self {
        Self {
            openai_api_key: None,
            model: String::from("gpt-3.5-turbo"),
        }
    }
}

impl GPT {
    pub fn default_model() -> String {
        String::from("gpt-3.5-turbo")
    }
}

#[derive(Deserialize, Debug)]
pub struct KeyBindings {
    #[serde(default = "KeyBindings::default_show_help")]
    pub show_help: char,

    #[serde(default = "KeyBindings::default_show_history")]
    pub show_history: char,

    #[serde(default = "KeyBindings::default_new_chat")]
    pub new_chat: char,

    #[serde(default = "KeyBindings::default_save_chat")]
    pub save_chat: char,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            show_help: '?',
            show_history: 'h',
            new_chat: 'n',
            save_chat: 's',
        }
    }
}

impl KeyBindings {
    fn default_show_help() -> char {
        '?'
    }

    fn default_show_history() -> char {
        'h'
    }

    fn default_new_chat() -> char {
        'n'
    }

    fn default_save_chat() -> char {
        's'
    }
}

impl AppConfig {
    pub fn load() -> Self {
        let conf_path = dirs::config_dir()
            .unwrap()
            .join("tenere")
            .join("config.toml");

        let config = std::fs::read_to_string(conf_path).unwrap_or(String::new());
        let app_config: AppConfig = toml::from_str(&config).unwrap();
        app_config
    }
}
