use crate::llm::LLMBackend;
use toml;

use dirs;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default = "default_archive_file_name")]
    pub archive_file_name: String,

    #[serde(default)]
    pub key_bindings: KeyBindings,

    #[serde(default = "default_llm_backend")]
    pub model: LLMBackend,

    #[serde(default)]
    pub chatgpt: ChatGPTConfig,
}

pub fn default_archive_file_name() -> String {
    String::from("tenere.archive")
}

pub fn default_llm_backend() -> LLMBackend {
    LLMBackend::ChatGPT
}

#[derive(Deserialize, Debug, Clone)]
pub struct ChatGPTConfig {
    pub openai_api_key: Option<String>,

    #[serde(default = "ChatGPTConfig::default_model")]
    pub model: String,

    #[serde(default = "ChatGPTConfig::default_url")]
    pub url: String,
}

impl Default for ChatGPTConfig {
    fn default() -> Self {
        Self {
            openai_api_key: None,
            model: Self::default_model(),
            url: Self::default_url(),
        }
    }
}

impl ChatGPTConfig {
    pub fn default_model() -> String {
        String::from("gpt-3.5-turbo")
    }

    pub fn default_url() -> String {
        String::from("https://api.openai.com/v1/chat/completions")
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

    #[serde(default = "KeyBindings::default_stop_stream")]
    pub stop_stream: char,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            show_help: '?',
            show_history: 'h',
            new_chat: 'n',
            save_chat: 's',
            stop_stream: 't',
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

    fn default_stop_stream() -> char {
        't'
    }
}

impl Config {
    pub fn load() -> Self {
        let conf_path = dirs::config_dir()
            .unwrap()
            .join("tenere")
            .join("config.toml");

        let config = std::fs::read_to_string(conf_path).unwrap_or(String::new());
        let app_config: Config = toml::from_str(&config).unwrap();
        app_config
    }
}
