use twitchchat::{messages::Privmsg, Writer};

mod chat;
mod moderation;
mod shiftah;
mod thanos;
pub use chat::Chat;
pub use moderation::Moderation;
pub use moderation::ModerationConfig;
pub use shiftah::ShiftAh;
pub use thanos::Thanos;

/// This trait should be implemented if you are writing a new module.
pub trait Module {
    /// This function is generally used to update and run the timers of a module
    /// Don't rely on this running at a regular interval.
    fn tick(&mut self, _writer: Writer) {}
    /// This function is called every time a chat message is received
    fn privmsg(&mut self, msg: &Privmsg, writer: Writer);
}
