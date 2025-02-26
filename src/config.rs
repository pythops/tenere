use crate::llm::LLMBackend;
use toml;

use dirs;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default)]
    pub key_bindings: KeyBindings,

    #[serde(default = "default_llm_backend")]
    pub llm: LLMBackend,

    #[serde(default)]
    pub chatgpt: ChatGPTConfig,

    pub llamacpp: Option<LLamacppConfig>,

    pub ollama: Option<OllamaConfig>,
    
    #[serde(default)]
    pub tts: TTSConfig,
}

pub fn default_llm_backend() -> LLMBackend {
    LLMBackend::ChatGPT
}

// ChatGPT
#[derive(Deserialize, Debug, Clone)]
pub struct ChatGPTConfig {
    pub openai_api_key: Option<String>,

    #[serde(default = "ChatGPTConfig::default_model")]
    pub model: String,

    #[serde(default = "ChatGPTConfig::default_url")]
    pub url: String,
    
    #[serde(default = "ChatGPTConfig::default_system_prompt")]
    pub system_prompt: String,
}

impl Default for ChatGPTConfig {
    fn default() -> Self {
        Self {
            openai_api_key: None,
            model: Self::default_model(),
            url: Self::default_url(),
            system_prompt: Self::default_system_prompt(),
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
    
    pub fn default_system_prompt() -> String {
        String::from("You are a helpful assistant.")
    }
}

// LLamacpp

#[derive(Deserialize, Debug, Clone)]
pub struct LLamacppConfig {
    pub url: String,
    pub api_key: Option<String>,
}

// Ollama

#[derive(Deserialize, Debug, Clone)]
pub struct OllamaConfig {
    pub url: String,
    pub model: String,
}

// TTS
#[derive(Deserialize, Debug, Clone)]
pub struct TTSConfig {
    #[serde(default = "TTSConfig::default_url")]
    pub url: String,
    
    #[serde(default)]
    pub default_voice: Option<String>,
}

impl Default for TTSConfig {
    fn default() -> Self {
        Self {
            url: Self::default_url(),
            default_voice: None,
        }
    }
}

impl TTSConfig {
    pub fn default_url() -> String {
        String::from("http://0.0.0.0:8000/v1/audio/speech")
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

    #[serde(default = "KeyBindings::default_stop_stream")]
    pub stop_stream: char,
    
    #[serde(default = "KeyBindings::default_load_voice")]
    pub load_voice: char,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            show_help: '?',
            show_history: 'h',
            new_chat: 'n',
            stop_stream: 't',
            load_voice: 'v',
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

    fn default_stop_stream() -> char {
        't'
    }
    
    fn default_load_voice() -> char {
        'v'
    }
}

impl Config {
    pub fn load(custom_path: Option<PathBuf>) -> Self {
        let conf_path = if let Some(path) = custom_path {
            path
        } else {
            dirs::config_dir()
                .unwrap()
                .join("tenere")
                .join("config.toml")
        };

        let config = std::fs::read_to_string(conf_path).unwrap_or_default();
        let mut app_config: Config = toml::from_str(&config).unwrap();

        if app_config.llm == LLMBackend::LLamacpp && app_config.llamacpp.is_none() {
            eprintln!("Config for LLamacpp is not provided");
            std::process::exit(1)
        }

        if app_config.llm == LLMBackend::Ollama && app_config.ollama.is_none() {
            eprintln!("Config for Ollama is not provided");
            std::process::exit(1)
        }
        
        // Try to load saved default voice from file if one exists
        let voice_file = dirs::config_dir()
            .unwrap()
            .join("tenere")
            .join("default_voice.txt");
            
        if voice_file.exists() {
            if let Ok(voice_id) = std::fs::read_to_string(&voice_file) {
                if !voice_id.trim().is_empty() {
                    app_config.tts.default_voice = Some(voice_id.trim().to_string());
                }
            }
        }

        app_config
    }
}
