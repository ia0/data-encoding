# Changelog

## 0.3.11-git

### Patch

- Fix panic when printing the help, version, or specification output fails
- Fix panic when the block value is too large
- Fix inaccurate error positions when decoding by small blocks with ignored characters
- Fix panic when the width value is not a multiple of the base or is too large
- Fix write errors being silently ignored when flushing the end of the output
- Fix incorrect decoding for encodings with ignored characters and without padding
- Fix decoding silently stopping when the input buffer fills up with ignored characters
- Update `data-encoding` version

## 0.3.10

### Patch

- Remove deprecated `authors` field from `Cargo.toml`
- Update `data-encoding` version

## 0.3.9

### Patch

- Update `data-encoding` version

## 0.3.8

### Patch

- Update `data-encoding` version

## 0.3.7

### Patch

- Update `data-encoding` version

## 0.3.6

### Patch

- Update `data-encoding` version

## 0.3.5

### Minor

- Specify MSRV of 1.81

### Patch

- Update `data-encoding` version

## 0.3.4

### Patch

- Update `data-encoding` version

## 0.3.3

### Patch

- Update `data-encoding` version

## 0.3.2

### Patch

- Fix crash with invalid length for base16

## 0.3.1

### Minor

- Add `--version` flag

## 0.3.0

### Major

- Rename the mode `describe` to `specification`

### Minor

- Support prefix for modes

## 0.2.3

### Patch

- Code maintenance

## 0.2.2

### Patch

- Switch to edition 2018
- Include LICENSE file in cargo package

## 0.2.1

### Minor

- Use data-encoding-2.0.0

## 0.2.0
