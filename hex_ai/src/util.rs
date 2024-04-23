use regex::Regex;

use crate::{
    brain::Brain, claude_brain::ClaudeBrain, cohere_brain::CohereBrain, common::BrainKind,
};

pub fn remove_italic_actions(input: &str) -> String {
    let re = Regex::new(r"[_*][^_*]+[_*]").unwrap();
    let output = re.replace_all(input, "");
    output.trim().to_string()
}

pub fn get_brain(brain: BrainKind) -> Box<dyn Brain + Send + Sync + 'static> {
    match brain {
        BrainKind::CohereCommandR => Box::new(CohereBrain),
        BrainKind::ClaudeHaiku => Box::new(ClaudeBrain),
    }
}
