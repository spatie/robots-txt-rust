---
title: Installation
weight: 3
---

Requires Rust 1.89 or newer. Install Rust using [rustup](https://rustup.rs/).

You can install the package with Cargo:

```sh
cargo add spatie-robots-txt
```

HTTP convenience methods are synchronous and include rustls HTTPS support. Parsing supplied content does not perform network requests. No external executables or PHP runtime are required.
