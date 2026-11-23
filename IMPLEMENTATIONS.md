# Implementations

This file lists the known implementations of ProcurAuth.

## In this repository

- `procurauth-pep`: a library in Rust for a PEP. It follows the requirements of the protocol for a service (section 10) and the approval flow of section 17 for each action.
- `reference-service`: a service that uses the library. It passes the series U of the conformance cases. Each of its actions goes through the approval flow of section 17.
- `reference-edge`: an edge. It passes the series E.
- `fake-pdp`: a test PDP. It passes the series D.

## Other implementations

| Name | Language | Role | Result | Library of this repository |
| --- | --- | --- | --- | --- |
| ClickHouse Dash | C++ | PEP, with an MCP endpoint | It passes the series U. | None. Dash has its own PEP, its own token checks and its own HTTP client. |
| portal | Rust | PDP and edge, with the approval inbox | It passes the series D and E. | None. The portal has its own decision code and its own effect hash. |

Each project gives its own result of `access-conformance`. The cases that version 1.1 adds (U118 to U127, D97, D98 and E20) need a new run of each project. Neither project links `procurauth-core` or `procurauth-pep` yet. `notes/design-libraries.md` describes a PDP library for the portal and a C interface of the PEP library for Dash.

A service conforms only when it follows section 17 for each action of its action document. An action that cannot ask for an approval makes the service not conformant.

To add one, open an issue with its name, its role (PEP, PDP or edge), a link and the result of `access-conformance`.
