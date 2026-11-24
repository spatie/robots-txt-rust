/// A specific Disallow entry in a robots.txt file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Disallow {
    /// The user-agent group that contains this entry.
    pub user_agent: String,
    /// The literal disallowed path pattern.
    pub base_path: String,
}

#[derive(Debug, Clone)]
struct Group {
    agent: String,
    disallows: Vec<String>,
    allows: Vec<String>,
    delay: Option<String>,
}

/// Parse robots.txt rules, user-agent groups and crawl delays.
///
/// Exact user-agent matching and wildcard-group fallback are enabled by default.
/// See the usage guide for PHP-compatible selection and precedence details.
#[derive(Debug, Clone)]
#[must_use]
pub struct RobotsTxt {
    content: String,
    groups: Vec<Group>,
    match_exactly: bool,
    include_global: bool,
}

impl RobotsTxt {
    /// Parse robots.txt content without accessing the network or filesystem.
    pub fn new(content: impl Into<String>) -> Self {
        let content = content.into();
        let mut groups: Vec<Group> = Vec::new();
        let mut agents = Vec::new();
        let mut delay_agents = Vec::new();
        let mut seen_rule = false;
        let mut agent_list = false;

        for line in content.lines().map(str::trim) {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let lower = line.to_ascii_lowercase();
            if lower.starts_with("user-agent") {
                if seen_rule {
                    agents.clear();
                    seen_rule = false;
                }
                if !agent_list {
                    delay_agents.clear();
                    agent_list = true;
                }
                let agent = lower
                    .replace("user-agent", "")
                    .trim_matches([':', ' '])
                    .to_owned();
                let index = groups
                    .iter()
                    .position(|group| group.agent == agent)
                    .unwrap_or_else(|| {
                        groups.push(Group {
                            agent,
                            disallows: Vec::new(),
                            allows: Vec::new(),
                            delay: None,
                        });
                        groups.len() - 1
                    });
                groups[index].disallows.clear();
                groups[index].allows.clear();
                agents.push(index);
                delay_agents.push(index);
                continue;
            }
            agent_list = false;
            let directive = lower.replace(' ', "");
            let disallow = directive
                .get(..8)
                .unwrap_or(&directive)
                .trim_matches([':', ' '])
                == "disallow";
            let allow = directive
                .get(..6)
                .unwrap_or(&directive)
                .trim_matches([':', ' '])
                == "allow";
            if disallow || allow {
                seen_rule = true;
            }
            // PHP parses non-rule directives as paths too. Keep that behavior here;
            // it is observable when a caller asks about a relative path.
            for &index in &agents {
                if !allow {
                    let value = line.get(8..).unwrap_or_default().trim_matches([':', ' ']);
                    if !groups[index].disallows.iter().any(|rule| rule == value) {
                        groups[index].disallows.push(value.to_owned());
                    }
                }
                if !disallow {
                    let value = line.get(6..).unwrap_or_default().trim_matches([':', ' ']);
                    if !groups[index].allows.iter().any(|rule| rule == value) {
                        groups[index].allows.push(value.to_owned());
                    }
                }
            }
            if directive
                .get(..11)
                .unwrap_or(&directive)
                .trim_matches([':', ' '])
                == "crawl-delay"
            {
                let delay = line.get(11..).unwrap_or_default().trim_matches([':', ' ']);
                for &index in &delay_agents {
                    groups[index].delay = Some(delay.to_owned());
                }
            }
        }
        Self {
            content,
            groups,
            match_exactly: true,
            include_global: true,
        }
    }

    /// Parse literal content. Unlike PHP's URL heuristic, loading is explicit with `read_from`.
    pub fn create(content: impl Into<String>) -> Self {
        Self::new(content)
    }

    /// Read a local file or HTTP(S) URL, allowing everything if reading fails, like PHP.
    pub fn read_from(source: &str) -> Self {
        Self::try_read_from(source).unwrap_or_else(|_| Self::new(""))
    }

    /// Read a local file or HTTP(S) URL without silently allowing on failure.
    ///
    /// # Errors
    /// Returns an error if the file or HTTP response cannot be read.
    pub fn try_read_from(source: &str) -> Result<Self, crate::Error> {
        Ok(Self::new(
            crate::read_source(source, "spatie/robots-txt")?.0,
        ))
    }

    /// Return the unmodified source content.
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Exclude the wildcard user-agent group from rule selection and crawl delays.
    pub fn ignore_global_group(mut self) -> Self {
        self.include_global = false;
        self
    }

    /// Include the wildcard user-agent group (the default).
    pub fn include_global_group(mut self) -> Self {
        self.include_global = true;
        self
    }

    /// Combine groups whose names occur within the supplied user agent, plus the global group.
    pub fn with_partial_matches(mut self) -> Self {
        self.match_exactly = false;
        self
    }

    /// Select exact, case-insensitive user-agent names with global fallback (the default).
    pub fn exact_matches_only(mut self) -> Self {
        self.match_exactly = true;
        self
    }

    /// Whether the wildcard user agent may crawl a URL or path.
    pub fn allows(&self, url: &str) -> bool {
        self.allows_for(url, "*")
    }

    /// Whether a user agent may crawl a URL or path, including its query string.
    pub fn allows_for(&self, url: &str, user_agent: &str) -> bool {
        let agent = user_agent.trim().to_ascii_lowercase();
        let uri = request_uri(url);
        let allows = self.rules(&agent, true);
        if allows.contains(&url) {
            return true;
        }
        let denied = self
            .rules(&agent, false)
            .iter()
            .filter(|rule| path_matches(uri, rule))
            .map(|rule| rule.len())
            .max()
            .unwrap_or(0);
        if denied == 0 {
            return true;
        }
        let allowed = allows
            .iter()
            .filter(|rule| path_matches(uri, rule))
            .map(|rule| rule.len())
            .max()
            .unwrap_or(0);
        allowed > 0 && allowed >= denied
    }

    /// Return `None` for an allowed path, otherwise literal-prefix reasons, specific group first.
    ///
    /// Like PHP, this diagnostic does not expand wildcard patterns or normalize the agent.
    /// A wildcard-only denial can therefore return `Some(Vec::new())`.
    pub fn why_disallows(&self, path: &str, user_agent: &str) -> Option<Vec<Disallow>> {
        if self.allows_for(path, user_agent) {
            return None;
        }
        let mut reasons = Vec::new();
        for agent in [Some(user_agent), (user_agent != "*").then_some("*")]
            .into_iter()
            .flatten()
        {
            if let Some(group) = self.groups.iter().find(|group| group.agent == agent) {
                for rule in &group.disallows {
                    if path.starts_with(rule) {
                        reasons.push(Disallow {
                            user_agent: agent.to_owned(),
                            base_path: rule.clone(),
                        });
                    }
                }
            }
        }
        Some(reasons)
    }

    /// The wildcard group's crawl delay, as written rather than rounded to whole seconds.
    pub fn crawl_delay(&self) -> Option<&str> {
        self.crawl_delay_for("*")
    }

    /// A case-insensitive exact user-agent crawl delay, with optional wildcard fallback.
    ///
    /// Partial rule matching does not affect crawl-delay selection, matching PHP.
    pub fn crawl_delay_for(&self, user_agent: &str) -> Option<&str> {
        let agent = user_agent.trim().to_ascii_lowercase();
        self.groups
            .iter()
            .filter(|group| self.include_global || group.agent != "*")
            .find(|group| group.agent == agent && group.delay.is_some())
            .or_else(|| {
                self.groups.iter().find(|group| {
                    self.include_global && group.agent == "*" && group.delay.is_some()
                })
            })
            .and_then(|group| group.delay.as_deref())
    }

    fn rules(&self, agent: &str, allow: bool) -> Vec<&str> {
        let groups = || {
            self.groups
                .iter()
                .filter(|group| self.include_global || group.agent != "*")
                .filter(|group| !allow || !group.allows.is_empty())
        };
        if self.match_exactly {
            return groups()
                .find(|group| group.agent == agent)
                .or_else(|| groups().find(|group| group.agent == "*"))
                .into_iter()
                .flat_map(|group| {
                    let rules = if allow {
                        &group.allows
                    } else {
                        &group.disallows
                    };
                    rules.iter().map(String::as_str)
                })
                .collect();
        }
        groups()
            .filter(|group| group.agent == "*" || agent.contains(&group.agent))
            .flat_map(|group| {
                let rules = if allow {
                    &group.allows
                } else {
                    &group.disallows
                };
                rules.iter().map(String::as_str)
            })
            .collect()
    }
}

fn request_uri(url: &str) -> &str {
    let without_fragment = url.split('#').next().unwrap_or_default();
    let path = without_fragment
        .split_once(':')
        .filter(|(scheme, _)| {
            !scheme.is_empty()
                && scheme
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || b"+-.".contains(&byte))
        })
        .map_or(without_fragment, |(_, rest)| rest);
    if let Some(rest) = path.strip_prefix("//") {
        return rest
            .find(['/', '?'])
            .map(|index| &rest[index..])
            .unwrap_or_default();
    }
    path
}

fn path_matches(path: &str, rule: &str) -> bool {
    if rule.is_empty() {
        return false;
    }
    let anchored = rule.ends_with('$');
    let pattern = if anchored {
        &rule[..rule.len() - 1]
    } else {
        rule
    }
    .as_bytes();
    let path = path.as_bytes();
    let (mut p, mut s) = (0, 0);
    let mut star = None;
    let mut retry = 0;
    loop {
        if p == pattern.len() && (!anchored || s == path.len()) {
            return true;
        }
        if pattern.get(p) == Some(&b'*') {
            star = Some(p);
            p += 1;
            retry = s;
        } else if s < path.len() && pattern.get(p) == Some(&path[s]) {
            p += 1;
            s += 1;
        } else if let Some(index) = star.filter(|_| retry < path.len() && path[retry] != b'\n') {
            retry += 1;
            s = retry;
            p = index + 1;
        } else {
            // PCRE's `$` also matches immediately before a final newline.
            return anchored && p == pattern.len() && &path[s..] == b"\n";
        }
    }
}
