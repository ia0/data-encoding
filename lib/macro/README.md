This library provides macros to define compile-time byte slices and arrays from
encoded strings (using common bases like base64, base32, or hexadecimal, and
also custom bases). It also provides a macro to define compile-time custom
encodings to be used with the [data-encoding] crate.

See the [documentation] for more details.

Until [rust-lang/cargo#7915](https://github.com/rust-lang/cargo/issues/7915) is
fixed, you may need to add the following to your `.cargo/config.toml` to use
this library in no-std or no-alloc environments:

```toml
[unstable]
features = ["host_dep"]
```

[data-encoding]: https://crates.io/crates/data-encoding
[documentation]: https://docs.rs/data-encoding-macro
