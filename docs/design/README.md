# Design records

This directory holds the design records of the ProcurAuth tools. A record explains why the code and the data have their form. The protocol and the MCP binding are the normative texts. A record never changes them.

| File | Content |
| --- | --- |
| `decisions.md` | The technical decisions of the workspace, of the core, of the schemas and of the conformance data. |
| `decisions-fake-pdp.md` | The decisions of the tool `fake-pdp`. |
| `decisions-pep.md` | The decisions of the library `procurauth-pep` and of `reference-service`. |
| `spec-questions.md` | The places where the protocol or the binding has two possible meanings, and the meaning that the code uses. |
| `spec-questions-cases.md` | The same, for the conformance cases. |
| `spec-questions-pep.md` | The same, for the PEP library. |
| `spec-questions-integration.md` | The same, for the reference edge. |
| `core-requests.md` | What the tools need from `procurauth-core` and from the schemas. |
| `libraries.md` | The design of a PDP library for the portal and of a C interface of the PEP library for ClickHouse Dash. |
| `pep-vs-cases.md` | The functions of `procurauth-pep` that each case of the series U needs. |
| `case-fixes.md` | The conformance cases that the runner found wrong, and the change. |
| `integration-log.md` | The first runs of the runner against the reference programs. |
| `runner-integration.md` | What a service or an edge needs so that `access-conformance` can test it. |

A number such as "decision 171" is the number of an item in `decisions.md`. `decisions-pep.md`, `decisions-fake-pdp.md` and each `spec-questions` file have their own numbers.

To raise a question about the protocol, open an issue. Cite the paragraph.
