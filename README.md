# Data-encoding

[![Latest Version][version_badge]][crate]
[![Documentation][documentation_badge]][documentation]
[![Latest License][license_badge]][license]
[![Build Status][travis_badge]][travis]
[![Build Status][appveyor_badge]][appveyor]
[![Coverage Status][coveralls_badge]][coveralls]

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

[appveyor]: https://ci.appveyor.com/project/ia0/data-encoding
[appveyor_badge]:https://ci.appveyor.com/api/projects/status/wm4ga69xnlriukhl/branch/master?svg=true
[changelog]: https://github.com/ia0/data-encoding/blob/master/lib/CHANGELOG.md
[coveralls]: https://coveralls.io/github/ia0/data-encoding
[coveralls_badge]: https://coveralls.io/repos/ia0/data-encoding/badge.svg?branch=master&service=github
[crate]: https://crates.io/crates/data-encoding
[documentation]: https://docs.rs/data-encoding
[documentation_badge]: https://docs.rs/data-encoding/badge.svg
[license]: https://github.com/ia0/data-encoding/blob/master/LICENSE
[license_badge]: https://img.shields.io/crates/l/data-encoding.svg
[travis]: https://travis-ci.org/ia0/data-encoding
[travis_badge]: https://travis-ci.org/ia0/data-encoding.svg?branch=master
[version_badge]: https://img.shields.io/crates/v/data-encoding.svg
