use reqwest::header::HeaderMap;
use serde_json::{json, Value};
use std;

pub async fn ask_gpt(user_input: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let url = "https://api.openai.com/v1/chat/completions";
    let token =
        std::env::var("OPENAI_API_KEY").expect("Can not find the OPENAI_API_KEY env variable");

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert(
        "Authorization",
        format!("Bearer {}", token).parse().unwrap(),
    );

    let body: Value = json!({
        "model": "gpt-3.5-turbo",
        "messages": [
            {"role": "system", "content": "You are a helpful assistant."},
            {"role": "user", "content": user_input},
        ]
    });

    let response = client.post(url).headers(headers).json(&body).send().await?;

    let response_body: Value = response.json().await?;

    // TODO: better way to get the answer
    let answer = response_body["choices"][0]["message"]["content"].clone();

    Ok(answer.to_string())
}
