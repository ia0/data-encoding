# Generic data encoding functions

[![Build Status](https://travis-ci.org/ia0/data-encoding.svg?branch=master)](https://travis-ci.org/ia0/data-encoding)

This [crate](https://crates.io/crates/data-encoding) provides generic
encoding and decoding functions with instances for common bases
(base64, base32, hex, etc.). It also provides a file encoding and
decoding binary example exercising the library quite exhaustively.

## About the library

The implementation is meant:
- to guarantee mathematical properties,
- to conform to RFC 4648 (base64, base32, hex, etc.),
- to be efficient (wrt. the base64 GNU program), and
- to give choice between allocating and in-place functions.

For more information, please refer to the
[documentation](http://ia0.github.io/data-encoding/data_encoding) or
to the
[changelog](https://github.com/ia0/data-encoding/blob/master/CHANGELOG.md).

## About the binary

The binary can be build with `make encode`. Here is its usage:

```
Usage: encode [<options>]

Options:
    -b, --base <name>   select base 2, 4, 8, 16 (or hex), 32, 32hex, 64, or
                        64url if <name> matches. Otherwise, build base using
                        the first character of <name> as padding and the
                        remaining characters as symbols in value order.
                        Default is 64.
    -d, --decode        decode data. Default is to encode data.
    -i, --input <file>  use <file> as input. Default is to use standard input.
    -o, --output <file> use <file> as output. Default is to use standard
                        output.
    -s, --skip          when decoding, skip newlines. Default is to accept
                        only well-formed input.
    -w, --wrap <cols>   when encoding, add newlines every <cols> characters
                        and at the end. Default is to produce well-formed
                        output.

Examples:
    encode
    encode -d
    encode -b32 -d -s
    encode -b=0123456789abcdef -w76
```
