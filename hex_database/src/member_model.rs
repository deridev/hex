use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct MemberModel {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub user_id: String,
    pub guild_id: String,
    pub karma: i64,
    pub notes: Vec<String>,
}

impl MemberModel {
    pub fn new(user_id: String, guild_id: String) -> Self {
        Self {
            id: ObjectId::new(),
            user_id,
            guild_id,
            karma: 0,
            notes: vec![],
        }
    }

    pub fn push_note(&mut self, note: String) {
        self.notes.push(note);

        while self.notes.len() > 8 {
            self.notes.remove(0);
        }
    }
}
