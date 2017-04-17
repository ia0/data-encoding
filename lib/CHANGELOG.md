# Changelog

### Patch

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
