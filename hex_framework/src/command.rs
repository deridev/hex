use crate::{CommandBuilder, CommandContext};
use hex_discord::twilight_model::id::{marker::ApplicationMarker, Id};

pub struct CommandConfig;

#[async_trait::async_trait]
pub trait Command {
    fn command_config(&self) -> CommandConfig;
    fn build_command(&self, application_id: Id<ApplicationMarker>) -> CommandBuilder;
    async fn run(&self, ctx: CommandContext) -> anyhow::Result<()>;
}
