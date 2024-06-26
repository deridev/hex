use async_trait::async_trait;
use hex_common::load_image_from_url;
use serde::{Deserialize, Serialize};

use crate::{
    brain::{Brain, BrainParameters},
    common::*,
    util::remove_italic_actions,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ClaudeImage {
    #[serde(rename = "type")]
    pub ty: String,
    pub media_type: String,
    pub data: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ClaudeContent {
    #[serde(rename = "type")]
    pub ty: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<ClaudeImage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ClaudeChatMessage {
    pub role: String,
    pub content: Vec<ClaudeContent>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
pub struct ClaudeChatResponse {
    pub content: Vec<ClaudeContent>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ClaudeRequest {
    pub model: String,
    pub system: String,
    pub max_tokens: usize,
    pub messages: Vec<ClaudeChatMessage>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClaudeBrain;

#[async_trait]
impl Brain for ClaudeBrain {
    fn api_key(&self, _debug: bool) -> String {
        std::env::var("CLAUDE_API_KEY").expect("Expected a valid Claude API key")
    }

    fn default_parameters(&self) -> BrainParameters {
        BrainParameters {
            debug: true,
            model: "claude-3-haiku-20240307".to_string(),
            max_tokens: 400,
            system_prompt: String::new(),
            strip_italic_actions: true,
        }
    }

    async fn prompt_chat(
        &self,
        params: BrainParameters,
        mut messages: Vec<ChatMessage>,
    ) -> anyhow::Result<ChatResponse> {
        let mut claude_messages = Vec::with_capacity(messages.len());
        let len = messages.len();

        while messages.first().is_some_and(|m| m.role != Role::User) {
            messages.remove(0);
        }

        for (index, message) in messages.iter().enumerate() {
            let mut contents = vec![];
            let mut has_image = false;
            if let Some(image_url) = &message.image_url {
                if index == len - 1 && message.role == Role::User {
                    let image = load_image_from_url(image_url).await?;
                    has_image = true;
                    contents.push(ClaudeContent {
                        ty: "image".to_string(),
                        source: Some(ClaudeImage {
                            ty: "base64".to_string(),
                            media_type: image.mime_type,
                            data: image.data,
                        }),
                        text: None,
                    });
                }
            }

            contents.push(ClaudeContent {
                ty: "text".to_string(),
                text: Some(if has_image {
                    format!("{} [IMAGEM]", message.content)
                } else {
                    message.content.clone()
                }),
                source: None,
            });

            let claude_message = ClaudeChatMessage {
                role: match message.role {
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                },
                content: contents,
            };

            claude_messages.push(claude_message);
        }

        let request = ClaudeRequest {
            model: params.model,
            max_tokens: params.max_tokens,
            messages: claude_messages,
            system: params.system_prompt.to_owned(),
        };

        let response = self
            .http_client()
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", self.api_key(params.debug))
            .header("content-type", "application/json")
            .header("anthropic-version", "2023-06-01")
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let mut text = response.text().await?;
            text.truncate(1800);
            return Err(anyhow::anyhow!("Status code: {}\n{:?}", status, text));
        }

        let mut response: ClaudeChatResponse = response.json().await?;

        if params.strip_italic_actions {
            response.content.iter_mut().for_each(|reply| {
                reply.text = reply.text.as_ref().map(|t| remove_italic_actions(t));
            });
        }

        response.content.iter_mut().for_each(|reply| {
            let Some(content) = &mut reply.text else {
                return;
            };

            let text = content.to_uppercase().trim().to_owned();
            if text.contains("{AWAIT}") {
                *content = "{AWAIT}".to_string();
            } else if text.contains("{EXIT}") {
                *content = "{EXIT}".to_string();
            }
        });

        Ok(ChatResponse {
            message: ChatMessage {
                role: Role::Assistant,
                content: response
                    .content
                    .first()
                    .map(|r| r.text.clone().unwrap_or_default())
                    .unwrap_or_default()
                    .trim()
                    .to_owned(),
                image_url: None,
            },
        })
    }
}
