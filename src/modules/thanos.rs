//!# Thanos Snap
//!
//!**WARNING:** This module can and will ruin chat.
//!
//! The Thanos Snap module is a module to be used after the "Thanos Snap"
//! channel point reward has been redeemed. To prevent abuse and accidental
//! activations, 3 mods need to run the command in a short time period before it
//! will go off.
//!
//!```
//!$snap run
//!    (mod only) After 3 different moderators run this, half of the people who
//!    have spoken in chat in the past 15 minutes will get a 30 minute timeout.
//!    $snap will be disabled afterward.
//!$snap rearm
//!    (bot/channel owner only) Rearms $snap after it has been disabled.
//!```

use super::Module;
use crate::config;
use std::{
    collections::{HashMap, HashSet},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use twitchchat::{
    commands::{privmsg, timeout},
    messages::Privmsg,
    Writer,
};

pub struct Thanos {
    chatters: HashMap<String, f64>,
    keys: HashSet<String>,
    enabled: bool,
    cleanup_ts: f64,
    key_ts: f64,
}

impl Thanos {
    pub fn new() -> Self {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        Self {
            chatters: HashMap::new(),
            keys: HashSet::new(),
            enabled: true,
            cleanup_ts: time + 900.0,
            key_ts: time + 60.0,
        }
    }

    fn do_snap(&mut self, writer: Writer) {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();

        let names: Vec<_> = self
            .chatters
            .iter()
            .filter(|(_, v)| *v > &(time - 900.0))
            .map(|(k, _)| k.clone())
            .filter(|_| rand::random::<bool>())
            .collect();

        tokio::task::spawn(timeout_loop(names, writer));
    }

    fn cmd_handler(&mut self, msg: &Privmsg, args: Vec<String>, mut writer: Writer) {
        if args.len() >= 2
            && args[1] == "rearm"
            && (msg.is_broadcaster() || msg.data() == config::OWNER)
        {
            writer
                .encode_sync(privmsg(msg.channel(), "$snap rearmed"))
                .unwrap();
            info!("snap rearmed");
            self.enabled = true;
            self.keys.clear();
            return;
        }

        if !self.enabled {
            writer
                .encode_sync(privmsg(
                    msg.channel(),
                    "$snap currently disabled. shift or dark should rearm it with $snap rearm",
                ))
                .unwrap();
            info!("snap disabled");
            return;
        }

        if args.len() >= 2 && args[1] == "run" {
            self.keys.insert(msg.name().into());
            //self.keys.insert(rand::random::<i32>().to_string());

            if self.keys.len() >= 3 {
                for i in self.keys.iter() {
                    info!("snap activated by {}", i);
                }
                writer
                    .encode_sync(privmsg(msg.channel(), "*snap*"))
                    .unwrap();

                self.do_snap(writer);
                self.enabled = false;
            } else {
                info!("key turned by {}", msg.name());
                writer
                    .encode_sync(privmsg(msg.channel(), "key turned"))
                    .unwrap();
            }
        } else {
            writer
                .encode_sync(privmsg(msg.channel(), "invalid usage"))
                .unwrap();
        }
    }
}

impl Module for Thanos {
    fn tick(&mut self, _writer: Writer) {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();

        while time > self.cleanup_ts {
            self.chatters = self
                .chatters
                .iter()
                .filter(|(_, v)| *v > &(time - 900.0))
                .map(|(k, v)| (k.clone(), *v))
                .collect();
            self.cleanup_ts += 900.0;
        }
        while time > self.key_ts {
            self.keys.clear();
            self.key_ts += 900.0;
        }
    }
    fn privmsg(&mut self, msg: &Privmsg, writer: Writer) {
        let args: Vec<_> = msg
            .data()
            .split_whitespace()
            .map(|x| x.to_string())
            .collect();

        if args[0] == "$snap"
            && (msg.is_moderator() || msg.is_broadcaster() || msg.name() == config::OWNER)
        {
            self.cmd_handler(msg, args, writer);
            return;
        }

        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();

        self.chatters.insert(msg.name().into(), time);
    }
}

async fn timeout_loop(names: Vec<String>, mut writer: Writer) {
    let mut interval = tokio::time::interval(Duration::from_millis(500));
    for name in names {
        info!("{}", &name);
        writer
            .encode_sync(timeout(
                config::CHANNEL,
                &name,
                "1800",
                "gone, reduced to atoms",
            ))
            .unwrap();

        interval.tick().await;
    }
}
