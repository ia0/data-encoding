:warning: It is strongly **discouraged** to use this crate _at this time_. There are no guarantees
whatsoever (correctness, stability, documentation, etc). This warning will be updated as new
versions are published.

This crate is the development branch of `data-encoding@3.0.0`. It is provided as a workaround to the
[Cargo pre-release issues](https://github.com/rust-lang/cargo/issues/2222#issuecomment-2509149376).
It will obey the following rules:
- The initial version is `data-encoding-v3@0.1.0`
- The final version is `data-encoding-v3@1.0.0`
- Version `data-encoding-v3@0.x.y` represents `data-encoding@3.0.0-x.y`
- Version `data-encoding-v3@1.0.0` corresponds to `data-encoding@3.0.0`
- Only the final version will be published to `data-encoding`

In particular, the `data-encoding` crate won't have any `3.0.0` pre-release version. This crate
should be used instead. To minimize code changes (v3 won't break simple usages of v2), you can
modify your `Cargo.toml` from using `data-encoding`:

```toml
[dependencies]
data-encoding = "2.9.0"
```

to using `data-encoding-v3` while preserving the crate name:

```toml
[dependencies]
data-encoding = { package = "data-encoding-v3", version = "0.1.0" }
```

When `data-encoding-v3` reaches `1.0.0`, then `data-encoding` will reach `3.0.0` too, and the
`Cargo.toml` should use `data-encoding` again:

```toml
[dependencies]
data-encoding = "3.0.0"
```
