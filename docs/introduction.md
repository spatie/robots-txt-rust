---
title: Introduction
weight: 1
---

Determine if a page may be crawled from robots.txt, robots meta tags and robot headers.

This is the Rust port of [spatie/robots-txt](https://github.com/spatie/robots-txt). It includes robots.txt rules and crawl delays, robots meta tags, X-Robots-Tag headers, and a checker that combines them.

```rust
# use spatie_robots_txt::RobotsTxt;
let robots_txt = RobotsTxt::new("
  User-agent: *
  Disallow: /admin
  Crawl-delay: 1.5
");

robots_txt.allows_for("/admin", "google"); // false
robots_txt.crawl_delay(); // Some("1.5")
# assert!(!robots_txt.allows_for("/admin", "google"));
# assert_eq!(robots_txt.crawl_delay(), Some("1.5"));
```

Continue with [installation](installation-and-setup.md) and [usage](usage.md).
