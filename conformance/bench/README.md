# Conformance bench

The bench tests the released programs of the family with the latest runner of ProcurAuth. It writes one result and one badge for each project. The badge in the README of a project shows the result.

## What the bench does

The bench takes the latest release of the tools of ProcurAuth. It takes the latest release of a project. It starts the released binary of the project with the test files of the same release. It runs the runner `access-conformance` against it. It builds nothing.

A project is compliant when no case fails, except the cases that its file `expected.txt` lists as failed. A case that the runner skips does not change the status. The result lists a skipped case that the file does not list, as a warning.

## When the bench runs

- A new release of ProcurAuth starts the bench for all projects (`workflow_run` of the workflow `release`).
- A release of a project starts the bench for this project. The last job of the release workflow of the project does this. The job never blocks the release, and its failure changes no other result.
- A person can start the workflow `bench` in the tab Actions. Give a project name, or `all`. Give a tag only for one project.

A job takes about 20 seconds for Stowr and about one minute for ClickHouse Dash, which needs a ClickHouse container.

## Where the results are

The workflow writes the branch `conformance-results` of this repository. Do not edit this branch by hand.

| Path | Content |
| --- | --- |
| `latest/PROJECT/badge.json` | The badge of the project, in the endpoint format of shields.io. |
| `latest/PROJECT/result.json` | The last result: the cases, the problems, the date and the time of the run. |
| `history/PROJECT/` | One result for each release of the project and each version of ProcurAuth. |

A README badge reads the file `badge.json`:

```text
https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/ProcurAuth/procurauth/conformance-results/latest/PROJECT/badge.json
```

The badge is green when the project is compliant. It shows the protocol version and the roles, for example `1.1 · PEP`. It is red when a case fails. It is grey when the bench could not run.

## Add a project

A project has two files in `projects/`:

- `NAME.json` has the data: `name`, `repo`, `asset` (the name of the archive, with `{version}`), `binary` (the file in the archive) and `series`. Each entry of `series` has the `role`, the `series` letter and the path of the file `expected.txt` in the repository of the project.
- `NAME.sh` starts the released binary and runs the runner. It reads five variables:
  - `BENCH_BIN` is the binary.
  - `BENCH_SRC` has the files of the release tag.
  - `BENCH_TOOLS` is the directory of the runner.
  - `BENCH_KEYS` has the test keys of ProcurAuth.
  - `BENCH_OUT` is the output directory.

  The script writes one report `SERIES.json` in `BENCH_OUT`, made with `--format json`. It exits with 0 when the runner ended, also when a case failed. It exits with 2 for a problem of the set-up.

Test the script on your machine with `conformance/bench/run.sh NAME TAG`. It needs `gh`, `git`, `python3` and Docker. Set `BENCH_TOOLS_DIR` to an extracted archive of the tools, and `BENCH_SPEC_TAG` to its tag.

## Limits

- The bench tests a PEP with the series U. A PDP needs a way to load its directories and to set its clock. An edge needs a proxy in front of it. The portal has neither interface in its released binary, so the bench does not test it today. Its own tests run the series D and E inside the process.
- The bench shows the result for the released binary. It does not show the result for each push to `main`.
