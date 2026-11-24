use crate::{Error, RobotsHeaders, RobotsMeta, RobotsTxt};

/// Check robots.txt, robots meta tags and response headers together.
///
/// HTTP operations are blocking and use a 60-second timeout. In an async
/// application, run these methods on a blocking worker or parse content fetched
/// by your own HTTP client using the individual parser types.
#[derive(Debug, Clone, Default)]
#[must_use]
pub struct Robots {
    user_agent: Option<String>,
    txt: Option<RobotsTxt>,
}

impl Robots {
    /// Create a checker that discovers `/robots.txt` on each page's origin.
    pub fn create() -> Self {
        Self::default()
    }

    /// Set the user agent for robots.txt selection and page HTTP requests.
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    /// Use an already parsed robots.txt instead of discovering it.
    pub fn with_txt(mut self, txt: RobotsTxt) -> Self {
        self.txt = Some(txt);
        self
    }

    /// Read robots.txt from a file or URL. Failed reads allow crawling, as in PHP.
    pub fn with_txt_from(self, source: &str) -> Self {
        self.with_txt(RobotsTxt::read_from(source))
    }

    /// Whether robots.txt, the page's first robots meta tag and global headers permit indexing.
    ///
    /// # Errors
    /// Returns an error for invalid discovery URLs or unreadable pages. An unreadable
    /// robots.txt is treated as empty. A denied robots.txt short-circuits the page request.
    pub fn may_index(&self, url: &str) -> Result<bool, Error> {
        self.may_index_for(url, self.user_agent.as_deref().unwrap_or(""))
    }

    /// Override the user agent used to select robots.txt rules for this check.
    ///
    /// Like PHP, this override does not change the page's HTTP User-Agent or select
    /// agent-specific response headers; use `RobotsHeaders` directly for those.
    ///
    /// # Errors
    /// Returns the same source/URL errors as `may_index`.
    pub fn may_index_for(&self, url: &str, user_agent: &str) -> Result<bool, Error> {
        let discovered;
        let txt = if let Some(txt) = &self.txt {
            txt
        } else {
            let mut robots_url = url::Url::parse(url)?;
            robots_url.set_path("/robots.txt");
            robots_url.set_query(None);
            robots_url.set_fragment(None);
            // robots.txt discovery uses the origin, not page credentials.
            let _ = robots_url.set_username("");
            let _ = robots_url.set_password(None);
            discovered = RobotsTxt::read_from(robots_url.as_str());
            &discovered
        };
        if !txt.allows_for(url, user_agent) {
            return Ok(false);
        }
        let (html, headers) = crate::read_source(url, self.http_user_agent())?;
        Ok(RobotsMeta::create(&html).may_index() && RobotsHeaders::create(headers).may_index())
    }

    /// Whether the page's robots meta tag and global headers permit following links.
    ///
    /// robots.txt is not consulted, matching PHP's `mayFollowOn`.
    ///
    /// # Errors
    /// Returns an error if the page cannot be read.
    pub fn may_follow_on(&self, url: &str) -> Result<bool, Error> {
        let (html, headers) = crate::read_source(url, self.http_user_agent())?;
        Ok(RobotsMeta::create(&html).may_follow() && RobotsHeaders::create(headers).may_follow())
    }

    fn http_user_agent(&self) -> &str {
        self.user_agent.as_deref().unwrap_or("spatie/robots-txt")
    }
}
