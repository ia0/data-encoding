# Changelog

## 2.8.0-git

### Minor

- Add `BASE32_NOPAD_NOCASE` and `BASE32_NOPAD_VISUAL`

### Patch

- Add missing safety documentation and assertions for testing and fuzzing
- Move lints from `src/lib.rs` to `Cargo.toml` to ignore MSRV restrictions

## 2.7.0

### Minor

- Add `Encoding::encode_display()` for easier formatting (fixes #118)

### Patch

- Fix `needless-lifetimes` clippy lint
- Improve documentation of `Encoding::decode_len()` (fixes #111)

## 2.6.0

### Minor

- Add `BASE64_MIME_PERMISSIVE` constant to ignore trailing bits (fixes #102)

## 2.5.0

### Minor

- Add `Encoding::encode_write()` to encode to `core::fmt::Write` (fixes #69)
- Add `Encoder` and `Encoding::new_encoder()` for fragmented inputs (fixes #81)
- Make some functions `must_use`
- Bump MSRV from 1.47 to 1.48

### Patch

- Use clippy pedantic and add more lints
- Use `doc_auto_cfg` to show gating features (fixes #73)
- Hide the documentation of the `Encoding` implementation (fixes #75)

## 2.4.0

### Minor

- Bump MSRV from 1.46 to 1.47

### Patch

- Use TryInto for slice to array conversion
- Update documentation with reference to attack vectors
- Fix English in documentation

## 2.3.3

### Patch

- Specify MSRV to minimum that passes the tests
- Update CI badge in readme

## 2.3.2

### Patch

- Remove the `std` feature for `Display`

## 2.3.1

### Patch

- Maybe fix #33
- Update documentation

## 2.3.0

### Minor

- Add `encode_append` function

## 2.2.1

## 2.2.0

### Minor

- Add `std` and `alloc` features

### Patch

- Run `cargo clippy`

## 2.1.2

### Patch

- Switch to edition 2018

## 2.1.1

### Patch

- Increase test coverage for specifications
- Update readme and documentation
- Add maintenance-related badges to Cargo.toml

## 2.1.0

### Minor

- Accept duplicate but identical value specification
- Add `BASE32_DNSCURVE`
- Add `BASE32HEX_NOPAD` and `BASE32_DNSSEC`

### Patch

- Expose internal methods for `data-encoding-macro-internal`
- Include LICENSE file in cargo package

## 2.0.0

### Minor

- Add `BASE64URL_NOPAD`

## 2.0.0-rc.2

### Major

- Merge the `NoPad` and `Padded` types as `Encoding`
- Support for partial decoding in `decode_mut`

### Minor

- Support character translation while decoding
- Support ignoring characters while decoding
- Support column wrapping while encoding

### Patch

- Link to docs.rs for documentation
- Add a `lazy_static` example to the documentation
- Increase error message test coverage

## 2.0.0-rc.1

### Major

- Replace the `base`, `encode`, and `decode` modules by the types `NoPad` and
  `Padded`
- Remove the `base2`, `base4`, and `base8` modules
- Replace the `base16`/`hex`, `base32`, `base32hex`, `base64`, and `base64url`
  modules by the constants `HEXUPPER`, `BASE32`, `BASE32HEX`, `BASE64`, and
  `BASE64URL`

### Minor

- Support decoding concatenated padded inputs
- Support non-zero trailing bits
- Support non-canonical symbols
- Support least significant first bit-order
- Add `HEXLOWER` and `HEXLOWER_PERMISSIVE` constants

### Patch

- Increase performance of custom bases to match predefined ones

## 1.2.0

### Minor

- Add encoding and decoding functions without padding

## 1.1.2

### Patch

- Enhance performance by 15%
- Document the commands to build the example and run the benchmarks
- Add `generic` to the crate keywords

## 1.1.1

### Patch

- Update crate description
- Add link to the changelog in the readme file

## 1.1.0

### Minor

- Add `map` method to `decode::Error`
- Implement `Display` and `Error` for `ValidError` and `EqualError`
- Add a `base` function to each module

### Patch

- Update encode example
- Reword error messages
- Discuss implementation discrepancies in the documentation
- Test decoding differences with rustc-serialize
- Add the missing panic sections in the documentation
- Test that base specifications are valid

## 1.0.0
