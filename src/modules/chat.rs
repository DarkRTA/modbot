//! Basic module for logging sent chat messages.

use super::Module;
use twitchchat::{messages::Privmsg, Writer};

pub struct Chat;

impl Chat {
    pub fn new() -> Self {
        Self
    }
}

impl Module for Chat {
    fn privmsg(&mut self, msg: &Privmsg, _writer: Writer) {
        info!("<{}> {}", msg.name(), msg.data());
    }
}
