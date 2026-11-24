---
title: Usage
weight: 4
---

## Checking a page

```rust,no_run
# use spatie_robots_txt::Robots;
# fn main() -> Result<(), spatie_robots_txt::Error> {
let robots = Robots::create();

robots.may_index("https://www.spatie.be/nl/admin")?;
robots.may_follow_on("https://www.spatie.be/nl/admin")?;
# Ok(())
# }
```

`may_index` first checks robots.txt. If crawling is allowed, it reads the page and checks its robots meta tag and global X-Robots-Tag headers. `may_follow_on` checks only the page's meta tag and global headers, not robots.txt.

You can also specify a user agent:

```rust
# use spatie_robots_txt::Robots;
let robots = Robots::create().user_agent("UserAgent007");
# let _ = robots;
```

`may_index_for(url, "google")` overrides robots.txt rule selection for one call. This does not change the HTTP User-Agent or select agent-specific response headers. Inspect `RobotsHeaders` directly when you need those.

By default, `Robots` will look for a `robots.txt` file on `https://host.com/robots.txt`. The page's scheme, host and port are preserved. Another location can be specified like so:

```rust,no_run
# use spatie_robots_txt::Robots;
let robots = Robots::create()
    .with_txt_from("https://www.spatie.be/robots-custom.txt");

let robots = Robots::create()
    .with_txt_from("public/robots.txt");
# let _ = robots;
```

Use `with_txt` for already parsed content:

```rust
# use spatie_robots_txt::{Robots, RobotsTxt};
# fn main() -> Result<(), spatie_robots_txt::Error> {
let robots_txt = RobotsTxt::new("User-agent: *\nDisallow: /admin");
let robots = Robots::create().with_txt(robots_txt);

robots.may_index("https://www.spatie.be/admin")?; // false; no page request
# assert!(!robots.may_index("https://www.spatie.be/admin")?);
# Ok(())
# }
```

HTTP methods are blocking, use rustls for HTTPS, follow up to ten redirects and time out after 60 seconds. In async applications, use a blocking worker or supply already fetched strings to the parsers below. Fetch only trusted URLs; the convenience methods do not filter private network destinations.

## Parsing robots.txt

```rust
# use spatie_robots_txt::RobotsTxt;
let robots_txt = RobotsTxt::new("
  User-agent: *
  Disallow: /admin
  Crawl-delay: 1.5
");

robots_txt.allows_for("/admin", "google"); // false
robots_txt.why_disallows("/admin", "google"); // Some(reasons)
robots_txt.crawl_delay(); // Some("1.5")
# assert!(!robots_txt.allows_for("/admin", "google"));
# assert_eq!(robots_txt.why_disallows("/admin", "google").unwrap()[0].base_path, "/admin");
# assert_eq!(robots_txt.crawl_delay(), Some("1.5"));
```

`allows(url)` uses the wildcard agent. `allows_for(url, user_agent)` accepts paths or full URLs, includes query strings (even an empty `?`), and excludes fragments. Paths are case-sensitive. `*` matches any number of characters, and a final `$` anchors a pattern to the end. The longest matching rule wins; an allow wins a tie. Pattern weights count bytes, including `*` and `$`.

By default, agent names are matched exactly after trimming and lowercasing. When no group matches, the wildcard group is used. Allow and disallow group selection are independent; an exact group without any allow entries can still use the wildcard group's allows. Repeated groups replace previous rules rather than merging them.

For partial user-agent matching:

```rust
# use spatie_robots_txt::RobotsTxt;
# let content = "User-agent: *\nDisallow: /global\nUser-agent: UserAgent007\nDisallow: /no-agents\n";
let robots_txt = RobotsTxt::new(content).with_partial_matches();

robots_txt.allows_for("/no-agents", "Mozilla/5.0 (compatible; UserAgent007/1.1)"); // false
# assert!(!robots_txt.allows_for("/no-agents", "Mozilla/5.0 (compatible; UserAgent007/1.1)"));
```

Partial matching combines all contained agent names and the wildcard group. Chain `ignore_global_group()` to exclude wildcard rules and delays; `include_global_group()` restores them. `exact_matches_only()` restores exact rule matching. These consuming setters return the configured value.

`crawl_delay_for("bing")` returns an exact agent's delay with wildcard fallback, regardless of partial rule matching. Delays remain strings so fractional values such as `"1.5"` are preserved.

`why_disallows(path, user_agent)` returns `None` when allowed, or a list of `Disallow { user_agent, base_path }` reasons. Reasons use literal prefix checks, list the exact agent before the global group, and do not normalize the diagnostic agent argument. Wildcard-only denials can return an empty list; use `allows_for` for the decision.

`new(content)` and `create(content)` parse literal strings. `read_from(source)` loads a file or HTTP(S) URL. An unreadable robots.txt becomes empty and allows crawling. To handle loading errors explicitly:

```rust
# use spatie_robots_txt::RobotsTxt;
# fn main() -> Result<(), spatie_robots_txt::Error> {
# let source = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/robots.txt");
let robots_txt = RobotsTxt::try_read_from(source)?;
# assert!(!robots_txt.allows("/en/admin/"));
# Ok(())
# }
```

## Robots meta tags

```rust
# use spatie_robots_txt::RobotsMeta;
let robots = RobotsMeta::create("<meta name=\"robots\" content=\"noindex, nofollow\">");

robots.may_index(); // false
robots.may_follow(); // false
robots.noindex(); // true
robots.nofollow(); // true
# assert!(!robots.may_index() && !robots.may_follow());
# assert!(robots.noindex() && robots.nofollow());
```

`read_from(source)?` reads HTML from a file or URL. Only the first tag starting with `<meta name="robots"` is considered. Single quotes, mixed case and multiline attributes work. Attributes in a different order do not match. Only `noindex` and `nofollow` substrings are checked; `none` is not expanded in meta tags.

## X-Robots-Tag headers

```rust
# use spatie_robots_txt::RobotsHeaders;
let robots = RobotsHeaders::create([
    "X-custom: test",
    "X-Robots-Tag: nofollow",
    "X-Robots-Tag: google: noindex, nofollow",
]);

robots.may_index(); // true
robots.may_follow(); // false
robots.may_index_for("google"); // false
robots.may_follow_for("google"); // false
# assert!(robots.may_index());
# assert!(!robots.may_follow());
# assert!(!robots.may_index_for("google") && !robots.may_follow_for("google"));
```

`none` disables both indexing and following in headers. The `noindex`, `nofollow` and `none` predicates each have a `_for(agent)` variant. Header names and directives are case-insensitive; agent keys are case-sensitive. The last header for a given agent replaces earlier ones. Specific-agent directives replace global directives, including explicit false values. Search engines can apply different rules.

Use `from_pairs` for `(name, value)` pairs, including repeated header names. Use `read_from(url)?` to fetch a response's headers. Meta/header reads and page checks return `Result` on loading errors, unlike the fail-open robots.txt reader.

Sources are local paths or HTTP(S) URLs. Local files must be UTF-8; HTTP text uses the response encoding. HTTPS certificates are verified. Requests follow at most ten redirects, inspect the final response headers and have a 60-second whole-request timeout.
