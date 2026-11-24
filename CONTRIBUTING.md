# Contribute to ProcurAuth

Thank you for your interest. This repository holds the specification of ProcurAuth, its tools and its conformance data. This document tells how to propose a change.

## Ask a question or report an error

Open an issue with [the issue forms](https://github.com/ProcurAuth/procurauth/issues/new/choose). Cite the number of the paragraph of the protocol or of the MCP binding. Do not report a vulnerability in an issue. Read [SECURITY.md](SECURITY.md).

## Add an implementation

Open the form "Add an implementation". Give the name, the role (PEP, PDP or edge), a link and the result of `access-conformance`. The file [IMPLEMENTATIONS.md](IMPLEMENTATIONS.md) lists each implementation.

## Change the text

1. Open an issue first for a change of a rule. State the paragraph and the reason.
2. Keep the numbers of the paragraphs. A paragraph that leaves the text keeps its number, with the word "Unused".
3. Change or add a conformance case that cites the paragraph. Add the case to `conformance/COVERAGE.md`.
4. Write in ASD-STE100 Simplified Technical English. A procedural sentence has 20 words at most. A descriptive sentence has 25 words at most. A paragraph has six sentences at most.
5. Update `docs/CHANGELOG.md`.

The site in `docs/` repeats the protocol and the binding as HTML. Change the HTML in the same pull request as the Markdown.

## Change the code

1. Read `devdocs/architecture.md`.
2. Write one test for each paragraph that the code implements. Name the test after the paragraph.
3. Use Docker only. Do not install Rust on your machine.
4. Run the checks:

```sh
scripts/ci.sh                              # format, lint, tests and the conformance runs
python3 -I scripts/check-docs.py           # links, site, ASD-STE100 and hygiene of the files
```

The workspace needs Rust 1.89. `scripts/msrv.sh` checks it.

### What the CI runs

A job of the CI for a push or a pull request takes seconds to a few tens of seconds. The workflow `ci` checks the workflows, the format and clippy. The workflow `docs` runs `scripts/check-docs.py`.

The tests, the conformance runs and the MSRV check take more than one minute. They do not run for a pull request. The workflow `full` runs them each night, on request (Actions, `full`, Run workflow) and before a release. You run the same checks with `scripts/ci.sh` and `scripts/msrv.sh`. Run them before you open a pull request that changes code.

## Rules for each file

- Use `example.com` and the address range `203.0.113.0/24` in examples. Never write a real host name, a real address or a real secret.
- Do not add a container image to the release. A release holds binaries and source only.
- Do not name a product in the protocol or in the binding. They are neutral.

## Pull requests

Open a pull request against `main`. Complete the template. The CI must pass. A contribution is under the licence of the files that it changes: Apache-2.0 for code and CC BY 4.0 for texts.

## Code of conduct

This project follows [the code of conduct](CODE_OF_CONDUCT.md).
