# Changelog

## 0.1.3-git

### Minor

- Add `Encoding::encode_align()` to decide where to split long inputs
- Document maximum input length for `{decode,encode}_len()`

### Patch

- Document the safety requirements of unsafe code and enable `undocumented_unsafe_blocks`
- Fix `redundant_explicit_links` rustdoc lint
- Rename lints in `Cargo.toml` to use underscores
- Use `iter().enumerate()` when possible
- Encode by the greatest multiple of blocks that fits 128 bits
- Use `doc_cfg` instead of `doc_auto_cfg`

## 0.1.2

### Patch

- Remove deprecated `authors` field from `Cargo.toml`

## 0.1.1

### Patch

- Fix documentation and readme formatting

## 0.1.0
