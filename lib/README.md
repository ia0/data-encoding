## Common use-cases

This library provides the following common encodings:

- `HEXLOWER`: lowercase hexadecimal
- `HEXLOWER_PERMISSIVE`: lowercase hexadecimal with case-insensible decoding
- `HEXUPPER`: uppercase hexadecimal
- `HEXUPPER_PERMISSIVE`: uppercase hexadecimal with case-insensible decoding
- `BASE32`: RFC4648 base32
- `BASE32_NOPAD`: RFC4648 base32 without padding
- `BASE32HEX`: RFC4648 base32hex
- `BASE64`: RFC4648 base64
- `BASE64_NOPAD`: RFC4648 base64 without padding
- `BASE64URL`: RFC4648 base64url
- `BASE64_MIME`: RFC2045-like base64

Typical usage looks like:

```rust
// allocating functions
BASE64.encode(&input_to_encode)
HEXLOWER.decode(&input_to_decode)
// in-place functions
BASE32.encode_mut(&input_to_encode, &mut encoded_output)
BASE64_URL.decode_mut(&input_to_decode, &mut decoded_output)
```

See the [documentation] or the [changelog] for more details.

## Custom use-cases

This library also provides the possibility to define custom little-endian ASCII
base-conversion encodings for bases of size 2, 4, 8, 16, 32, and 64 (for which
all above use-cases are simply instances). It supports:

- padded and non-padded encodings
- canonical encodings (trailing bits are checked)
- in-place encoding and decoding functions
- partial decoding functions
- character translation (for case-insensitivity for example)
- most and least significant bit-order
- ignoring characters when decoding
- wrapping the output when encoding

The typical definition of a custom encoding looks like:

```rust
lazy_static! {
    static ref DNSCURVE: data_encoding::Encoding = {
        use data_encoding::{Specification, BitOrder};
        let mut spec = Specification::new();
        spec.symbols.push_str("0123456789bcdfghjklmnpqrstuvwxyz");
        spec.translate.from.push_str("BCDFGHJKLMNPQRSTUVWXYZ");
        spec.translate.to.push_str("bcdfghjklmnpqrstuvwxyz");
        spec.bit_order = BitOrder::LeastSignificantFirst;
        spec.encoding().unwrap()
    };
}
```

See the [documentation] or the [changelog] for more details.

## Performance

The performance of the encoding and decoding functions (for both common and
custom encodings) are similar to existing implementations in C, Rust, and other
high-performance languages (see how to run the benchmarks on [github]).

## Swiss-knife binary

This crate is a library. If you are looking for the [binary] using this library,
see the installation instructions on [github].

[binary]: https://crates.io/crates/data-encoding-bin
[changelog]: https://github.com/ia0/data-encoding/blob/master/lib/CHANGELOG.md
[documentation]: https://docs.rs/data-encoding
[github]: https://github.com/ia0/data-encoding
