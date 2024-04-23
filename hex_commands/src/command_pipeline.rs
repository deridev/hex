use std::{collections::HashSet, fmt::Display, time::Duration};

use anyhow::bail;
use hex_ai::{
    common::{BrainKind, ChatMessage, Role},
    util::get_brain,
};
use hex_discord::{
    twilight_http::request::AuditLogReason,
    twilight_model::{channel::ChannelType, id::Id, user::User},
    UserExtension,
};
use hex_framework::CommandContext;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum InputObject {
    Suggestion(UserContentData),
    Message(UserContentData),
    CommandResponse(CommandResponse),
    SystemError(String),
}

impl InputObject {
    pub fn is_user_input(&self) -> bool {
        matches!(self, Self::Suggestion(..) | Self::Message(..))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct UserContentData {
    pub lang: String,
    pub user: UserIdentifier,
    pub content: String,
    pub channel: ChannelRepresentation,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct UserIdentifier {
    pub name: String,
    pub uid: u64,
    pub karma: i64,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandResponse {
    pub command_type: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ChannelRepresentation {
    pub id: u64,
    pub name: String,
    pub topic: String,
    pub kind: String,
    pub category: Option<String>,
    pub message_count: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandObject {
    pub reasoning: String,
    pub cmd: CommandType,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Module {
    Channels,
    Moderation,
    Members,
}

impl Module {
    pub const LIST: &'static [Self] = &[Self::Channels, Self::Moderation, Self::Members];

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "channels" => Some(Self::Channels),
            "moderation" => Some(Self::Moderation),
            "members" => Some(Self::Members),
            _ => None,
        }
    }

    pub fn prompt(&self) -> &'static str {
        match self {
            Self::Channels => include_str!("module_channel.txt"),
            Self::Moderation => include_str!("module_moderation.txt"),
            Self::Members => include_str!("module_members.txt"),
        }
    }
}

impl Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Channels => f.write_str("channels"),
            Self::Moderation => f.write_str("moderation"),
            Self::Members => f.write_str("members"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum CommandType {
    ImportModule(ImportModuleData),
    AddKarma(UpdateKarmaData),
    RemoveKarma(UpdateKarmaData),
    GetMemberData(GetMemberData),
    GetAllMembersData(
        #[serde(default = "Option::default", skip_serializing_if = "Option::is_none")] Option<()>,
    ),
    AddNote(AddNoteData),
    GetChannelList(
        #[serde(default = "Option::default", skip_serializing_if = "Option::is_none")] Option<()>,
    ),
    GetChannel(GetChannelData),
    CreateChannel(CreateChannelData),
    DeleteChannel(DeleteChannelData),
    EditChannel(EditChannelData),
    SendReply(SendReplyData),
    KickMember(PunishMemberData),
    BanMember(PunishMemberData),
    Stop(#[serde(default = "Option::default", skip_serializing_if = "Option::is_none")] Option<()>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ImportModuleData {
    pub module_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct UpdateKarmaData {
    pub user_id: u64,
    pub amount: i64,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct GetMemberData {
    pub namefilter: Option<String>,
    pub idfilter: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct AddNoteData {
    pub member_id: u64,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct GetChannelData {
    pub namefilter: Option<String>,
    pub idfilter: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CreateChannelData {
    pub channel_name: String,
    pub topic: String,
    pub category: Option<String>,
    pub is_category: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DeleteChannelData {
    pub channel_id: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EditChannelData {
    pub channel_id: u64,
    pub name: String,
    pub topic: String,
    pub category: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SendReplyData {
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PunishMemberData {
    pub user_id: u64,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MemberData {
    pub id: u64,
    pub display_name: String,
    pub username: String,
    pub karma: i64,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PipelineObject {
    Input(InputObject),
    Command(CommandObject),
    MalformmedCommand(String),
}

#[derive(Debug, Clone)]
pub struct AiCommandPipeline {
    pub ctx: CommandContext,
    pub author: User,
    pub history: Vec<PipelineObject>,
    pub brain: BrainKind,
    pub error_counter: u32,
    pub active: bool,
    pub imported_modules: HashSet<Module>,
}

impl AiCommandPipeline {
    pub async fn new(ctx: CommandContext) -> anyhow::Result<Self> {
        let author = ctx.author().await?;

        Ok(Self {
            ctx,
            author,
            history: vec![],
            brain: BrainKind::ClaudeHaiku,
            error_counter: 0,
            active: true,
            imported_modules: HashSet::new(),
        })
    }

    pub async fn execute_error(&mut self, error: String) -> anyhow::Result<CommandObject> {
        self.error_counter += 1;
        if self.error_counter > 3 {
            self.active = false;
            bail!(error);
        }

        self.execute_input(InputObject::SystemError(error)).await
    }

    pub async fn execute_input(&mut self, input: InputObject) -> anyhow::Result<CommandObject> {
        tokio::time::sleep(Duration::from_millis(1500)).await;
        self.history.push(PipelineObject::Input(input));

        let brain = get_brain(self.brain);

        let mut parameters = brain.default_parameters();
        parameters.max_tokens = 1024;
        parameters.system_prompt = include_str!("pipeline_prompt.txt").to_string();

        loop {
            let mut messages = vec![];

            for o in self.history.iter() {
                let message = match o {
                    PipelineObject::Input(input) => {
                        let is_user_input = input.is_user_input();
                        let json = serde_json::to_string_pretty(&input)?;

                        ChatMessage {
                            content: if is_user_input {
                                format!("<User Input Object. This was sent by a REAL user in a text-channel. The user is unable to interact again. Respond to this with a command object. Maintain the user language.>\n{json}")
                            } else {
                                format!("<Command Pipeline System Object. This was sent by the system. The user cannot see this. Respond to this with a command object.>\n{json}")
                            },
                            image_url: None,
                            role: Role::User,
                        }
                    }
                    PipelineObject::Command(cmd) => {
                        let json = serde_json::to_string_pretty(&cmd)?;

                        ChatMessage {
                            content: json,
                            image_url: None,
                            role: Role::Assistant,
                        }
                    }
                    PipelineObject::MalformmedCommand(cmd) => ChatMessage {
                        content: cmd.to_owned(),
                        image_url: None,
                        role: Role::Assistant,
                    },
                };

                messages.push(message);
            }

            let response = brain.prompt_chat(parameters.clone(), messages).await?;
            let mut content = response.message.content;
            content = content
                .replace("\"data\": {}", "\"data\": null")
                .replace("\"data\": undefined", "\"data\": null");

            println!("\x1b[33m{}\x1b[0m", content);
            let output: Result<CommandObject, _> = serde_json::from_str(&content);

            match output {
                Ok(cmd) => {
                    self.history.push(PipelineObject::Command(cmd.clone()));
                    while self.history.len() > 10 {
                        self.history.remove(0);
                    }

                    break Ok(cmd);
                }
                Err(e) => {
                    self.error_counter += 1;
                    if self.error_counter > 3 {
                        self.active = false;
                        bail!(e);
                    }

                    println!("Malformed command:\n{e}\n{}\n\n", content);

                    self.history
                        .push(PipelineObject::MalformmedCommand(content.clone()));

                    self.history.push(PipelineObject::Input(
                        InputObject::SystemError(format!("<JSON Error>\nCannot parse your JSON command. Rewrite it again. Remember: only ONE command per message, no markdown and text other than JSON, omitting {{}} and undefined fields.\nError: {}", e))  
                    ));

                    while self.history.len() > 10 {
                        self.history.remove(0);
                    }

                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
            }
        }
    }

    pub async fn execute(&mut self, mut command: CommandObject) -> anyhow::Result<()> {
        let http = self.ctx.client.http.clone();
        let guild_id = self.ctx.interaction.guild_id.unwrap_or(Id::new(1234567));
        let db = self.ctx.db();

        macro_rules! all_channels {
            () => {{
                let channels = match http.guild_channels(guild_id).await {
                    Ok(channels) => channels,
                    Err(e) => {
                        command = self
                            .execute_error(format!("Error while getting all guild channels: {e}"))
                            .await?;
                        continue;
                    }
                };

                let channels = match channels.models().await {
                    Ok(channels) => channels,
                    Err(e) => {
                        command = self
                            .execute_error(format!("Error while getting all guild channels: {e}"))
                            .await?;
                        continue;
                    }
                };

                channels
            }};
        }

        while self.active {
            println!(
                "-> CMD: {:?}\nReasoning: \"{}\"\n",
                command.cmd, command.reasoning
            );
            match &command.cmd {
                CommandType::ImportModule(data) => {
                    let module = match Module::parse(&data.module_name) {
                        Some(module) => module,
                        None => {
                            command = self
                                .execute_error(format!(
                                    "Invalid module name: {}. Valid modules are: {}",
                                    data.module_name,
                                    Module::LIST
                                        .iter()
                                        .map(|m| m.to_string())
                                        .collect::<Vec<_>>()
                                        .join(", ")
                                ))
                                .await?;
                            continue;
                        }
                    };

                    if self.imported_modules.contains(&module) {
                        command = self
                            .execute_error(format!("Module is already imported: {}", module))
                            .await?;
                        continue;
                    }

                    self.imported_modules.insert(module);
                    command = self
                        .execute_input(InputObject::CommandResponse(CommandResponse {
                            command_type: "ImportModule".to_string(),
                            data: Value::String(format!(
                                "<Successfuly imported module {}. Module content below>\n{}",
                                module,
                                module.prompt()
                            )),
                        }))
                        .await?;
                }
                CommandType::AddKarma(data) => {
                    let user_id = Id::new(data.user_id);
                    let member = match http.guild_member(guild_id, user_id).await {
                        Ok(member) => member,
                        Err(e) => {
                            command = self
                                .execute_error(format!("Error while getting member: {e}"))
                                .await?;
                            continue;
                        }
                    };

                    let member = match member.model().await {
                        Ok(member) => member,
                        Err(e) => {
                            command = self
                                .execute_error(format!("Error while getting member: {e}"))
                                .await?;
                            continue;
                        }
                    };

                    let mut member = db
                        .members()
                        .get_member(&member.user.id.to_string(), &guild_id.to_string())
                        .await?;
                    member.karma += data.amount.abs();
                    db.members().save(member).await?;

                    command = self
                        .execute_input(InputObject::CommandResponse(CommandResponse {
                            command_type: "AddKarma".to_string(),
                            data: Value::String("Success".to_string()),
                        }))
                        .await?;
                }
                CommandType::GetAllMembersData(..) => {
                    let members = match http.guild_members(guild_id).limit(1000)?.await {
                        Ok(members) => members.models().await?,
                        Err(e) => {
                            command = self
                                .execute_error(format!("Error while getting members: {e}"))
                                .await?;
                            continue;
                        }
                    };

                    let mut members_list = vec![];
                    for member in members {
                        let data = db
                            .members()
                            .get_member(&member.user.id.to_string(), &guild_id.to_string())
                            .await?;
                        members_list.push((member, data));
                    }

                    let members_list = members_list
                        .iter()
                        .map(|(m, d)| MemberData {
                            id: m.user.id.get(),
                            display_name: m.user.display_name().to_string(),
                            username: m.user.name.to_string(),
                            karma: d.karma,
                            notes: d.notes.clone(),
                        })
                        .collect::<Vec<_>>();

                    command = self
                        .execute_input(InputObject::CommandResponse(CommandResponse {
                            command_type: "GetAllMembersData".to_string(),
                            data: Value::String(serde_json::to_string_pretty(&members_list)?),
                        }))
                        .await?;
                }
                CommandType::RemoveKarma(data) => {
                    let user_id = Id::new(data.user_id);
                    let member = match http.guild_member(guild_id, user_id).await {
                        Ok(member) => member,
                        Err(e) => {
                            command = self
                                .execute_error(format!("Error while getting member: {e}"))
                                .await?;
                            continue;
                        }
                    };

                    let member = match member.model().await {
                        Ok(member) => member,
                        Err(e) => {
                            command = self
                                .execute_error(format!("Error while getting member: {e}"))
                                .await?;
                            continue;
                        }
                    };

                    let mut member = db
                        .members()
                        .get_member(&member.user.id.to_string(), &guild_id.to_string())
                        .await?;
                    member.karma -= data.amount.abs();
                    db.members().save(member).await?;

                    command = self
                        .execute_input(InputObject::CommandResponse(CommandResponse {
                            command_type: "RemoveKarma".to_string(),
                            data: Value::String("Success".to_string()),
                        }))
                        .await?;
                }
                CommandType::AddNote(data) => {
                    let user_id = Id::new(data.member_id);
                    let member = match http.guild_member(guild_id, user_id).await {
                        Ok(member) => member,
                        Err(e) => {
                            command = self
                                .execute_error(format!("Error while getting member: {e}"))
                                .await?;
                            continue;
                        }
                    };

                    let member = match member.model().await {
                        Ok(member) => member,
                        Err(e) => {
                            command = self
                                .execute_error(format!("Error while getting member: {e}"))
                                .await?;
                            continue;
                        }
                    };

                    let mut member = db
                        .members()
                        .get_member(&member.user.id.to_string(), &guild_id.to_string())
                        .await?;
                    member.notes.push(data.note.clone());
                    db.members().save(member).await?;

                    command = self
                        .execute_input(InputObject::CommandResponse(CommandResponse {
                            command_type: "AddNote".to_string(),
                            data: Value::String("Success".to_string()),
                        }))
                        .await?;
                }
                CommandType::GetMemberData(data) => {
                    let idfilter = data.idfilter;
                    let namefilter = data.namefilter.clone();

                    if idfilter.is_none() && namefilter.is_none() {
                        command = self
                            .execute_error("You must provide at least one filter, otherwise this command is not useful.".to_string())
                            .await?;
                        continue;
                    }

                    let members = match http.guild_members(guild_id).limit(1000)?.await {
                        Ok(members) => members.models().await?,
                        Err(e) => {
                            command = self
                                .execute_error(format!("Error while getting members: {e}"))
                                .await?;
                            continue;
                        }
                    };

                    println!("{:?}", members);

                    let member = members.iter().find(|m| {
                        let idfilter = match idfilter {
                            None => true,
                            Some(idfilter) => m.user.id.get() == idfilter,
                        };

                        let namefilter = match &namefilter {
                            None => true,
                            Some(namefilter) => {
                                let filter = namefilter.to_ascii_lowercase();
                                m.user.name.to_ascii_lowercase().contains(&filter)
                                    || m.user.display_name().to_ascii_lowercase().contains(&filter)
                            }
                        };

                        idfilter && namefilter
                    });

                    match member {
                        Some(member) => {
                            let data = db
                                .members()
                                .get_member(&member.user.id.to_string(), &guild_id.to_string())
                                .await?;

                            let member = MemberData {
                                id: member.user.id.get(),
                                display_name: member.user.display_name().to_string(),
                                username: member.user.name.to_string(),
                                karma: data.karma,
                                notes: data.notes,
                            };

                            command = self
                                .execute_input(InputObject::CommandResponse(CommandResponse {
                                    command_type: "GetMemberData".to_string(),
                                    data: Value::String(format!(
                                        "<Matched one member entry for the given filters>\n{}",
                                        serde_json::to_string_pretty(&member).unwrap_or_default()
                                    )),
                                }))
                                .await?;
                        }
                        None => {
                            command = self
                                .execute_input(InputObject::CommandResponse(CommandResponse {
                                    command_type: "GetMemberData".to_string(),
                                    data: Value::String("Member not found".to_string()),
                                }))
                                .await?;
                        }
                    }
                }
                CommandType::CreateChannel(data) => {
                    let channel = match http.create_guild_channel(guild_id, &data.channel_name) {
                        Ok(channel) => channel,
                        Err(e) => {
                            command = self
                                .execute_error(format!("Error while creating the channel: {e}"))
                                .await?;
                            continue;
                        }
                    };

                    let category_id = if let Some(category) = &data.category {
                        let channels = all_channels!();
                        let category = match channels
                            .iter()
                            .find(|c| c.name.as_ref() == Some(category))
                        {
                            Some(category) => category.clone(),
                            None => match http.create_guild_channel(guild_id, category) {
                                Ok(category) => {
                                    match category.kind(ChannelType::GuildCategory).await {
                                        Ok(category) => category.model().await?,
                                        Err(e) => {
                                            command = self
                                            .execute_error(format!("Error while creating the category for this channel: {e}"))
                                            .await?;
                                            continue;
                                        }
                                    }
                                }
                                Err(e) => {
                                    command = self
                                        .execute_error(format!(
                                            "Error while creating the category for this channel: {e}"
                                        ))
                                        .await?;
                                    continue;
                                }
                            },
                        };

                        Some(category.id)
                    } else {
                        None
                    };

                    match channel.topic(&data.topic) {
                        Ok(mut channel) => {
                            if let Some(category_id) = category_id {
                                channel = channel.parent_id(category_id);
                            } else if data.is_category {
                                channel = channel.kind(ChannelType::GuildCategory);
                            }

                            let message = match channel.await {
                                Ok(channel) => {
                                    let channel = channel.model().await?;
                                    format!("Success - Created the channel {} with ID {}. The parent-category channel ID is: {}", data.channel_name, channel.id, channel.parent_id.map(|id| id.get().to_string()).unwrap_or(String::from("<None>")))
                                }
                                Err(e) => format!("Api Failure. Error: {e}"),
                            };

                            command = self
                                .execute_input(InputObject::CommandResponse(CommandResponse {
                                    command_type: "CreateChannel".to_string(),
                                    data: Value::String(message),
                                }))
                                .await?;

                            continue;
                        }
                        Err(e) => {
                            command = self
                                .execute_input(InputObject::SystemError(format!(
                                    "Error while setting the channel topic: {e}"
                                )))
                                .await?;
                            continue;
                        }
                    }
                }
                CommandType::DeleteChannel(data) => {
                    match http.delete_channel(Id::new(data.channel_id)).await {
                        Ok(..) => {
                            command = self
                                .execute_input(InputObject::CommandResponse(CommandResponse {
                                    command_type: "DeleteChannel".to_string(),
                                    data: Value::String("Success".to_string()),
                                }))
                                .await?;
                            continue;
                        }
                        Err(e) => {
                            command = self
                                .execute_error(format!("Error while deleting the channel: {e}"))
                                .await?;
                            continue;
                        }
                    };
                }
                CommandType::EditChannel(data) => {
                    let data_channel_id = data.channel_id;

                    let channels = all_channels!();
                    let channel = match channels.iter().find(|c| c.id.get() == data_channel_id) {
                        Some(channel) => channel,
                        None => {
                            command = self
                                .execute_error(format!(
                                    "Channel not found with ID {}",
                                    data.channel_id
                                ))
                                .await?;
                            continue;
                        }
                    };

                    let category_id = if let Some(category) = &data.category {
                        let category = match channels.iter().find(|c| {
                            c.kind == ChannelType::GuildCategory
                                && c.name.as_ref() == Some(category)
                        }) {
                            Some(category) => category,
                            None => {
                                command = self
                                    .execute_error(format!(
                                        "Category not found with name {}",
                                        category
                                    ))
                                    .await?;
                                continue;
                            }
                        };

                        Some(category.id)
                    } else {
                        None
                    };

                    match http.update_channel(channel.id).name(&data.name) {
                        Ok(channel) => match channel.topic(&data.topic) {
                            Ok(channel) => {
                                let message = match channel.parent_id(category_id).await {
                                    Ok(_) => "Success".to_string(),
                                    Err(e) => format!("Api Failure. Error: {e}"),
                                };

                                command = self
                                    .execute_input(InputObject::CommandResponse(CommandResponse {
                                        command_type: "EditChannel".to_string(),
                                        data: Value::String(message),
                                    }))
                                    .await?;

                                continue;
                            }
                            Err(e) => {
                                command = self
                                    .execute_error(format!(
                                        "Failed to update channel topic. Error: {e}"
                                    ))
                                    .await?;
                                continue;
                            }
                        },
                        Err(e) => {
                            command = self
                                .execute_error(format!("Failed to update channel name. Error: {e}"))
                                .await?;
                            continue;
                        }
                    }
                }
                CommandType::GetChannel(data) => {
                    let idfilter = data.idfilter;
                    let namefilter = data.namefilter.clone();

                    if idfilter.is_none() && namefilter.is_none() {
                        command = self
                            .execute_error("You must provide at least one filter, otherwise this command will return all channels, which is the GetAllChannels cmd job.".to_string())
                            .await?;
                        continue;
                    }

                    let channels = all_channels!();
                    let channel = match channels.iter().find(|c| {
                        let idfilter = match idfilter {
                            None => true,
                            Some(idfilter) => c.id.get() == idfilter,
                        };

                        let namefilter = match &namefilter {
                            None => true,
                            Some(namefilter) => c
                                .name
                                .as_ref()
                                .is_some_and(|n| n.to_lowercase() == namefilter.to_lowercase()),
                        };

                        idfilter && namefilter
                    }) {
                        Some(channel) => channel,
                        None => {
                            command = self
                                .execute_error("Channel not found with this filter".to_string())
                                .await?;
                            continue;
                        }
                    };

                    let channel = ChannelRepresentation {
                        id: channel.id.get(),
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
                    };

                    command = self
                        .execute_input(InputObject::CommandResponse(CommandResponse {
                            command_type: "GetChannel".to_string(),
                            data: serde_json::to_value(channel).unwrap_or(Value::String(
                                "Failed to serialize the channel. This is propably a bug. Please, tell the user to report this to the developer and stop the command execution."
                                    .to_string(),
                            )),
                        }))
                        .await?;
                }
                CommandType::GetChannelList(..) => {
                    let channels = all_channels!();
                    let channels = channels
                        .iter()
                        .map(|c| ChannelRepresentation {
                            id: c.id.get(),
                            name: c.name.clone().unwrap_or_default(),
                            topic: c.topic.clone().unwrap_or(String::from("<Empty>")),
                            category: c.parent_id.and_then(|id| {
                                channels
                                    .iter()
                                    .find(|c| c.id.get() == id)
                                    .and_then(|c| c.name.clone())
                            }),
                            kind: if c.kind == ChannelType::GuildCategory {
                                "Category".to_string()
                            } else {
                                "Chat".to_string()
                            },
                            message_count: c.message_count,
                        })
                        .collect::<Vec<_>>();

                    command = self
                        .execute_input(InputObject::CommandResponse(CommandResponse {
                            command_type: "GetChannelList".to_string(),
                            data: serde_json::to_value(channels).unwrap_or(Value::String(
                                "Failed to serialize the channels. Explain this error to the user."
                                    .to_string(),
                            )),
                        }))
                        .await?;
                }
                CommandType::SendReply(reply) => {
                    let content = reply.content.clone();

                    self.ctx.send(content).await?;
                    command = self
                        .execute_input(InputObject::CommandResponse(CommandResponse {
                            command_type: "SendReply".to_string(),
                            data: Value::String("Success".to_string()),
                        }))
                        .await?;
                }
                CommandType::KickMember(data) => {
                    let user_id = Id::new(data.user_id);
                    let member = match http.guild_member(guild_id, user_id).await {
                        Ok(member) => member,
                        Err(e) => {
                            command = self
                                .execute_error(format!("Error while getting member: {e}"))
                                .await?;
                            continue;
                        }
                    };

                    let member = match member.model().await {
                        Ok(member) => member,
                        Err(e) => {
                            command = self
                                .execute_error(format!("Error while getting member: {e}"))
                                .await?;
                            continue;
                        }
                    };

                    match http
                        .remove_guild_member(guild_id, member.user.id)
                        .reason(&data.reason)
                    {
                        Ok(kick) => match kick.await {
                            Ok(_) => (),
                            Err(e) => {
                                command = self
                                    .execute_error(format!("Error while kicking member: {e}"))
                                    .await?;
                                continue;
                            }
                        },
                        Err(e) => {
                            command = self
                                .execute_error(format!("Error while kicking member: {e}"))
                                .await?;
                            continue;
                        }
                    }
                }
                CommandType::BanMember(data) => {
                    let user_id = Id::new(data.user_id);
                    let member = match http.guild_member(guild_id, user_id).await {
                        Ok(member) => member,
                        Err(e) => {
                            command = self
                                .execute_error(format!("Error while getting member: {e}"))
                                .await?;
                            continue;
                        }
                    };

                    let member = match member.model().await {
                        Ok(member) => member,
                        Err(e) => {
                            command = self
                                .execute_error(format!("Error while getting member: {e}"))
                                .await?;
                            continue;
                        }
                    };

                    match http
                        .create_ban(guild_id, member.user.id)
                        .reason(&data.reason)
                    {
                        Ok(kick) => match kick.await {
                            Ok(_) => (),
                            Err(e) => {
                                command = self
                                    .execute_error(format!("Error while kicking member: {e}"))
                                    .await?;
                                continue;
                            }
                        },
                        Err(e) => {
                            command = self
                                .execute_error(format!("Error while kicking member: {e}"))
                                .await?;
                            continue;
                        }
                    }
                }
                CommandType::Stop(..) => {
                    self.active = false;
                    break;
                }
            }
        }

        Ok(())
    }
}
