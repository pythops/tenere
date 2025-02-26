use std::error::Error;
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use serde::Serialize;
use futures::StreamExt;
use reqwest::Client;
use reqwest::header;
use tokio::process::Command as TokioCommand;

// Debug helper macro - you can remove this after debugging
macro_rules! debug {
    ($($arg:tt)*) => {
        // Log to a file for debugging
        if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open("/tmp/tenere_tts_debug.log") {
            use std::io::Write;
            let _ = writeln!(&mut file, "[{}] {}", 
                chrono::Local::now().format("%H:%M:%S%.3f"),
                format!($($arg)*));
        }
    };
}

/// Request structure for the new TTS API
#[derive(Debug, Serialize)]
struct TTSRequest {
    model: String,
    input: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    voice: Option<String>,
    speed: f32,
    language: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    emotion: Option<serde_json::Value>,
    response_format: String,
}

/// Play text through TTS service with pure streaming (no file storage)
pub async fn play_tts(text: &str) -> Result<(), Box<dyn Error>> {
    debug!("TTS request for text: {}", text);
    
    // Add a terminal bell to indicate TTS is starting (optional)
    print!("\x07"); // Bell character
    
    // Skip empty or whitespace-only text
    let text = text.trim();
    if text.is_empty() {
        debug!("Skipping TTS for empty text");
        return Ok(());
    }
    
    // Build the request with the API parameters
    let request = TTSRequest {
        model: "Zyphra/Zonos-v0.1-transformer".to_string(),
        input: text.to_string(),
        voice: None,
        speed: 1.0,
        language: "en-us".to_string(),
        emotion: None,
        response_format: "mp3".to_string(),
    };

    debug!("Sending request to TTS API on port 8000");
    
    // Send request to TTS service
    let client = Client::new();
    let response = client.post("http://0.0.0.0:8000/v1/audio/speech")
        .json(&request)
        .send()
        .await?;

    let status = response.status();
    debug!("Got response with status: {}", status);
    
    if !status.is_success() {
        let error_text = response.text().await?;
        debug!("Error response: {}", error_text);
        return Err(format!("TTS request failed with status: {}, body: {}", status, error_text).into());
    }

    // Get the content type to pass to player
    let content_type = response.headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("audio/mp3")
        .to_string();
    
    debug!("Content type: {}", content_type);

    // Stream the audio directly to the player
    stream_audio(response, &content_type).await
}

/// Stream audio data directly to a player
async fn stream_audio(
    response: reqwest::Response, 
    content_type: &str
) -> Result<(), Box<dyn Error>> {
    debug!("Starting audio streaming");
    
    // Set up a suitable player based on what's available
    debug!("Setting up audio player");
    let (mut player_child, mut player_stdin) = match setup_streaming_player(content_type) {
        Ok(player) => player,
        Err(e) => {
            debug!("Player setup failed: {}", e);
            return Err(e);
        }
    };
    
    // Process chunks as they arrive
    let mut stream = stream_helpers::get_stream(response);
    let mut total_bytes = 0;
    let mut chunk_count = 0;
    
    debug!("Starting to receive audio chunks");
    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                chunk_count += 1;
                total_bytes += chunk.len();
                debug!("Received chunk #{} - {} bytes", chunk_count, chunk.len());
                
                // Write directly to player's stdin
                if let Err(e) = player_stdin.write_all(&chunk).await {
                    debug!("Error writing to player: {}", e);
                    return Err(e.into());
                }
            },
            Err(e) => {
                debug!("Error in stream: {}", e);
                return Err(e.into());
            }
        }
    }
    
    debug!("All chunks received. Total: {} chunks, {} bytes", chunk_count, total_bytes);
    
    // Close stdin to signal end of input
    drop(player_stdin);
    debug!("Closed stdin, waiting for player to finish");
    
    // Wait for player to finish
    let status = player_child.wait().await?;
    
    if !status.success() {
        let code = status.code().unwrap_or(-1);
        debug!("Player exited with error code: {}", code);
        return Err(format!("Audio player exited with code {}", code).into());
    }
    
    debug!("Audio playback completed successfully");
    Ok(())
}

// Helper function to get a stream from response
mod stream_helpers {
    use futures::Stream;
    use futures::StreamExt;
    use std::pin::Pin;
    
    pub fn get_stream(
        response: reqwest::Response
    ) -> Pin<Box<dyn Stream<Item = Result<Vec<u8>, reqwest::Error>> + Send>> {
        Box::pin(response.bytes_stream().map(|result| {
            result.map(|bytes| bytes.to_vec())
        }))
    }
}

/// Set up a streaming audio player based on what's available
fn setup_streaming_player(content_type: &str) -> Result<(tokio::process::Child, tokio::process::ChildStdin), Box<dyn Error>> {
    // Try to find which players are available on the system
    let mpv_available = std::process::Command::new("mpv").arg("--version").output().is_ok();
    let ffplay_available = std::process::Command::new("ffplay").arg("-version").output().is_ok();
    let aplay_available = std::process::Command::new("aplay").arg("--version").output().is_ok();
    
    debug!("Available players: mpv={}, ffplay={}, aplay={}", 
           mpv_available, ffplay_available, aplay_available);
    
    // Try mpv first (most versatile)
    if mpv_available {
        debug!("Trying to use mpv for playback");
        let mut command = TokioCommand::new("mpv")
            .args(["-", "--no-cache", "--no-terminal", "--audio-buffer=0.1"])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
            
        let stdin = command.stdin.take()
            .ok_or_else(|| "Failed to open mpv stdin".to_string())?;
        debug!("Successfully started mpv");
        return Ok((command, stdin));
    }
    
    // Try ffplay as second option
    if ffplay_available {
        debug!("Trying to use ffplay for playback");
        let mut command = TokioCommand::new("ffplay")
            .args(["-i", "pipe:0", "-autoexit", "-nodisp", "-hide_banner", "-loglevel", "quiet"])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
            
        let stdin = command.stdin.take()
            .ok_or_else(|| "Failed to open ffplay stdin".to_string())?;
        debug!("Successfully started ffplay");
        return Ok((command, stdin));
    }

    // For aplay (Linux) - only works with WAV
    if aplay_available && content_type.contains("wav") {
        debug!("Trying to use aplay for playback");
        let mut command = TokioCommand::new("aplay")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
            
        let stdin = command.stdin.take()
            .ok_or_else(|| "Failed to open aplay stdin".to_string())?;
        debug!("Successfully started aplay");
        return Ok((command, stdin));
    }

    debug!("No suitable player found!");
    Err("No suitable streaming audio player found. Please install mpv, ffplay, or aplay.".into())
}
