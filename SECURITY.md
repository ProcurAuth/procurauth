# Security policy

This document tells how to report a vulnerability in ProcurAuth. It also tells what the project covers.

## Report a vulnerability

Do not open a public issue for a vulnerability. Use the private reports of GitHub.

1. Open [the page to report a vulnerability](https://github.com/ProcurAuth/procurauth/security/advisories/new).
2. Describe the problem. Name the version, the file or the paragraph of the protocol, and the steps to repeat it.
3. Add a test or a proof of concept if you have one.

Only the maintainers of the repository can read your report. We never ask for an e-mail address or for a secret. If the page is not available, open a public issue with the title "Security contact needed". Write no detail in it. A maintainer then starts a private channel.

## Response times

These times are targets for a project that has no paid staff.

| Step | Target |
| --- | --- |
| We acknowledge your report. | 5 business days |
| We tell you if we accept it as a vulnerability. | 15 business days |
| We publish a fix or a corrected text. | 90 days after the report, or sooner |

We ask you to wait for the fix before you publish details. We give credit in the advisory, unless you ask us not to.

## Supported versions

| Version | Supported |
| --- | --- |
| The latest release (1.1.x) | Yes |
| The branch `main` | Yes |
| Older releases | No. Update to the latest release. |

## What the project covers

The project covers three kinds of problem:

- **The protocol and the MCP binding.** A rule that lets a key have more rights than its owner is a vulnerability. So is a rule that makes a service leak a grant, a limit or a key.
- **The reference programs and the library.** `procurauth-pep`, `reference-service`, `reference-edge`, `fake-pdp`, `access-token`, `access-lint`, `keycheck` and `access-conformance` can have a vulnerability like any program. A bypass of a check of the identity token in `procurauth-pep` is in scope.
- **The conformance cases.** A case that passes a PEP that has a security fault is a defect. Report it as a vulnerability when the fault is serious.

## What the project does not cover

- A product that follows the protocol, such as the portal, Stowr, Hune or ClickHouse Dash. Report the problem to that project.
- A configuration of a deployment that does not follow the protocol or the documents.
- `fake-pdp` and `reference-edge` as production servers. They are test programs. They have no TLS code, and the documents say so.

## Known limits

`access-token`, `access-conformance` and `reference-edge` can sign an identity token with ES256. For ES256, they use the crate `p256` 0.13 on `crypto-bigint` 0.5.

The advisory [RUSTSEC-2026-0387](https://rustsec.org/advisories/RUSTSEC-2026-0387.html) applies to `crypto-bigint` 0.5. Some of its operations on secret values do not run in constant time. This can let an attacker who measures the time of many signatures learn data about the private key. The check of an ES256 signature uses only a public key. The advisory does not apply to it.

Use Ed25519 for a key that protects real data. This limit stays until the project uses a version of `p256` with a corrected `crypto-bigint`.

## Test values

The repository holds test keys, test certificates and test values. They are public on purpose, and they protect nothing. A report about one of these values is not a vulnerability.

- The private keys for the identity token, Ed25519 and P-256:
  - `crates/access-token/examples/test-edge-ed25519.pem`
  - `crates/access-token/examples/test-edge-es256.pem`
  - `crates/access-conformance/examples/test-edge-ed25519.pem`
  - `crates/access-conformance/examples/test-edge-es256.pem`
- The public keys of the identity token, in PEM and JWKS form:
  - `crates/access-token/examples/test-edge-ed25519.pub.pem`
  - `crates/access-token/examples/test-edge-es256.pub.pem`
  - `crates/access-token/examples/service-two-keys.json`
  - `crates/access-conformance/examples/edge-jwks.json`
  - `crates/access-conformance/examples/edge-jwks-one.json`
- The test certificate authorities, the server certificate and its private key for the TLS tests of `procurauth-pep`:
  - `crates/procurauth-pep/tests/data/tls-test-ca.pem`
  - `crates/procurauth-pep/tests/data/tls-test-other-ca.pem`
  - `crates/procurauth-pep/tests/data/tls-test-server.pem`
  - `crates/procurauth-pep/tests/data/tls-test-server-key.pem`
- The test API keys and the test credential of the conformance runs:
  - `crates/access-conformance/examples/edge-valid-key.txt`
  - `crates/access-conformance/examples/edge-confirm-key.txt`
  - `crates/access-conformance/examples/edge-no-access-key.txt`
  - `crates/access-conformance/examples/edge-unknown-key.txt`
  - `crates/access-conformance/examples/pdp-credential.txt`
- The test credential of `fake-pdp`: `crates/fake-pdp/examples/pep-credential.txt`.
- The digests and the format of the test API keys of `reference-edge`: `crates/reference-edge/examples/keys.test-only.json` and `crates/reference-edge/examples/key-format.test-only.json`.
- The test API keys in the tests: `crates/reference-edge/tests/common/mod.rs`, `crates/reference-edge/tests/edge.rs` and `crates/access-conformance/tests/common/mock_edge.rs`.
