mod command_handler;
mod event_handler;

use std::sync::Arc;

pub use event_handler::EventHandler;
use hex_common::config;
use hex_database::{DatabaseState, HexDatabase};

use hex_discord::twilight_gateway::{
    stream::{self, ShardEventStream},
    Config, Intents,
};

use hex_framework::{watcher::Watcher, HexClient};
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() {
    dotenv::dotenv().unwrap();

    let discord_token = std::env::var(if config::DEBUG {
        "DEBUG_DISCORD_TOKEN"
    } else {
        "DISCORD_TOKEN"
    })
    .expect("expected a valid Discord token");

    let intents = Intents::GUILD_MESSAGES
        | Intents::MESSAGE_CONTENT
        | Intents::GUILD_MEMBERS
        | Intents::GUILDS;
    let config = Config::new(discord_token.clone(), intents);

    let database = Arc::new(
        HexDatabase::new(if config::DEBUG {
            DatabaseState::Debug
        } else {
            DatabaseState::Release
        })
        .await,
    );
    let client = Arc::new(HexClient::new(discord_token).await.unwrap());
    let watcher = Arc::new(Watcher::new());

    if config::DEBUG {
        println!("=== DEBUG ===");
    }

    // Load a single shard
    let mut shards =
        stream::create_range(0..1, 1, config, |_, builder| builder.build()).collect::<Vec<_>>();

    let mut stream = ShardEventStream::new(shards.iter_mut());

    while let Some((_shard, event)) = stream.next().await {
        let event = match event {
            std::result::Result::Ok(event) => event,
            Err(source) => {
                if source.is_fatal() {
                    eprintln!("FATAL ERROR: {:?}", source);
                    break;
                }

                continue;
            }
        };

        let event_handler = EventHandler::new(client.clone(), watcher.clone(), database.clone());
        tokio::spawn(event_handler.handle(event));
    }
}
