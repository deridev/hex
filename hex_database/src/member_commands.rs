use std::hash::Hash;

use bson::{doc, oid::ObjectId, Document};
use hex_common::Cache;
use mongodb::Collection;
use once_cell::sync::Lazy;

use crate::{common::*, member_model::MemberModel, *};

static CACHE_ID: Lazy<Cache<ObjectId, MemberModel>> = Lazy::new(|| Cache::new(1000));
static CACHE_GUILD_MEMBER_ID: Lazy<Cache<(String, String), MemberModel>> =
    Lazy::new(|| Cache::new(1000));

#[allow(unused)]
pub struct MemberCommands {
    pub collection: Collection<MemberModel>,
    db: HexDatabase,
}

impl MemberCommands {
    pub const fn new(collection: Collection<MemberModel>, db: HexDatabase) -> Self {
        Self { collection, db }
    }

    pub async fn save(&self, member: MemberModel) -> anyhow::Result<()> {
        CACHE_ID.remove(&member.id);
        CACHE_GUILD_MEMBER_ID.remove(&(member.guild_id.clone(), member.user_id.clone()));

        self.collection
            .replace_one(query_by_id(member.id), &member, None)
            .await?;
        Ok(())
    }

    async fn get<K: Eq + Hash>(
        &self,
        cache: &Lazy<Cache<K, MemberModel>>,
        key: K,
        query: Document,
    ) -> anyhow::Result<Option<MemberModel>> {
        let cached = cache.get_cloned(&key);
        match cached {
            Some(model) => Ok(Some(model)),
            None => {
                let Some(model) = self.collection.find_one(query, None).await? else {
                    return Ok(None);
                };

                cache.insert(key, model.clone());
                Ok(Some(model))
            }
        }
    }

    pub async fn get_by_id(&self, id: ObjectId) -> anyhow::Result<Option<MemberModel>> {
        self.get(&CACHE_ID, id, query_by_id(id)).await
    }

    pub async fn get_member(&self, user_id: &str, guild_id: &str) -> anyhow::Result<MemberModel> {
        let query = doc! {
            "user_id": user_id,
            "guild_id": guild_id
        };

        let data = self
            .get(
                &CACHE_GUILD_MEMBER_ID,
                (guild_id.to_string(), user_id.to_string()),
                query,
            )
            .await?;

        match data {
            Some(data) => Ok(data),
            None => {
                let model = MemberModel::new(user_id.to_string(), guild_id.to_string());

                self.collection.insert_one(&model, None).await?;

                Ok(model)
            }
        }
    }
}
