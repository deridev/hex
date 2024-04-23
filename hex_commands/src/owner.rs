use hex_database::{bson::doc, common::DatabaseDateTime};

use crate::prelude::*;

#[command("Comando restrito")]
pub async fn owner(mut ctx: CommandContext) -> anyhow::Result<()> {
    let author = ctx.author().await?;
    let guild_id = ctx.guild_id()?;
    if author.id.to_string() != "518830049949122571" {
        return Ok(());
    }

    let confirmation = ctx
        .helper()
        .create_confirmation(author.id, false, "você quer mesmo resetar o servidor?")
        .await?;
    if !confirmation {
        return Ok(());
    }

    ctx.send("Resetando... Isso pode levar alguns segundos.")
        .await?;

    let mut all_users = ctx
        .db()
        .users()
        .collection
        .find(doc! { "guild_id": guild_id.to_string() }, None)
        .await?;

    while all_users.advance().await? {
        let mut user = all_users.deserialize_current()?;
        user.memes = vec![];
        user.daily_cooldown = DatabaseDateTime::zeroed();
        user.meme_collected_cooldown = DatabaseDateTime::zeroed();
        user.awaiting_cooldown_notification = true;

        ctx.db().users().save(user).await?;
    }

    ctx.send("Resetado com sucesso.").await?;

    Ok(())
}
