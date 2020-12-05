//! Miscellaneous regex that doesn't really fit anywhere else.

use super::{Filter, FilterMsg};
use regex::Regex;

pub struct Misc;

impl Misc {
    pub fn new() -> Self {
        Self
    }
}

impl Filter for Misc {
    fn filter(&mut self, msg: &FilterMsg) -> Option<(i32, &'static str)> {
        //TODO: lazy compile the regex here
        let re = Regex::new("live_[a-zA-Z0-9]*_[a-zA-Z0-9]*").unwrap();
        if re.is_match(&msg.text) {
            return Some((1, "possible stream key"));
        }

        let re = Regex::new("oauth:[a-zA-Z0-9]*").unwrap();
        if re.is_match(&msg.text) {
            return Some((1, "possible oauth token"));
        }

        let re = Regex::new("\\(?[0-9]{3}\\)?[- ][0-9]{3}[- ][0-9]{4}").unwrap();
        if re.is_match(&msg.text) {
            return Some((2, "possible phone number"));
        }

        None
    }
}

#[cfg(test)]
mod test {
    use super::super::{Filter, FilterMsg};
    use super::Misc;

    fn message(text: &str) -> FilterMsg {
        FilterMsg {
            text: text.into(),
            ..Default::default()
        }
    }

    #[test]
    fn short_text() {
        let msg = message("this is a normal chat message");
        let mut filter = Misc::new();
        assert_eq!(None, filter.filter(&msg));
    }
    #[test]
    fn long_text() {
        let msg = message(&"this is a normal chat message".to_string().repeat(30));
        let mut filter = Misc::new();
        assert_eq!(None, filter.filter(&msg));
    }
    #[test]
    fn stream_key() {
        let msg = message("my stream key is live_34jk2l34_343");
        let mut filter = Misc::new();
        assert_eq!(Some((1, "possible stream key")), filter.filter(&msg));
    }
    #[test]
    fn oauth_token() {
        let msg = message("lol oauth:3wfkjwlkfjawklfjwelkfjkljf");
        let mut filter = Misc::new();
        assert_eq!(Some((1, "possible oauth token")), filter.filter(&msg));
    }
    #[test]
    fn phone_number() {
        let msg = message("256-555-7556");
        let mut filter = Misc::new();
        assert_eq!(Some((2, "possible phone number")), filter.filter(&msg));

        let msg = message("256 555 7556");
        assert_eq!(Some((2, "possible phone number")), filter.filter(&msg));

        let msg = message("(256) 555-7556");
        assert_eq!(Some((2, "possible phone number")), filter.filter(&msg));

        let msg = message("2565557556");
        assert_eq!(None, filter.filter(&msg));
    }
}
