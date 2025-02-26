use crate::llm::{LLMAnswer, LLMRole};
use crate::{chat::Chat, prompt::Mode};
use crate::event::TTSEvent;
use crate::config::{TTSConfig, Config};  // Add Config import

use crate::{
    app::{App, AppResult, FocusedBlock},
    event::Event,
};

use crate::llm::LLM;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use ratatui::text::Line;

use std::sync::Arc;
use tokio::sync::Mutex;

use tokio::sync::mpsc::UnboundedSender;

use crate::tts;
use std::path::Path;
use tokio::fs;
use crate::notification::{Notification, NotificationLevel};
use std::sync::atomic::Ordering;
use std::time::Duration;

pub async fn handle_key_events(
    key_event: KeyEvent,
    app: &mut App<'_>,
    llm: Arc<Mutex<Box<dyn LLM + 'static>>>,
    sender: UnboundedSender<Event>,
) -> AppResult<()> {
    match key_event.code {
        // Quit the app
        KeyCode::Char('q') if app.prompt.mode != Mode::Insert => {
            app.running = false;
        }

        KeyCode::Char('c') if key_event.modifiers == KeyModifiers::CONTROL => {
            app.running = false;
        }

        // Terminate the stream response
        KeyCode::Char('t') if key_event.modifiers == KeyModifiers::CONTROL => {
            app.terminate_response_signal
                .store(true, std::sync::atomic::Ordering::Relaxed);
        }

        // Read the current response with TTS
        KeyCode::Char('l') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            // Play the current answer with TTS
            if !app.chat.answer.plain_answer.is_empty() {
                sender.send(Event::TTSEvent(TTSEvent::PlayText {
                    text: app.chat.answer.plain_answer.clone(),
                    voice: None,
                }))?;
            }
        }

        // Load voice for TTS
        KeyCode::Char(c) if c == app.config.key_bindings.load_voice && 
                            key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            // Spawn an async task to handle voice loading
            let sender_clone = sender.clone();
            // Pass the actual app config here
            let config_clone = Arc::clone(&app.config);
            tokio::spawn(async move {
                match load_voice_file(sender_clone.clone(), config_clone).await {
                    Ok(voice_id) => {
                        sender_clone.send(Event::Notification(
                            Notification::new(
                                format!("Voice loaded successfully: {}", voice_id),
                                NotificationLevel::Info
                            )
                        )).unwrap_or_default();
                    },
                    Err(e) => {
                        sender_clone.send(Event::Notification(
                            Notification::new(
                                format!("Error loading voice: {}", e),
                                NotificationLevel::Error
                            )
                        )).unwrap_or_default();
                    }
                }
            });
        },

        // scroll down
        KeyCode::Char('j') | KeyCode::Down => match app.focused_block {
            FocusedBlock::History => {
                app.history.scroll_down();
            }

            FocusedBlock::Chat => {
                app.chat
                    .automatic_scroll
                    .store(false, std::sync::atomic::Ordering::Relaxed);
                app.chat.scroll = app.chat.scroll.saturating_add(1);
            }

            FocusedBlock::Preview => {
                app.history.preview.scroll = app.history.preview.scroll.saturating_add(1);
            }
            _ => (),
        },

        // scroll up
        KeyCode::Char('k') | KeyCode::Up => match app.focused_block {
            FocusedBlock::History => app.history.scroll_up(),

            FocusedBlock::Preview => {
                app.history.preview.scroll = app.history.preview.scroll.saturating_sub(1);
            }

            FocusedBlock::Chat => {
                app.chat
                    .automatic_scroll
                    .store(false, std::sync::atomic::Ordering::Relaxed);
                app.chat.scroll = app.chat.scroll.saturating_sub(1);
            }

            _ => (),
        },

        // `G`:  Mo to the bottom
        KeyCode::Char('G') => match app.focused_block {
            FocusedBlock::Chat => app.chat.move_to_bottom(),
            FocusedBlock::History => app.history.move_to_bottom(),
            _ => (),
        },

        // `gg`: Move to the top
        KeyCode::Char('g') => {
            if app.previous_key == KeyCode::Char('g') {
                match app.focused_block {
                    FocusedBlock::Chat => {
                        app.chat.move_to_top();
                    }
                    FocusedBlock::History => {
                        app.history.move_to_top();
                    }
                    _ => (),
                }
            }
        }

        // New chat
        KeyCode::Char(c)
            if c == app.config.key_bindings.new_chat
                && key_event.modifiers == KeyModifiers::CONTROL =>
        {
            app.prompt.clear();

            app.history
                .preview
                .text
                .push(app.chat.formatted_chat.clone());

            app.history.text.push(app.chat.plain_chat.clone());
            // after adding to history, save the chat in file
            app.history.save(app.history.text.len() - 1, sender.clone());

            app.chat = Chat::default();

            let llm = llm.clone();
            {
                let mut llm = llm.lock().await;
                llm.clear();
            }

            app.chat.scroll = 0;
        }

        // Switch the focus
        KeyCode::Tab => match app.focused_block {
            FocusedBlock::Chat => {
                app.focused_block = FocusedBlock::Prompt;

                app.chat
                    .automatic_scroll
                    .store(true, std::sync::atomic::Ordering::Relaxed);
            }
            FocusedBlock::Prompt => {
                app.chat.move_to_bottom();

                app.focused_block = FocusedBlock::Chat;
                app.prompt.mode = Mode::Normal;
            }
            FocusedBlock::History => {
                app.focused_block = FocusedBlock::Preview;
                app.history.preview.scroll = 0;
            }
            FocusedBlock::Preview => {
                app.focused_block = FocusedBlock::History;
                app.history.preview.scroll = 0;
            }
            _ => (),
        },

        // Show help
        KeyCode::Char(c)
            if c == app.config.key_bindings.show_help && app.prompt.mode != Mode::Insert =>
        {
            app.focused_block = FocusedBlock::Help;
            app.chat
                .automatic_scroll
                .store(true, std::sync::atomic::Ordering::Relaxed);
        }

        // Show history
        KeyCode::Char(c)
            if c == app.config.key_bindings.show_history
                && app.prompt.mode != Mode::Insert
                && key_event.modifiers == KeyModifiers::CONTROL =>
        {
            app.focused_block = FocusedBlock::History;
            app.chat
                .automatic_scroll
                .store(true, std::sync::atomic::Ordering::Relaxed);
        }

        // Discard help & history popups
        KeyCode::Esc => match app.focused_block {
            FocusedBlock::History | FocusedBlock::Preview | FocusedBlock::Help => {
                app.focused_block = FocusedBlock::Prompt
            }
            _ => {}
        },

        _ => {}
    }

    if let FocusedBlock::Prompt = app.focused_block {
        if let Mode::Normal = app.prompt.mode {
            if key_event.code == KeyCode::Enter {
                let user_input = app.prompt.editor.lines().join("\n");
                let user_input = user_input.trim();
                if user_input.is_empty() {
                    return Ok(());
                }

                app.prompt.clear();

                app.chat.plain_chat.push(format!("👤 : {}\n", user_input));

                if app.chat.formatted_chat.width() == 0 {
                    app.chat.formatted_chat = app
                        .formatter
                        .format(format!("👤: {}\n", user_input).as_str());
                } else {
                    app.chat.formatted_chat.extend(
                        app.formatter
                            .format(format!("👤: {}\n", user_input).as_str()),
                    );
                }

                let llm = llm.clone();
                {
                    let mut llm = llm.lock().await;
                    llm.append_chat_msg(user_input.into(), LLMRole::USER);
                }

                app.spinner.active = true;

                app.chat
                    .formatted_chat
                    .lines
                    .push(Line::raw("🤖: ".to_string()));

                let terminate_response_signal = app.terminate_response_signal.clone();

                let sender = sender.clone();

                let llm = llm.clone();

                tokio::spawn(async move {
                    let llm = llm.lock().await;
                    let res = llm.ask(sender.clone(), terminate_response_signal).await;

                    if let Err(e) = res {
                        sender
                            .send(Event::LLMEvent(LLMAnswer::StartAnswer))
                            .unwrap();
                        sender
                            .send(Event::LLMEvent(LLMAnswer::Answer(e.to_string())))
                            .unwrap();
                    }
                });
            }
        }

        app.prompt
            .handler(key_event, app.previous_key, app.clipboard.as_mut());
    }

    app.previous_key = key_event.code;

    Ok(())
}

/// Load a voice file from the configured directory and update the config
/// Cycles through available voices each time it's called
async fn load_voice_file(
    sender: UnboundedSender<Event>, 
    config: Arc<Config>
) -> Result<String, Box<dyn std::error::Error>> {
    // Get the voice directory
    let voice_dir = tts::get_voice_dir()?;
    
    // Read all files in the directory
    let mut entries = fs::read_dir(&voice_dir).await?;
    let mut voice_files = Vec::new();
    
    // Collect all audio files
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_file() {
            // Only include files with audio extensions
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                if ["mp3", "wav", "ogg", "m4a", "flac"].contains(&ext_str.as_str()) {
                    voice_files.push(path);
                }
            }
        }
    }
    
    // If there are no voice files, return an error
    if voice_files.is_empty() {
        return Err(format!("No voice files found in {:?}. Place audio files in this directory.", voice_dir).into());
    }
    
    // Sort the files to ensure consistent order
    voice_files.sort();
    
    // Get the last used voice file index
    let last_index_file = dirs::config_dir().unwrap().join("tenere").join("last_voice_index");
    let last_index = if last_index_file.exists() {
        match tokio::fs::read_to_string(&last_index_file).await {
            Ok(content) => content.trim().parse::<usize>().unwrap_or(0),
            Err(_) => 0
        }
    } else {
        0
    };
    
    // Calculate the next index (cycling through the list)
    let next_index = (last_index + 1) % voice_files.len();
    
    // Save the next index for future calls
    if let Some(parent) = last_index_file.parent() {
        if !parent.exists() {
            tokio::fs::create_dir_all(parent).await?;
        }
    }
    tokio::fs::write(&last_index_file, next_index.to_string()).await?;
    
    // Get the selected voice file
    let voice_path = &voice_files[next_index];
    let file_name = voice_path.file_name().unwrap().to_string_lossy().to_string();
    
    // Create a more reliable cache key using file name and file size
    let file_metadata = tokio::fs::metadata(voice_path).await?;
    let file_size = file_metadata.len();
    let cache_key = format!("{}_size_{}", file_name, file_size);
    
    // Debug the voice file selection
    // eprintln!("Selected voice file: {} (size: {} bytes)", file_name, file_size);
    
    // Check if we have a cached voice ID for this file to avoid re-uploading
    let cache_file = dirs::config_dir().unwrap().join("tenere").join("voice_cache.json");
    let mut voice_id = None;
    
    // Try to get the voice ID from cache first
    if cache_file.exists() {
        // eprintln!("Voice cache file exists at: {:?}", cache_file);
        
        match tokio::fs::read_to_string(&cache_file).await {
            Ok(content) => {
                // eprintln!("Read cache content: {} bytes", content.len());
                // Parse as JSON map directly - more robust error handling
                match serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&content) {
                    Ok(cache_map) => {
                        // First try with the cache_key
                        if let Some(id) = cache_map.get(&cache_key).and_then(|v| v.as_str()) {
                            voice_id = Some(id.to_string());
                            // eprintln!("Found voice ID in cache with key {}: {}", cache_key, id);
                        } 
                        // Fallback to just the filename
                        else if let Some(id) = cache_map.get(&file_name).and_then(|v| v.as_str()) {
                            voice_id = Some(id.to_string());
                            // eprintln!("Found voice ID in cache with filename {}: {}", file_name, id);
                        } else {
                            // eprintln!("No cache entry found for {} or {}", cache_key, file_name);
                        }
                    },
                    Err(e) => {
                        // eprintln!("Failed to parse voice cache: {}", e);
                    }
                }
            },
            Err(e) => {
                // eprintln!("Failed to read voice cache file: {}", e);
            }
        }
    } else {
        // eprintln!("Voice cache file doesn't exist yet at: {:?}", cache_file);
    }
    
    // If not found in cache, upload the file
    let voice_id = if let Some(id) = voice_id {
        // Voice found in cache, notify the user
        sender.send(Event::Notification(
            Notification::new(
                format!("Using voice: {} ({}/{})", 
                    file_name, next_index + 1, voice_files.len()),
                NotificationLevel::Info
            )
        ))?;
        id
    } else {
        // Voice not found in cache, upload it
        // eprintln!("No cached voice found, uploading file: {}", file_name);
        
        // Upload the voice file and get the voice ID
        let id = tts::upload_voice_file(voice_path, &config.tts).await?;
        // eprintln!("Voice uploaded successfully with ID: {}", id);
        
        // Create the cache map
        let mut cache_map = if cache_file.exists() {
            match tokio::fs::read_to_string(&cache_file).await {
                Ok(content) => match serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&content) {
                    Ok(map) => map,
                    Err(_) => {
                        // If parsing fails, create a fresh map
                        // eprintln!("Cache file exists but couldn't be parsed, creating new one");
                        serde_json::Map::new()
                    }
                },
                Err(_) => serde_json::Map::new()
            }
        } else {
            serde_json::Map::new()
        };
        
        // Add both the filename and the cache_key entries
        cache_map.insert(file_name.clone(), serde_json::Value::String(id.clone()));
        cache_map.insert(cache_key.clone(), serde_json::Value::String(id.clone()));
        
        let cache_content = serde_json::to_string_pretty(&cache_map)?;
        
        // Make sure the directory exists
        let parent = cache_file.parent().unwrap();
        if !parent.exists() {
            tokio::fs::create_dir_all(parent).await?;
        }
        
        // Write the updated cache
        match tokio::fs::write(&cache_file, &cache_content).await {
            // Ok(_) => eprintln!("Cache file updated successfully"),
            // Err(e) => eprintln!("Failed to write cache file: {}", e),
        }
        
        // Send notification that we're uploading a new voice
        sender.send(Event::Notification(
            Notification::new(
                format!("Uploaded new voice: {} ({}/{})", 
                    file_name, next_index + 1, voice_files.len()),
                NotificationLevel::Info
            )
        ))?;
        
        id
    };
    
    // Update the config file
    let config_dir = dirs::config_dir().unwrap().join("tenere");
    let config_path = config_dir.join("config.toml");
    
    // Read the existing config
    let config_content = match tokio::fs::read_to_string(&config_path).await {
        Ok(content) => content,
        Err(_) => String::new()
    };
    
    // Parse it as a document to preserve formatting and comments
    let mut doc = match config_content.parse::<toml_edit::Document>() {
        Ok(doc) => doc,
        Err(_) => toml_edit::Document::new()
    };
    
    // Update the voice in the config file
    if !doc.as_table().contains_key("tts") {
        doc["tts"] = toml_edit::Item::Table(toml_edit::Table::new());
    }
    doc["tts"]["default_voice"] = toml_edit::value(voice_id.clone());
    
    // Write the config back
    tokio::fs::write(&config_path, doc.to_string()).await?;
    
    // Update the in-memory config too
    let tts_config_ptr = &config.tts as *const TTSConfig as *mut TTSConfig;
    unsafe {
        (*tts_config_ptr).default_voice = Some(voice_id.clone());
    }
    
    // Return the voice ID
    Ok(voice_id)
}
