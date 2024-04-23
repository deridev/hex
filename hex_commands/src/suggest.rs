use hex_discord::twilight_model::channel::ChannelType;

use crate::{command_pipeline::*, prelude::*};

#[command("Sugira mudanças e melhorias para o seu servidor atual!")]
#[name("sugerir")]
pub async fn suggest(
    mut ctx: CommandContext,
    #[rename("sugestão")]
    #[description("A sua sugestão")]
    suggestion: String,
) -> anyhow::Result<()> {
    let locale = ctx
        .interaction
        .locale
        .clone()
        .unwrap_or("pt-BR".to_string());
    let channel = match &ctx.interaction.channel {
        Some(channel) => channel.clone(),
        None => return Ok(()),
    };

    let channel_id = channel.id;
    let guild_id = match &channel.guild_id {
        Some(id) => *id,
        None => return Ok(()),
    };

    let author = ctx.author().await?;
    let db = ctx.db();
    ctx.reply("`Executando...`").await?;

    let channels = ctx
        .client
        .http
        .guild_channels(guild_id)
        .await?
        .models()
        .await?;

    let mut pipeline = AiCommandPipeline::new(ctx).await?;

    let author_member = db
        .members()
        .get_member(&author.id.to_string(), &guild_id.to_string())
        .await?;

    let command = pipeline
        .execute_input(InputObject::Suggestion(UserContentData {
            lang: locale,
            user: UserIdentifier {
                name: author.display_name(),
                uid: author.id.get(),
                karma: author_member.karma,
                notes: author_member.notes,
            },
            content: suggestion,
            channel: ChannelRepresentation {
                id: channel_id.get(),
                name: channel.name.clone().unwrap_or_default(),
                topic: channel.topic.clone().unwrap_or(String::from("<Empty>")),
                category: channel.parent_id.and_then(|id| {
                    channels
                        .iter()
                        .find(|c| c.id.get() == id)
                        .and_then(|c| c.name.clone())
                }),
                kind: if channel.kind == ChannelType::GuildCategory {
                    "Category".to_string()
                } else {
                    "Chat".to_string()
                },
                message_count: channel.message_count,
            },
        }))
        .await?;

    pipeline.execute(command).await?;

    Ok(())
}
