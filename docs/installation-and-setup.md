---
title: Installation
weight: 3
---

Requires Rust 1.89 or newer. Install Rust using [rustup](https://rustup.rs/).

Before publication, use the public repository:

```toml
[dependencies]
spatie-robots-txt = { git = "https://github.com/spatie/robots-txt-rust", branch = "main" }
```

HTTP convenience methods are synchronous and include rustls HTTPS support. Parsing supplied content does not perform network requests. No external executables or PHP runtime are required.
