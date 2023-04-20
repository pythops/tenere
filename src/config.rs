use crossterm::event::KeyCode;
use serde::Deserialize;
use toml;

#[derive(Deserialize, Debug)]
pub struct AppConfig {
    gpt: Option<GPT>,
    key_bindings: Option<KeyBindings>,
}

#[derive(Deserialize, Debug)]
struct GPT {
    pub openai_api_key: String,
}

#[derive(Deserialize, Debug)]
struct KeyBindings {
    pub show_help: char,
}

pub fn load_config() -> AppConfig {
    let conf_path = dirs::config_dir()
        .unwrap()
        .join("tenere")
        .join("config.toml");

    let config = std::fs::read_to_string(conf_path).unwrap();
    let app_config: AppConfig = toml::from_str(&config).unwrap();
    app_config
}
