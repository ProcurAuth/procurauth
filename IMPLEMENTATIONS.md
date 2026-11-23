# Implementations

This file lists the known implementations of ProcurAuth. The data comes from the repository of each program at the date in the next line. A result belongs to the commit of this repository that the program pins. A later version can add cases, so a result can change.

Date of the data: 2026-11-23. Version of ProcurAuth: 1.1. The conformance data of 1.1 has 245 cases. The series U has 127 cases for a PEP. The series D has 98 cases for a PDP. The series E has 20 cases for an edge.

## In this repository

- `procurauth-pep`: a library in Rust for a PEP. It follows the requirements of the protocol for a service (section 10) and the approval flow of section 17 for each action.
- `reference-service`: a service that uses the library. It passes the series U: 127 cases pass when the action `mcp.session` has a budget. Otherwise 126 pass and U95 is skipped.
- `reference-edge`: an edge. It passes the series E: 20 cases.
- `fake-pdp`: a test PDP. It passes the series D: 98 cases as a separate process. In the process of the runner, 97 pass and D97 is skipped.

`scripts/conformance.sh all` runs these results again.

## The ProcurAuth family

The organisation [ProcurAuth](https://github.com/ProcurAuth) holds three programs that follow the protocol. ClickHouse Dash is in the organisation Circular-fi.

| Name | Language | Role | Version | Follows ProcurAuth | Pin of this repository |
| --- | --- | --- | --- | --- | --- |
| [portal](https://github.com/ProcurAuth/portal) | Rust | Backend, edge and PDP, with the approval inbox | 5.6.0 | 1.1 | `c820b4ac0e37c4769394876105fd549a6c8b38cc`, the tag `v1.1`. |
| [Stowr](https://github.com/ProcurAuth/stowr) | Rust | PEP: browser for S3 and Google Cloud Storage | 0.3.0 | 1.1 | `c820b4ac0e37c4769394876105fd549a6c8b38cc`, the tag `v1.1`. |
| [Hune](https://github.com/ProcurAuth/hune) | Rust | PEP: dashboards | 0.1.1 | The 1.0 line | `058563cfbc3536b9b850255d953568ef0a5cabbe`, 18 commits after the tag `v1.0`. |
| [ClickHouse Dash](https://github.com/Circular-fi/Clickhouse-dash) | C++ | PEP with an MCP endpoint | 1.6.0 | 1.1 | `c820b4ac0e37c4769394876105fd549a6c8b38cc`, the tag `v1.1`. |

### Results

| Name | Series | Result | Note |
| --- | --- | --- | --- |
| portal | D and E | The CI runs the 98 cases of the series D and the 20 cases of the series E. | The portal is not a PEP, so it has no series U result. The script `demo/conformance/run-conformance-u.sh` runs the series U against the demo service `notes`. The portal repository records no count of this run. |
| Stowr | U | 127 cases: 81 passed, 0 failed, 46 skipped. | 45 skipped cases need an MCP endpoint. U84 needs a use of the service itself. Stowr has no MCP endpoint. |
| Hune | U | 110 cases: 60 passed, 0 failed, 50 skipped. The run is of 2026-11-11. | The run is on the 1.0 line. The cases U111 to U127 need a new run. Hune has no MCP endpoint. |
| ClickHouse Dash | U | 127 cases: 126 passed, 0 failed, 1 skipped. | The skipped case is U117. Dash has no tool that asks for input. |

### Libraries

| Name | Library of this repository |
| --- | --- |
| Stowr | `procurauth-pep` and `procurauth-core`, as git dependencies at the pinned commit. |
| Hune | `procurauth-pep` and `procurauth-core`, as git dependencies at the pinned commit. |
| portal | None. The portal has its own decision code and its own effect hash. |
| ClickHouse Dash | None. Dash has its own PEP, its own token checks and its own HTTP client. |

`docs/design/libraries.md` describes a PDP library for the portal and a C interface of the PEP library for ClickHouse Dash. Neither library exists yet.

## Rules for the list

A service conforms only when it follows section 17 for each action of its action document. An action that cannot ask for an approval makes the service not conformant.

Each project runs `access-conformance` itself. Skipped cases are not failures. The reason of each skipped case is in the output of the runner.

## Add your implementation

Open the form "Add an implementation" in [the issue tracker](https://github.com/ProcurAuth/procurauth/issues/new/choose). Give the name, the role (PEP, PDP or edge) and a link. Give the version of ProcurAuth and the commit that you tested. Add the result of `access-conformance`.
