#![doc = include_str!("../README.md")]

mod headers;
mod meta;
mod robots;
mod txt;

pub use headers::RobotsHeaders;
pub use meta::RobotsMeta;
pub use robots::Robots;
pub use txt::{Disallow, RobotsTxt};

/// A source could not be read or a URL could not be parsed.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// A local file could not be read as UTF-8.
    #[error("Could not read source: {0}")]
    Io(#[from] std::io::Error),
    /// An HTTP request failed, including non-success response statuses.
    #[error("Could not read URL: {0}")]
    Http(#[from] reqwest::Error),
    /// A page URL used for robots.txt discovery was invalid.
    #[error("Invalid URL: {0}")]
    Url(#[from] url::ParseError),
}

fn read_source(source: &str, user_agent: &str) -> Result<(String, Vec<String>), Error> {
    if source.split_once("://").is_some_and(|(scheme, _)| {
        scheme.eq_ignore_ascii_case("http") || scheme.eq_ignore_ascii_case("https")
    }) {
        let response = reqwest::blocking::Client::builder()
            .user_agent(user_agent)
            .timeout(std::time::Duration::from_secs(60))
            .build()?
            .get(source)
            .send()?
            .error_for_status()?;
        let mut headers = Vec::new();
        for (name, value) in response.headers() {
            headers.push(format!(
                "{name}: {}",
                String::from_utf8_lossy(value.as_bytes())
            ));
        }
        Ok((response.text()?, headers))
    } else {
        Ok((std::fs::read_to_string(source)?, Vec::new()))
    }
}

#[cfg(doctest)]
mod guide {
    include!(concat!(env!("OUT_DIR"), "/documentation.rs"));
}
