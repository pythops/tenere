use serde_json::{Result as JsonResult, Value};

/// Attempts to parse a JSON string safely, handling common issues
/// that might cause "EOF while parsing a string" errors
pub fn parse_json_safely(json_str: &str) -> JsonResult<Value> {
    // Try parsing normally first
    let result = serde_json::from_str::<Value>(json_str);
    
    if result.is_ok() {
        return result;
    }
    
    // If normal parsing fails, try to fix common issues
    
    // 1. Try to fix unescaped quotes in strings
    let mut fixed_json = String::with_capacity(json_str.len());
    let mut in_string = false;
    let mut prev_char = '\0';
    
    for c in json_str.chars() {
        if c == '"' && prev_char != '\\' {
            in_string = !in_string;
        }
        
        if c == '\n' && in_string {
            // Replace newlines inside strings with \n
            fixed_json.push_str("\\n");
        } else if c == '\r' && in_string {
            // Skip or replace carriage returns
            continue;
        } else {
            fixed_json.push(c);
        }
        
        prev_char = c;
    }
    
    // 2. If we ended with an open string, close it
    if in_string {
        fixed_json.push('"');
    }
    
    // Try parsing the fixed JSON
    let result = serde_json::from_str::<Value>(&fixed_json);
    
    if result.is_ok() {
        return result;
    }
    
    // 3. If all else fails, try to add missing closing braces/brackets
    // Count opening and closing braces/brackets
    let open_braces = json_str.chars().filter(|&c| c == '{').count();
    let close_braces = json_str.chars().filter(|&c| c == '}').count();
    let open_brackets = json_str.chars().filter(|&c| c == '[').count();
    let close_brackets = json_str.chars().filter(|&c| c == ']').count();
    
    let mut fixed_json = fixed_json;
    
    // Add missing closing braces
    for _ in 0..(open_braces - close_braces) {
        fixed_json.push('}');
    }
    
    // Add missing closing brackets
    for _ in 0..(open_brackets - close_brackets) {
        fixed_json.push(']');
    }
    
    serde_json::from_str::<Value>(&fixed_json)
}

/// Truncates a string to the specified maximum length,
/// ensuring the result is valid JSON if possible
pub fn truncate_json_safely(json_str: &str, max_length: usize) -> String {
    if json_str.len() <= max_length {
        return json_str.to_string();
    }
    
    // Try to find a good cutoff point that doesn't break JSON structure
    let truncated = &json_str[..max_length];
    
    // Find the last complete JSON object or string
    if let Some(last_brace) = truncated.rfind('}') {
        return json_str[..=last_brace].to_string();
    } else if let Some(last_bracket) = truncated.rfind(']') {
        return json_str[..=last_bracket].to_string();
    } else if let Some(last_quote) = truncated.rfind('"') {
        // Make sure this is an actual closing quote, not an escape sequence
        if truncated.chars().nth(last_quote.saturating_sub(1)) != Some('\\') {
            return json_str[..=last_quote].to_string();
        }
    }
    
    // If we can't find a clean break point, just truncate
    truncated.to_string()
}
