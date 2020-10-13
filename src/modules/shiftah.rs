//! SHiFT chat easter egg. Randomly says "shiftAh" in chat.

use super::Module;
use twitchchat::{commands::privmsg, messages::Privmsg, Writer};

pub struct ShiftAh;

impl ShiftAh {
    pub fn new() -> ShiftAh {
        ShiftAh
    }
}

impl Module for ShiftAh {
    fn privmsg(&mut self, msg: &Privmsg, mut writer: Writer) {
        if rand::random::<f32>() < 1.0 / 8192.0 {
            writer
                .encode_sync(privmsg(msg.channel(), "shiftAh"))
                .unwrap();
        }
    }
}
