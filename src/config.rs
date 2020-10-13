//! The config module is used for compile time configuration of the bot.

compile_error!("please edit config.rs");
use crate::modules::{self, Module};

/// The username of the bot's owner.
pub const OWNER: &str = "username";
/// The channel to join.
pub const CHANNEL: &str = "#channame";
/// The username of the bot.
pub const USERNAME: &str = "botname";
/// The oauth token used to log into the bot's account.
pub const TOKEN: &str = "oauth:xxxxxxxxxx";

/// A url to a discord webhook.
pub const WEBHOOK_URL: &str = "url/goes/here";

/// This function is used to load and configure the modules.
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
