//! This module attempts to prevent anyone from flodding chat. This can be very
//! effective when paired with the anit-bot feature to clean up really fast
//! bot spam.
use super::{Filter, FilterMsg};

//TODO: move to config
const FLOOD_TIME: f64 = 6.0;
const FLOOD_COUNT: usize = 4;
const FLOOD_LEN: usize = 200;

pub struct NoFlood {
    log: Vec<FilterMsg>,
}

impl NoFlood {
    pub fn new() -> Self {
        Self { log: Vec::new() }
    }

    fn pushuser(&mut self, msg: &FilterMsg) {
        // TODO: make this more efficient
        self.log.push(msg.clone());
        self.log = self
            .log
            .clone()
            .into_iter()
            .filter(|x| !(x.vip || x.ts < msg.ts - FLOOD_TIME))
            .collect()
    }
}

impl Filter for NoFlood {
    fn filter(&mut self, msg: &FilterMsg) -> Option<(i32, &'static str)> {
        if msg.vip {
            return None;
        }

        self.pushuser(msg);

        let count = self.log.iter().filter(|x| x.nick == msg.nick).count();

        if count >= FLOOD_COUNT {
            if self
                .log
                .iter()
                .filter(|x| x.nick == msg.nick && x.text.len() >= FLOOD_LEN)
                .count()
                >= 1
            {
                Some((4, "possible flooder"))
            } else {
                Some((1, "hit rate limit"))
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::super::{Filter, FilterMsg};
    use super::{NoFlood, FLOOD_COUNT, FLOOD_LEN, FLOOD_TIME};

    fn msg(nick: &str, text: &str, ts: f64) -> FilterMsg {
        FilterMsg {
            nick: nick.into(),
            text: text.into(),
            ts,
            ..Default::default()
        }
    }

    #[test]
    fn single_message() {
        let mut filter = NoFlood::new();
        let msg = msg("a", "hey shift!", 0.0);
        assert_eq!(None, filter.filter(&msg));
    }
    #[test]
    fn single_user_flood() {
        let mut filter = NoFlood::new();
        for i in 0..(FLOOD_COUNT - 1) {
            filter.filter(&msg("a", &format!("hey shift! {}", i), 0.0));
        }
        assert_eq!(
            Some((4, "possible flooder")),
            filter.filter(&msg(
                "a",
                &"h".to_string().repeat(FLOOD_LEN),
                FLOOD_TIME - 0.01
            ))
        );
    }
    #[test]
    fn single_user_missed_flood() {
        let mut filter = NoFlood::new();
        for i in 0..(FLOOD_COUNT - 1) {
            filter.filter(&msg("a", &format!("hey shift! {}", i), 0.0));
        }
        assert_eq!(
            Some((1, "hit rate limit")),
            filter.filter(&msg(
                "a",
                &"h".to_string().repeat(FLOOD_LEN - 1),
                FLOOD_TIME - 0.01
            ))
        );
    }
    #[test]
    fn single_user_missed_ratelimit() {
        let mut filter = NoFlood::new();
        for i in 0..(FLOOD_COUNT - 1) {
            filter.filter(&msg("a", &format!("hey shift! {}", i), 0.0));
        }
        assert_eq!(
            None,
            filter.filter(&msg(
                "a",
                &"h".to_string().repeat(FLOOD_LEN),
                FLOOD_TIME + 0.01
            ))
        );
    }
    #[test]
    fn multi_user_no_timeout() {
        let mut filter = NoFlood::new();
        for i in 0..(FLOOD_COUNT - 1) {
            filter.filter(&msg("a", &format!("hey shift! {}", i), 0.0));
        }
        assert_eq!(
            None,
            filter.filter(&msg(
                "b",
                &"h".to_string().repeat(FLOOD_LEN),
                FLOOD_TIME - 0.01
            ))
        );
    }
    #[test]
    fn exclude_vips() {
        let mut filter = NoFlood::new();
        for i in 0..(FLOOD_COUNT - 1) {
            filter.filter(&msg("a", &format!("hey shift! {}", i), 0.0));
        }
        assert_eq!(
            None,
            filter.filter(&FilterMsg {
                nick: "a".into(),
                text: "h".to_string().repeat(FLOOD_LEN),
                vip: true,
                ts: FLOOD_TIME - 0.01,
                ..Default::default()
            })
        );
    }
}
