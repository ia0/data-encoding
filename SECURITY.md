# Security Policy

## Supported Versions

Only report a vulnerability for the latest version (according to their branch) of supported crates:

| Name                           | Directory            | Branch | Supported |
|--------------------------------|----------------------|--------|-----------|
| `data-encoding`                | `lib`                | `main` | Yes       |
| `data-encoding-bin`            | `bin`                | `main` | Yes       |
| `data-encoding-macro`          | `lib/macro`          | `main` | Yes       |
| `data-encoding-macro-internal` | `lib/macro/internal` | `main` | No        |
| `data-encoding-v3`             | `lib/v3`             | `main` | No        |

Notes for unsupported crates:

- `data-encoding-macro-internal` is an implementation detail of `data-encoding-macro`. A bug only
  reachable by using it directly is not a vulnerability, whereas a bug reachable through
  `data-encoding-macro` is.
- `data-encoding-v3` is a development crate for `data-encoding`. A bug only reachable in
  `data-encoding-v3` is not a vulnerability, whereas the same bug reachable in `data-encoding` is.

## Reporting a Vulnerability

Click **Report a vulnerability** from the [Security Advisories][advisories] page. You can expect an
acknowledgment within 24 hours and triage assessment within 7 days. If the vulnerability is
accepted, we will work with you for a patch release.

Do not report security vulnerabilities through public GitHub issues or pull requests.

[advisories]: https://github.com/ia0/data-encoding/security/advisories
