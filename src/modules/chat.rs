//! Simple module that prints out chat messsages

use super::Module;
use twitchchat::{messages::Privmsg, Writer};

pub struct Chat;

impl Chat {
    pub fn new() -> Chat {
        Chat
    }
}

impl Module for Chat {
    fn privmsg(&mut self, msg: &Privmsg, _writer: Writer) {
        info!("<{}> {}", msg.name(), msg.data());
    }
}
