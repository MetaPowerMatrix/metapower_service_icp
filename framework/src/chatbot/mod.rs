use serde::{Deserialize, Serialize};

use crate::ActionInfo;

pub mod gemini;
pub mod langchain;
pub mod openai;

#[derive(Serialize, Deserialize, Clone)]
pub struct GptParam {
    pub engine: String,
    pub max_tokens: i32,
    pub temperature: f64,
    pub top_p: f64,
    pub stream: bool,
    pub frequency_penalty: f64,
    pub presence_penalty: f64,
    pub stop: Option<Vec<String>>,
}
pub trait ChatBot {
    
}