use std;
use std::collections::HashMap;

pub trait LLM {
    fn new(&self) -> Box<dyn LLM>;
    fn ask(&self, chat: Vec<HashMap<String, String>>)
        -> Result<String, Box<dyn std::error::Error>>;
}
