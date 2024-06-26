use std::{collections::HashMap, sync::Arc};

use crate::{Command, HexClient};

pub struct Framework {
    pub client: Arc<HexClient>,
    pub commands: HashMap<String, Box<(dyn Command + Send + Sync)>>,
}
