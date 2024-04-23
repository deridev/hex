use std::sync::Arc;

use crate::command_handler;
use hex_database::HexDatabase;
use hex_discord::{
    twilight_gateway::Event,
    twilight_model::gateway::payload::incoming::{InteractionCreate, MessageCreate, Ready},
};
use hex_framework::{watcher::Watcher, HexClient};

pub struct EventHandler {
    client: Arc<HexClient>,
    watcher: Arc<Watcher>,
    database: Arc<HexDatabase>,
}

impl EventHandler {
    pub fn new(client: Arc<HexClient>, watcher: Arc<Watcher>, database: Arc<HexDatabase>) -> Self {
        Self {
            client,
            watcher,
            database,
        }
    }

    pub async fn handle(self, event: Event) {
        self.watcher.process(&event);

        match event {
            Event::Ready(ready) => {
                self.ready(ready).await.ok();
            }
            Event::MessageCreate(message_create) => {
                self.message_create(message_create).await.ok();
            }
            Event::InteractionCreate(interaction_create) => {
                self.interaction_create(interaction_create).await.ok();
            }
            _ => {}
        };
    }

    pub async fn ready(self, ready: Box<Ready>) -> anyhow::Result<()> {
        let current_user = self.client.current_user().await?;
        println!("{} is ready!", current_user.name);

        command_handler::register_commands(ready.application.id, self.client.clone()).await;

        Ok(())
    }

    pub async fn interaction_create(
        self,
        interaction: Box<InteractionCreate>,
    ) -> anyhow::Result<()> {
        command_handler::execute_command(interaction, self.client, self.watcher, self.database)
            .await
    }

    pub async fn message_create(self, _message: Box<MessageCreate>) -> anyhow::Result<()> {
        Ok(())
    }
}
