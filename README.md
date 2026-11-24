# Parse `robots.txt`, `robots` meta and headers

[![Latest version on crates.io](https://img.shields.io/crates/v/spatie-robots-txt.svg?style=flat-square)](https://crates.io/crates/spatie-robots-txt)
[![Tests](https://img.shields.io/github/actions/workflow/status/spatie/robots-txt-rust/ci.yml?branch=main&style=flat-square)](https://github.com/spatie/robots-txt-rust/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square)](https://github.com/spatie/robots-txt-rust/blob/main/LICENSE.md)
[![Total downloads](https://img.shields.io/crates/d/spatie-robots-txt.svg?style=flat-square)](https://crates.io/crates/spatie-robots-txt)

Determine if a page may be crawled from robots.txt, robots meta tags and robot headers.

A Rust port of [spatie/robots-txt](https://github.com/spatie/robots-txt).

## Support us

[<img src="https://spatie.be/github-ad-image/robots-txt-rust.jpg" alt="Support Spatie's open source work" width="419px" />](https://spatie.be/github-ad-click/robots-txt-rust)

We invest a lot of resources into creating [best in class open source packages](https://spatie.be/open-source). You can support us by [buying one of our paid products](https://spatie.be/open-source/support-us).

We highly appreciate you sending us a postcard from your hometown, mentioning which of our package(s) you are using. You'll find our address on [our contact page](https://spatie.be/about-us). We publish all received postcards on [our virtual postcard wall](https://spatie.be/open-source/postcards).

## Installation

Requires Rust 1.89 or newer. You can install the package with Cargo:

```sh
cargo add spatie-robots-txt
```

## Usage

```rust
use spatie_robots_txt::RobotsTxt;

let robots_txt = RobotsTxt::new("
  User-agent: *
  Disallow: /admin
  Crawl-delay: 1.5
");

robots_txt.allows_for("/admin", "google"); // false
robots_txt.why_disallows("/admin", "google"); // Some(reasons), user_agent is "*"
robots_txt.crawl_delay(); // Some("1.5")
# assert!(!robots_txt.allows_for("/admin", "google"));
# assert_eq!(robots_txt.why_disallows("/admin", "google").unwrap()[0].user_agent, "*");
# assert_eq!(robots_txt.crawl_delay(), Some("1.5"));
```

To check a page, including its meta tags and headers:

```rust,no_run
# fn main() -> Result<(), spatie_robots_txt::Error> {
use spatie_robots_txt::Robots;

let robots = Robots::create();

robots.may_index("https://www.spatie.be/nl/admin")?;
robots.may_follow_on("https://www.spatie.be/nl/admin")?;
# Ok(())
# }
```

You can also specify a user agent:

```rust
# use spatie_robots_txt::Robots;
let robots = Robots::create().user_agent("UserAgent007");
# let _ = robots;
```

By default, `Robots` will look for a `robots.txt` file on `https://host.com/robots.txt`.
Another location can be specified with `with_txt_from`. Use `with_txt` to supply a parsed `RobotsTxt`.

The HTTP convenience methods are blocking. The parsers work without network access and can inspect content fetched by any HTTP client. See the [usage guide](https://github.com/spatie/robots-txt-rust/blob/main/docs/usage.md). An unreadable robots.txt allows crawling by default; use `try_read_from` to handle loading errors explicitly.

## Testing

```sh
cargo test --locked --all-targets --all-features
cargo test --locked --doc --all-features
```

See [all quality checks](https://github.com/spatie/robots-txt-rust/blob/main/docs/testing.md). The README and Markdown guides are tested by rustdoc, including setup on hidden `# ` lines.

## Changelog

See [CHANGELOG.md](https://github.com/spatie/robots-txt-rust/blob/main/CHANGELOG.md).

## Contributing

See [CONTRIBUTING.md](https://github.com/spatie/robots-txt-rust/blob/main/CONTRIBUTING.md) and the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/).

Using an AI coding agent? Load Apollo's [rust-best-practices skill](https://github.com/apollographql/skills/tree/main/skills/rust-best-practices) when writing or reviewing code in this package.

## Security Vulnerabilities

Report vulnerabilities privately using the instructions in [SECURITY.md](https://github.com/spatie/robots-txt-rust/blob/main/SECURITY.md).

## Credits

- [Brent Roose](https://github.com/brendt)
- [All contributors](https://github.com/spatie/robots-txt-rust/graphs/contributors)

## License

The MIT License (MIT). See [LICENSE.md](https://github.com/spatie/robots-txt-rust/blob/main/LICENSE.md).
