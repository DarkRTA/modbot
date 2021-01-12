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
    collections::HashMap,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use twitchchat::{
    commands::{privmsg, timeout},
    messages::Privmsg,
    Writer,
};

#[derive(PartialEq)]
enum ThanosState {
    Idle,
    Ready,
    Disabled,
}

pub struct Thanos {
    chatters: HashMap<String, f64>,
    state: ThanosState,
    cleanup_ts: f64,
}

impl Thanos {
    pub fn new() -> Self {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        Self {
            chatters: HashMap::new(),
            state: ThanosState::Idle,
            cleanup_ts: time + 900.0,
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
        if args.len() >= 2 && args[1] == "rearm" {
            writer
                .encode_sync(privmsg(msg.channel(), "$snap rearmed"))
                .unwrap();
            info!("snap rearmed");
            self.state = ThanosState::Idle;
            return;
        }

        if self.state == ThanosState::Disabled {
            writer
                .encode_sync(privmsg(
                    msg.channel(),
                    "$snap currently disabled. it should be rearmed with $snap rearm",
                ))
                .unwrap();
            info!("snap disabled");
            return;
        }

        if args.len() >= 2 && args[1] == "prepare" {
            self.state = ThanosState::Ready;
            info!("$snap prepared by {}", msg.name());
            writer
                .encode_sync(privmsg(msg.channel(), "$snap ready"))
                .unwrap();
        }

        if args.len() >= 2 && args[1] == "run" && self.state == ThanosState::Ready {
            info!("$snap ran by {}", msg.name());
            writer
                .encode_sync(privmsg(msg.channel(), "*snap*"))
                .unwrap();

            self.do_snap(writer);
            self.state = ThanosState::Disabled;
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

            if self.state != ThanosState::Disabled {
                self.state = ThanosState::Idle;
            }

            self.cleanup_ts += 900.0;
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
