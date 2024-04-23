pub mod common;
mod member_commands;
mod member_model;

use std::sync::Arc;

use mongodb::{Client, Database};

use member_commands::MemberCommands;
pub use mongodb::bson;

#[derive(Debug, Clone)]
pub enum DatabaseState {
    Debug,
    Release,
}

#[derive(Debug, Clone)]
pub struct HexDatabase {
    /* MongoDB's Client uses Arc internally */
    client: Client,
    state: Arc<DatabaseState>,
}

impl HexDatabase {
    pub async fn new(state: DatabaseState) -> HexDatabase {
        let uri = std::env::var("DATABASE_URI").unwrap();

        let client = Client::with_uri_str(&uri).await.unwrap();

        HexDatabase {
            client,
            state: Arc::new(state),
        }
    }

    pub fn db(&self) -> Database {
        self.client.database(match *self.state {
            DatabaseState::Debug => "hex_debug",
            DatabaseState::Release => "hex_release",
        })
    }

    pub fn members(&self) -> MemberCommands {
        let collection = self.db().collection("members");
        MemberCommands::new(collection, self.clone())
    }
}
