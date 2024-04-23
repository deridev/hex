use hex_discord::{
    twilight_model::{
        id::{marker::UserMarker, Id},
        user::{CurrentUser, User},
    },
    DiscordHttpClient,
};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct HexClient {
    pub http: Arc<DiscordHttpClient>,
    pub user_id: Id<UserMarker>,
}

impl HexClient {
    pub async fn new(token: String) -> anyhow::Result<Self> {
        let http = Arc::new(DiscordHttpClient::new(token));

        let user = http.current_user().await?.model().await?;

        Ok(Self {
            http,
            user_id: user.id,
        })
    }

    pub async fn current_user(&self) -> anyhow::Result<CurrentUser> {
        Ok(self.http.current_user().await?.model().await?)
    }

    pub async fn get_user(&self, id: Id<UserMarker>) -> anyhow::Result<User> {
        Ok(self.http.user(id).await?.model().await?)
    }
}
