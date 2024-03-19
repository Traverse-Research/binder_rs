#[path = "android/hardware/power/stats/mod.rs"]
pub mod powerstats;

pub(crate) mod mangled {
    pub(crate) use super::powerstats::mangled::*;
}

#[test]
fn test() {}
