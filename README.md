# Generic data encoding functions

[![Build Status](https://travis-ci.org/ia0/data-encoding.svg?branch=master)](https://travis-ci.org/ia0/data-encoding)

This library provides generic encoding and decoding functions with
instances for common bases (base64, base32, hex, etc.). It also
provides a file encoding and decoding binary example exercising the
library quite exhaustively.

The implementation is meant:
- to guarantee mathematical properties,
- to conform to RFC 4648 (base64, base32, hex, etc.),
- to be efficient (wrt. the base64 GNU program), and
- to give choice between allocating and in-place functions.

For more information, please refer to the [online
documentation](http://ia0.github.io/data-encoding/data_encoding).
