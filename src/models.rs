use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize)]
pub(crate) struct ChatCompletionRequest<'a> {
    pub model: &'a str,
    pub messages: &'a [ChatMessage],
}

#[derive(Deserialize)]
pub(crate) struct ChatCompletionResponse {
    pub choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
pub(crate) struct ChatChoice {
    pub message: ChatMessage,
}

#[derive(Deserialize)]
pub(crate) struct ChatInput {
    pub session_id: String,
    pub message: String,
}

#[derive(Serialize)]
pub(crate)  struct ChatOutput {
    pub session_id: String,
    pub reply: String,
}