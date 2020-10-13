//compile_error!("please edit config.rs");
use crate::modules::{self, Module};

pub const OWNER: &str = "username";
pub const CHANNEL: &str = "#channame";
pub const USERNAME: &str = "botname";
pub const TOKEN: &str = "oauth:xxxxxxxxxx";

pub const WEBHOOK_URL: &str = "url/goes/here";

pub fn modules() -> Vec<Box<dyn Module>> {
    vec![
        Box::new(modules::Moderation::new(modules::ModerationConfig {
            lengths: vec![0, 5, 60, 300, 600, 1200, 1800, 3600, -1],
            antibot: true,
            noflood: true,
            asciiart: true,
            misc: true,
        })),
        Box::new(modules::Chat::new()),
        Box::new(modules::ShiftAh::new()),
        Box::new(modules::Thanos::new()),
    ]
}
