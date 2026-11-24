use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Default)]
struct Directives {
    noindex: bool,
    nofollow: bool,
    none: bool,
}

/// Inspect X-Robots-Tag response headers, including user-agent-specific directives.
///
/// Header names and directive values are case-insensitive. User-agent keys are
/// case-sensitive, and the last header for an agent replaces earlier headers.
/// A specific agent's directives replace the global directives as in PHP.
#[derive(Debug, Clone, Default)]
pub struct RobotsHeaders {
    directives: HashMap<String, Directives>,
}

impl RobotsHeaders {
    /// Parse raw header lines such as `X-Robots-Tag: google: noindex, nofollow`.
    #[must_use]
    pub fn create(headers: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
        let mut directives = HashMap::new();
        for header in headers {
            let header = header.as_ref();
            if !header.to_ascii_lowercase().starts_with("x-robots-tag") {
                continue;
            }
            let parts: Vec<_> = header.split(':').collect();
            let agent = if parts.len() == 3 {
                parts[1].trim()
            } else {
                "*"
            };
            let options = parts
                .last()
                .copied()
                .unwrap_or_default()
                .to_ascii_lowercase();
            directives.insert(
                agent.to_owned(),
                Directives {
                    noindex: options.contains("noindex"),
                    nofollow: options.contains("nofollow"),
                    none: options.contains("none"),
                },
            );
        }
        Self { directives }
    }

    /// Parse HTTP header name/value pairs, including repeated names.
    #[must_use]
    pub fn from_pairs<K: AsRef<str>, V: AsRef<str>>(
        headers: impl IntoIterator<Item = (K, V)>,
    ) -> Self {
        Self::create(
            headers
                .into_iter()
                .map(|(name, value)| format!("{}: {}", name.as_ref(), value.as_ref())),
        )
    }

    /// Read response headers from an HTTP(S) URL. Local files have no headers.
    ///
    /// # Errors
    /// Returns an error if the source cannot be read.
    pub fn read_from(source: &str) -> Result<Self, crate::Error> {
        Ok(Self::create(
            crate::read_source(source, "spatie/robots-txt")?.1,
        ))
    }

    fn for_agent(&self, agent: &str) -> Directives {
        self.directives
            .get(agent)
            .or_else(|| self.directives.get("*"))
            .copied()
            .unwrap_or_default()
    }

    /// Whether global headers permit indexing.
    #[must_use]
    pub fn may_index(&self) -> bool {
        self.may_index_for("*")
    }
    /// Whether this agent's headers permit indexing.
    #[must_use]
    pub fn may_index_for(&self, agent: &str) -> bool {
        !self.none_for(agent) && !self.noindex_for(agent)
    }
    /// Whether global headers permit following links.
    #[must_use]
    pub fn may_follow(&self) -> bool {
        self.may_follow_for("*")
    }
    /// Whether this agent's headers permit following links.
    #[must_use]
    pub fn may_follow_for(&self, agent: &str) -> bool {
        !self.none_for(agent) && !self.nofollow_for(agent)
    }
    /// Whether global headers contain `noindex`.
    #[must_use]
    pub fn noindex(&self) -> bool {
        self.noindex_for("*")
    }
    /// Whether this agent's headers contain `noindex`, falling back to global headers.
    #[must_use]
    pub fn noindex_for(&self, agent: &str) -> bool {
        self.for_agent(agent).noindex
    }
    /// Whether global headers contain `nofollow`.
    #[must_use]
    pub fn nofollow(&self) -> bool {
        self.nofollow_for("*")
    }
    /// Whether this agent's headers contain `nofollow`, falling back to global headers.
    #[must_use]
    pub fn nofollow_for(&self, agent: &str) -> bool {
        self.for_agent(agent).nofollow
    }
    /// Whether global headers contain `none`.
    #[must_use]
    pub fn none(&self) -> bool {
        self.none_for("*")
    }
    /// Whether this agent's headers contain `none`, falling back to global headers.
    #[must_use]
    pub fn none_for(&self, agent: &str) -> bool {
        self.for_agent(agent).none
    }
}
