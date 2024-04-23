use std::ops::RangeInclusive;

pub const DEBUG: bool = false;
pub const DEBUG_GUILD_ID: u64 = 562364424002994189;

pub const ARENA_NAME_SIZE: RangeInclusive<usize> = 1..=64;
pub const ARENA_DESCRIPTION_SIZE: RangeInclusive<usize> = 1..=300;
