//! # Moderation
//!
//! The moderation module has some basic filters to perform some common spam
//! clean up tasks. This module only has a single command used to turn the off
//! as it operates under the assumption that the compile time configration is
//! correct.
//!
//!```
//!$moderation on
//!    (mod only) turns on moderation
//!$moderation off
//!    (mod only) turns off moderation
//!```

mod asciiart;
mod misc;
mod noflood;

use super::Module;
use crate::config;
use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};
use twitchchat::{
    commands::{ban, privmsg, timeout},
    messages::Privmsg,
    Writer,
};
use webhook::Webhook;

#[derive(Default, Clone)]
pub struct FilterMsg {
    pub nick: String,
    pub text: String,
    pub vip: bool,
    pub sub: bool,
    pub ts: f64,
}

trait Filter {
    fn filter(&mut self, msg: &FilterMsg) -> Option<(i32, &'static str)>;
}

pub struct ModerationConfig {
    pub lengths: Vec<i32>,
    pub antibot: bool,
    pub noflood: bool,
    pub asciiart: bool,
    pub misc: bool,
}

pub struct Moderation {
    bans_ts: f64,
    antibot_ts: f64,

    lengths: Vec<i32>,
    antibot: bool,
    filters: Vec<Box<dyn Filter>>,
    bans: HashMap<String, i32>,
    ban_count: f32,
    kill_switch: bool,
}

impl Moderation {
    pub fn new(config: ModerationConfig) -> Self {
        let mut filters: Vec<Box<dyn Filter>> = Vec::new();
        if config.noflood {
            filters.push(Box::new(noflood::NoFlood::new()))
        }
        if config.asciiart {
            filters.push(Box::new(asciiart::AsciiArt::new()))
        }
        if config.misc {
            filters.push(Box::new(misc::Misc::new()))
        }

        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();

        Self {
            bans_ts: time,
            antibot_ts: time,
            lengths: config.lengths,
            antibot: config.antibot,
            filters,
            bans: HashMap::new(),
            ban_count: 0.0,
            kill_switch: false,
        }
    }

    fn run_filters(&mut self, msg: &Privmsg) -> Option<(i32, &'static str)> {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();

        let ts = msg.tmi_sent_ts()? as f64 / 1000.0;
        // make sure our clock isn't too far out of sync with
        // what twitch sends us
        assert!((ts - time).abs() < 30.0);

        let msg = FilterMsg {
            nick: msg.name().into(),
            text: msg.data().into(),
            vip: msg.is_vip(),
            sub: msg.is_subscriber(),
            ts,
        };

        for filter in &mut self.filters {
            match filter.filter(&msg) {
                x @ Some((_, _)) => return x,
                _ => continue,
            }
        }
        None
    }

    fn log(&mut self, len: i32, reason: &str, msg: &Privmsg) {
        let mut duration = len.to_string();
        if len == -2 {
            duration = "ban (anti-bot)".into()
        }
        if len == -1 {
            duration = "ban".into()
        }
        info!("{}: {}: {}: {}", duration, msg.name(), reason, msg.data());
        let msg = format!(
            "```\nusername: {}\nreason:   {}\nduration: {}\nmessage:  {}\n```",
            msg.name(),
            reason,
            duration,
            msg.data()
        );
        tokio::spawn(async move {
            Webhook::from_url(config::WEBHOOK_URL)
                .send(|x| x.content(&msg))
                .await
                .unwrap()
        });
    }
}

impl Module for Moderation {
    fn tick(&mut self, _writer: Writer) {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();

        while time > self.bans_ts {
            self.bans = self
                .bans
                .iter()
                .map(|(k, v)| (k.clone(), v - 1))
                .filter(|(_, v)| *v > 0)
                .collect();
            self.bans_ts += 3600.0;
        }

        while time > self.antibot_ts {
            self.ban_count *= 0.7;
            self.antibot_ts += 15.0;
        }
    }

    fn privmsg(&mut self, msg: &Privmsg, mut writer: Writer) {
        if msg.name() == config::OWNER || msg.is_moderator() || msg.is_broadcaster() {
            if msg.data() == "$moderation off" {
                info!("moderation disabled");
                writer
                    .encode_sync(privmsg(msg.channel(), "moderation disabled"))
                    .unwrap();
                self.kill_switch = true;
            }
            if msg.data() == "$moderation on" {
                info!("moderation enabled");
                writer
                    .encode_sync(privmsg(msg.channel(), "moderation enabled"))
                    .unwrap();
                self.kill_switch = false;
            }
            return;
        }

        if self.kill_switch {
            return;
        }

        if let Some((score, reason)) = self.run_filters(msg) {
            let bans = match self.bans.get(msg.name()) {
                Some(x) => *x + score,
                None => score,
            };

            self.ban_count += 1.0;

            let len = {
                let bans = if bans as usize >= self.lengths.len() {
                    self.lengths.len() - 1
                } else {
                    bans as usize
                };

                self.lengths[bans]
            };

            self.bans.insert(msg.name().into(), bans);

            if self.ban_count > 5.0 && self.antibot {
                self.log(-2, reason, msg);
                writer
                    .encode_sync(ban(
                        msg.channel(),
                        msg.name(),
                        &format!("automatic ban (anti-bot): {}", reason)[..],
                    ))
                    .unwrap();
            } else if len == -1 {
                self.log(len, reason, msg);
                writer
                    .encode_sync(ban(
                        msg.channel(),
                        msg.name(),
                        &format!("automatic ban: {}", reason)[..],
                    ))
                    .unwrap();
            } else {
                self.log(len, reason, msg);
                writer
                    .encode_sync(timeout(
                        msg.channel(),
                        msg.name(),
                        &len.to_string()[..],
                        &format!("automatic timeout: {}", reason)[..],
                    ))
                    .unwrap();
            }
        }
    }
}
