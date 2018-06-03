This library provides macros to define compile-time byte slices and arrays from
encoded strings (using common bases like base64, base32, or hexadecimal, and
also custom bases). It also provides a macro to define compile-time custom
encodings to be used with the [data-encoding] crate.

If you were familiar with the [binary_macros] crate, this library is actually
[inspired][binary_macros_issue] from it.

If you use a nightly compiler, you may disable the "stable" feature:

```
data-encoding-macro = { version = "0.1.3", default-features = false }
```

### Examples

You can define a compile-time byte slice or array (using the `hexlower` or
`base64` macros for example):

```rust
const HELLO: &'static [u8] = &hexlower!("68656c6c6f");
const FOOBAR: &'static [u8] = &base64!("Zm9vYmFy");
// In nightly, it is possible to define an array instead of a slice:
hexlower_array!("const HELLO" = "68656c6c6f");
base64_array!("const FOOBAR" = "Zm9vYmFy");
```

You can define a compile-time custom encoding using the `new_encoding` macro:

```rust
const HEX: Encoding = new_encoding!{
    symbols: "0123456789abcdef",
    translate_from: "ABCDEF",
    translate_to: "abcdef",
};
const BASE64: Encoding = new_encoding!{
    symbols: "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/",
    padding: '=',
};
```

See the [documentation] for more details.

[binary_macros]: https://crates.io/crates/binary_macros
[binary_macros_issue]: https://github.com/ia0/data-encoding/issues/7
[data-encoding]: https://crates.io/crates/data-encoding
[documentation]: https://docs.rs/data-encoding-macro
