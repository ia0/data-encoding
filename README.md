# Data-encoding

[![Latest Version][1]][crate]
[![Documentation](https://docs.rs/data-encoding/badge.svg)][documentation]
[![Latest License][2]][crate]
[![Build Status][3]](https://travis-ci.org/ia0/data-encoding)
[![Coverage Status][4]](https://coveralls.io/github/ia0/data-encoding?branch=master)

This [crate] provides little-endian ASCII base-conversion encodings for bases of
size 2, 4, 8, 16, 32, and 64. It supports both padded and non-padded encodings.
It supports canonical encodings (trailing bits are checked). It supports
in-place encoding and decoding functions. It supports non-canonical symbols. And
it supports both most and least significant bit-order. The performance of the
encoding and decoding functions are similar to existing implementations (see how
to run the benchmarks below). See the [documentation] or the [changelog] for
more details.

You can run `make` or `make help` to list the Makefile targets:

```
Targets:
    make install   # installs the binary
    make bench     # runs all benchmarks
    make test      # runs all tests
    make doc       # generates the library documentation
    make help      # shows this help
```

You can also run `cargo install data-encoding-bin` to install the latest version
of the binary published on `crates.io`. This does not require to clone the
repository.

Once installed, you can run `data-encoding --help` to see the usage:

```
Usage: data-encoding [<options>]

Options:
    --mode <mode>       {encode|decode[_concat]|info}
    --base <base>       {16|hex|{32|32hex|64|64url}[_nopad]|custom}
    --input <file>      uses <file> instead of standard input
    --output <file>     uses <file> instead of standard output
    --block <size>      reads blocks of <size> bytes
    --skip              when decoding, skips newlines
    --wrap <cols>       when encoding, wraps every <cols> characters
    --symbols <symbols> custom base uses <symbols>
    --padding <padding> custom base pads with <padding>
    --translate <new><old>
                        when decoding, custom base translates <new> as <old>
    --ignore_trailing_bits
                        when decoding, custom base ignores trailing bits
    --least_significant_bit_first
                        custom base bit-order is least significant bit first

Examples:
    # Encode using the RFC4648 base64 encoding
    data-encoding --mode=encode --base=64        # with padding
    data-encoding --mode=encode --base=64_nopad  # without padding

    # Show base information for the permissive hex encoding
    data-encoding --mode=info --base=hex

    # Decode using a custom hexadecimal encoding
    data-encoding --mode=decode --base=custom --symbols=0123456789abcdef \
        --translate=ABCDEFabcdef

    # Decode using the DNSCurve base32 encoding
    data-encoding --mode=decode --base=custom \
        --symbols=0123456789bcdfghjklmnpqrstuvwxyz \
        --translate=BCDFGHJKLMNPQRSTUVWXYZbcdfghjklmnpqrstuvwxyz \
        --least_significant_bit_first
```

[1]: https://img.shields.io/crates/v/data-encoding.svg
[2]: https://img.shields.io/crates/l/data-encoding.svg
[3]: https://travis-ci.org/ia0/data-encoding.svg?branch=master
[4]: https://coveralls.io/repos/ia0/data-encoding/badge.svg?branch=master&service=github
[changelog]: https://github.com/ia0/data-encoding/blob/master/lib/CHANGELOG.md
[crate]: https://crates.io/crates/data-encoding
[documentation]: https://docs.rs/data-encoding
