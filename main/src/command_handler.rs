use std::sync::Arc;

use hex_commands::COMMANDS;
use hex_common::config;
use hex_database::HexDatabase;
use hex_discord::{
    twilight_http::client::InteractionClient,
    twilight_model::{
        gateway::payload::incoming::InteractionCreate,
        id::{
            marker::{ApplicationMarker, GuildMarker},
            Id,
        },
    },
    ApiCommand, InteractionData,
};
use hex_framework::{watcher::Watcher, CommandBuilder, CommandContext, HexClient};

pub async fn execute_command(
    interaction: Box<InteractionCreate>,
    client: Arc<HexClient>,
    watcher: Arc<Watcher>,
    database: Arc<HexDatabase>,
) -> anyhow::Result<()> {
    let data = interaction
        .data
        .clone()
        .and_then(|d| match d {
            InteractionData::ApplicationCommand(data) => Some(data),
            _ => None,
        })
        .ok_or(anyhow::anyhow!("Data not found"))?;

    let mut ctx = CommandContext::new(
        client.clone(),
        Box::new(interaction.0),
        watcher,
        database,
        data.options,
    );
    if ctx.interaction.is_dm() {
        ctx.reply("você só pode usar Hex em um servidor.").await?;
        return Ok(());
    }
    let command = COMMANDS
        .get(data.name.as_str())
        .ok_or(anyhow::anyhow!("Command not found"))?;

    let result = command.run(ctx).await;
    if result.is_err() {
        eprintln!("{}", result.unwrap_err());
    }

    Ok(())
}

pub async fn register_commands(application_id: Id<ApplicationMarker>, client: Arc<HexClient>) {
    let commands: Vec<CommandBuilder> = {
        let mut commands = Vec::new();
        for (_, command) in COMMANDS.iter() {
            commands.push(command.build_command(application_id));
        }

        commands
    };

    let guild_id = match config::DEBUG {
        true => Some(Id::new(config::DEBUG_GUILD_ID)),
        false => None,
    };

    register_http_commands(
        guild_id,
        commands
            .into_iter()
            .map(|mut c| {
                if let Some(guild_id) = guild_id {
                    c = c.clone().set_guild_id(guild_id);
                }

                let build = c.build();
                println!(
                    "Registering command {}{}",
                    build.name,
                    if config::DEBUG { " (DEBUG)" } else { "" }
                );

                build
            })
            .collect::<Vec<ApiCommand>>()
            .as_slice(),
        client.http.interaction(application_id),
    )
    .await;
}

async fn register_http_commands<'a>(
    guild_id: Option<Id<GuildMarker>>,
    commands: &[ApiCommand],
    interaction: InteractionClient<'a>,
) {
    match guild_id {
        Some(guild_id) => {
            interaction
                .set_guild_commands(guild_id, commands)
                .await
                .expect("Failed to register guild commands");
        }
        _ => {
            interaction
                .set_global_commands(commands)
                .await
                .expect("Failed to register global commands");
        }
    };
}
