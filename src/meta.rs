use std::sync::LazyLock;

use regex::Regex;

static META: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?mis)<meta name=("|')robots("|').*?>"#)
        .expect("the literal meta-tag expression is valid")
});

/// Inspect the first robots meta tag, matching the PHP package's tag syntax.
///
/// The tag must start with `<meta name="robots"` (single quotes and case variations
/// work too). `none` is not expanded; only `noindex` and `nofollow` are tested.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RobotsMeta {
    noindex: bool,
    nofollow: bool,
}

impl RobotsMeta {
    /// Inspect HTML without loading a source.
    #[must_use]
    pub fn create(html: &str) -> Self {
        let tag = META
            .find(html)
            .map(|found| found.as_str().to_ascii_lowercase())
            .unwrap_or_default();
        Self {
            noindex: tag.contains("noindex"),
            nofollow: tag.contains("nofollow"),
        }
    }

    /// Inspect HTML read from a file or HTTP(S) URL.
    ///
    /// # Errors
    /// Returns an error if the source cannot be read.
    pub fn read_from(source: &str) -> Result<Self, crate::Error> {
        Ok(Self::create(
            &crate::read_source(source, "spatie/robots-txt")?.0,
        ))
    }

    /// Whether the tag permits indexing (also true when no matching tag exists).
    #[must_use]
    pub fn may_index(&self) -> bool {
        !self.noindex
    }

    /// Whether the tag permits following links.
    #[must_use]
    pub fn may_follow(&self) -> bool {
        !self.nofollow
    }

    /// Whether the matching tag contains `noindex`.
    #[must_use]
    pub fn noindex(&self) -> bool {
        self.noindex
    }

    /// Whether the matching tag contains `nofollow`.
    #[must_use]
    pub fn nofollow(&self) -> bool {
        self.nofollow
    }
}
