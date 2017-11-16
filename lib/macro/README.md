This library requires a nightly compiler.

It provides macros to define compile-time byte arrays from encoded strings
(using common bases like base64, base32, or hexadecimal, and also custom bases).
It also provides a macro to define compile-time custom encodings to be used with
the [data-encoding] crate at run-time.

If you were familiar with the [binary_macros] crate, this library is actually
[inspired][binary_macros_issue] from it.

### Examples

You can define a compile-time byte array using the `hexlower` or `base64`
macros:

```rust
hexlower!("const HELLO" = "68656c6c6f");
base64!("const FOOBAR" = "Zm9vYmFy");
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
