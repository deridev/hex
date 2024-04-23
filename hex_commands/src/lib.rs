mod command_pipeline;
mod prelude;

use hex_discord::twilight_model::id::Id;
use hex_framework::Command;
use once_cell::sync::Lazy;
use std::collections::HashMap;

type BoxedCommand = Box<(dyn Command + Send + Sync)>;

macro_rules! register_command {
    ($map:expr, $command_pat:expr) => {{
        let cmd = $command_pat;
        $map.insert(
            cmd.build_command(Id::new(12345678)).command.name,
            Box::new(cmd),
        );
    }};
}

mod suggest;
mod util;

pub static COMMANDS: Lazy<HashMap<String, BoxedCommand>> = Lazy::new(|| {
    let mut map: HashMap<String, BoxedCommand> = HashMap::new();

    register_command!(map, util::PingCommand);
    register_command!(map, suggest::SuggestCommand);

    map
});
