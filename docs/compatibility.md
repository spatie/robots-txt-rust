---
title: PHP compatibility
weight: 7
---

The reference is [spatie/robots-txt at df0f6c8](https://github.com/spatie/robots-txt/tree/df0f6c8cdaa6565d9413db30b4a321a15cbd529e). The PHP package has a README rather than a separate documentation guide. The introduction and usage examples follow that README, with the parser details documented here and in the API reference.

## Feature parity

| PHP behavior | Rust API | Verification |
| --- | --- | --- |
| Literal robots.txt, grouped agents, wildcard/anchored paths, queries, longest-rule precedence | `RobotsTxt::new`, `create`, `allows`, `allows_for` | PHP-generated rule fixtures |
| Exact/partial agent matches and include/ignore global group | Consuming snake_case setters | All four mode combinations against PHP |
| Crawl delays including fractions and global fallback | `crawl_delay`, `crawl_delay_for` → `Option<&str>` | PHP-generated delay fixtures |
| Disallow reasons with original agent and path | `why_disallows` → `Option<Vec<Disallow>>` | PHP-generated reason fixtures |
| First robots meta tag, noindex/nofollow | `RobotsMeta` | PHP HTML files and syntax-edge fixtures |
| Global/scoped/repeated X-Robots-Tag and `none` | `RobotsHeaders` | PHP-generated header fixtures and real loopback responses |
| Source loading and failure policies | `read_from`, `try_read_from` | Local-file and real loopback HTTP tests |
| Default robots.txt discovery, custom source, user agent, combined index/follow checks | `Robots` | Real loopback HTTP tests, including denied-request short-circuiting |

The checked-in expectations are produced by `tests/generate-fixtures.php`, not by the Rust implementation. The normal Rust test suite needs no PHP installation or running web server. See [testing](testing.md) to regenerate them. The parser fixture checks execute locally and in CI; network examples marked `no_run` compile without contacting public sites.

## Rust differences

- Optional user-agent arguments become separate methods: `allows_for`, `may_index_for`, `crawl_delay_for` and header predicates ending in `_for`. `Robots::create()` takes no arguments; configure `.user_agent("UserAgent007")` afterward.
- `RobotsTxt::new` and `create` always parse literal content. PHP's `create` guesses whether a string is a robots.txt URL. Rust requires `read_from` for loading. `Robots::with_txt` takes a parsed value; `with_txt_from` takes a source string.
- Fields that would invalidate parsed state are private. Read original content through `content()`. Rust constructors and consuming setters prevent accidentally discarding updated configuration.
- Recoverable page, HTML and header loading failures return `Error` instead of exceptions. `RobotsTxt::read_from` deliberately retains PHP's fail-open policy; `try_read_from` adds explicit error handling.
- Sources are local paths or HTTP(S) URLs, not PHP stream wrappers. Local text must be UTF-8. HTTP text uses reqwest's decoding. Rustls validates HTTPS certificates. Redirect handling uses the final response's headers rather than PHP's accumulated redirect-header array, follows at most ten redirects and applies a 60-second whole-request timeout.
- Lines use platform-independent LF/CRLF handling. This avoids PHP's dependence on the host's `PHP_EOL` when parsing files written elsewhere.
- The unused PHP `WhyDisallows` wrapper and unused static cache are not reproduced. The actual `whyDisallows` method returns a list; Rust returns that list directly inside `Option`.

The PHP package's observable parsing quirks are retained: exact-group replacement, independent allow/disallow fallback, substring directive detection, literal-prefix diagnostic reasons, and the restricted meta-tag syntax. Unsupported robots.txt directives can also produce literal rule entries through PHP's fixed-offset parser. This crate is a compatibility port, not a replacement interpretation of RFC 9309.
