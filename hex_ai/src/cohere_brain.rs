use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    brain::{Brain, BrainParameters},
    common::*,
    util::remove_italic_actions,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MessageHistory {
    pub role: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct CohereChatRequest {
    pub message: String,
    pub model: String,
    pub chat_history: Vec<MessageHistory>,
    #[serde(rename = "preamble")]
    pub system_prompt: String,
    pub max_tokens: usize,
    pub temperature: f64,
    pub frequency_penalty: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CohereChatResponse {
    pub text: String,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CohereBrain;

#[async_trait]
impl Brain for CohereBrain {
    fn api_key(&self, debug: bool) -> String {
        if debug {
            std::env::var("DEBUG_COHERE_API_KEY").expect("Expected a valid DEBUG Cohere API key")
        } else {
            std::env::var("COHERE_API_KEY").expect("Expected a valid Cohere API key")
        }
    }

    fn default_parameters(&self) -> BrainParameters {
        BrainParameters {
            debug: true,
            model: "command-r".to_string(),
            max_tokens: 300,
            system_prompt: String::new(),
            strip_italic_actions: true,
        }
    }

    async fn prompt_chat(
        &self,
        mut params: BrainParameters,
        mut messages: Vec<ChatMessage>,
    ) -> anyhow::Result<ChatResponse> {
        let last_message = messages.pop();

        if params.max_tokens < 750 {
            params.max_tokens = 750;
        }
        let request = CohereChatRequest {
            model: params.model,
            max_tokens: params.max_tokens,
            message: last_message.map(|m| m.content.clone()).unwrap_or_default(),
            system_prompt: params.system_prompt.to_owned(),
            temperature: 0.6,
            chat_history: messages
                .iter()
                .map(|m| MessageHistory {
                    role: match m.role {
                        Role::User => "USER".to_string(),
                        Role::Assistant => "CHATBOT".to_string(),
                    },
                    message: m.content.clone(),
                })
                .collect(),
            frequency_penalty: 0.15,
        };

        let response = self
            .http_client()
            .post("https://api.cohere.ai/v1/chat")
            .header("accept", "application/json")
            .header("content-type", "application/json")
            .header(
                "Authorization",
                format!("bearer {}", self.api_key(params.debug)),
            )
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let mut text = response.text().await?;
            text.truncate(1800);
            return Err(anyhow::anyhow!("Status code: {}\n{:?}", status, text));
        }

        let mut response: CohereChatResponse = response.json().await?;

        if params.strip_italic_actions {
            response.text = remove_italic_actions(&response.text);
        }

        let content = response.text.to_uppercase().trim().to_owned();
        if content.contains("{AWAIT}") {
            response.text = "{AWAIT}".to_string();
        } else if content.contains("{EXIT}") {
            response.text = "{EXIT}".to_string();
        }

        if content.contains('<') && content.contains('>') && content.contains('@') {
            response.text = remove_after_lt(&response.text).trim().to_string();
        }

        Ok(ChatResponse {
            message: ChatMessage {
                role: Role::Assistant,
                content: response.text,
                image_url: None,
            },
        })
    }
}

fn remove_after_lt(input: &str) -> &str {
    if let Some(index) = input.find('<') {
        &input[..index]
    } else {
        input
    }
}
