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
| We acknowledge your report. | 5 working days |
| We tell you if we accept it as a vulnerability. | 15 working days |
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

## Test keys

The repository holds test keys and one test certificate authority. They are in `crates/access-token/examples/`, `crates/access-conformance/examples/`, `crates/reference-edge/examples/` and `crates/procurauth-pep/tests/data/`. They are public on purpose, and they protect nothing. A report about one of these keys is not a vulnerability.
