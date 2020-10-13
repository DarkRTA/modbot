use twitchchat::{messages::Privmsg, Writer};

//mod filters;
//pub use filters::Filters;

mod chat;
mod moderation;
mod shiftah;
mod thanos;
pub use chat::Chat;
pub use moderation::Moderation;
pub use moderation::ModerationConfig;
pub use shiftah::ShiftAh;
pub use thanos::Thanos;

pub trait Module {
    fn tick(&mut self, _writer: Writer) {}
    fn privmsg(&mut self, msg: &Privmsg, writer: Writer);
}
