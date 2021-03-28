//! Really dumb filter that removes a majority of ascii art from chat.

use super::{Filter, FilterMsg};
use regex::Regex;

pub struct AsciiArt;

impl AsciiArt {
    pub fn new() -> Self {
        Self
    }
}

impl Filter for AsciiArt {
    fn filter(&mut self, msg: &FilterMsg) -> Option<(i32, &'static str)> {
        //TODO: lazy compile this regex
        let re = Regex::new(r"[a-zA-Z0-9 \.\?!]").unwrap();
        // allow small ascii art
        if msg.text.len() <= 175 {
            return None;
        };

        let ascii_chars: usize = re.find_iter(&msg.text).count();

        if ascii_chars as f32 / msg.text.len() as f32 <= 0.08 {
            return Some((1, "possible ascii art"));
        }
        None
    }
}

#[cfg(test)]
mod test {
    use super::super::{Filter, FilterMsg};
    use super::AsciiArt;

    fn msg(text: &str) -> FilterMsg {
        FilterMsg {
            text: text.into(),
            ..Default::default()
        }
    }

    #[test]
    fn short_text() {
        let msg = msg("this is a normal chat message");
        let mut filter = AsciiArt::new();
        assert_eq!(None, filter.filter(&msg));
    }
    #[test]
    fn long_text() {
        let msg = msg(&"this is a normal chat message".to_string().repeat(30));
        let mut filter = AsciiArt::new();
        assert_eq!(None, filter.filter(&msg));
    }
    #[test]
    fn short_ascii_art() {
        let msg = msg(&"|".to_string().repeat(60));
        let mut filter = AsciiArt::new();
        assert_eq!(None, filter.filter(&msg));
    }
    #[test]
    fn long_ascii_art() {
        let msg = msg(&"|".to_string().repeat(300));
        let mut filter = AsciiArt::new();
        assert_eq!(Some((1, "possible ascii art")), filter.filter(&msg));
    }
    #[test]
    fn truck() {
        let msg = msg(r"──────▄▌▐▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▌ ───▄▄ █ KKona WATCH OUT I'M TRUCKIN ███████▌█▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▌ ▀(@)▀▀▀▀▀▀▀(@)(@)▀▀▀▀▀▀▀▀▀▀▀▀▀(@)▀");
        let mut filter = AsciiArt::new();
        assert_eq!(None, filter.filter(&msg));
    }
}
