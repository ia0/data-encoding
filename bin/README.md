This binary is a wrapper around the `data-encoding` [library].

## Installation

You can run `make install` to install the binary from the [github] repository.
By default, it will be installed as `~/.cargo/bin/data-encoding`. You can also
run `cargo install data-encoding-bin` to install the latest version published on
`crates.io`. This does not require to clone the repository.

## Usage

You can run `data-encoding --help` (assuming `~/.cargo/bin` is in your `PATH`
environment variable) to see the usage:

```
Usage: data-encoding --mode=<mode> --base=<base> [<options>]
Usage: data-encoding --mode=<mode> --symbols=<symbols> [<options>]

Options:
    -m, --mode <mode>   {encode|decode|describe}
    -b, --base <base>   {16|hex|32|32hex|64|64url}
    -i, --input <file>  read from <file> instead of standard input
    -o, --output <file> write to <file> instead of standard output
        --block <size>  read blocks of about <size> bytes
    -p, --padding <padding>
                        pad with <padding>
    -g, --ignore <ignore>
                        when decoding, ignore characters in <ignore>
    -w, --width <cols>  when encoding, wrap every <cols> characters
    -s, --separator <separator>
                        when encoding, wrap with <separator>
        --symbols <symbols>
                        define a custom base using <symbols>
        --translate <new><old>
                        when decoding, translate <new> as <old>
        --ignore_trailing_bits 
                        when decoding, ignore non-zero trailing bits
        --least_significant_bit_first 
                        use least significant bit first bit-order

Examples:
    # Encode using the RFC4648 base64 encoding
    data-encoding -mencode -b64     # without padding
    data-encoding -mencode -b64 -p= # with padding

    # Encode using the MIME base64 encoding
    data-encoding -mencode -b64 -p= -w76 -s$'\r\n'

    # Show base information for the permissive hexadecimal encoding
    data-encoding --mode=describe --base=hex

    # Decode using the DNSCurve base32 encoding
    data-encoding -mdecode \
        --symbols=0123456789bcdfghjklmnpqrstuvwxyz \
        --translate=BCDFGHJKLMNPQRSTUVWXYZbcdfghjklmnpqrstuvwxyz \
        --least_significant_bit_first
```

## Performance

The performance of this binary is similar or faster than the GNU `base64`
program (see how to run the benchmarks on [github]).

[library]: https://crates.io/crates/data-encoding
[github]: https://github.com/ia0/data-encoding
