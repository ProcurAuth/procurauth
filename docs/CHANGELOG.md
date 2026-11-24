# Changelog

This file follows the layout of [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). The protocol, the MCP binding and the workspace have one version number each time. Each version is a git tag.

## [Unreleased]

## [1.1.1] - 2026-11-24

Version 1.1.1 is a patch release. The protocol stays at version 1.1 and no rule of the protocol changes.

### Changed

- The repository moved from the GitHub user `SCcagg5` to the GitHub organisation `ProcurAuth`. The address of the repository is now `https://github.com/ProcurAuth/procurauth`. GitHub redirects the old address of the repository.
- The `$id` of each JSON Schema has a new host: `https://procurauth.github.io/procurauth/schemas/`. The old host was `https://sccagg5.github.io/procurauth/schemas/`, and it does not answer now. The path of each schema and its content do not change. The tags `v1.0` and `v1.1` still hold the old `$id`. A program that compares the `$id` of a schema with a fixed text must use the new value. The constant `BASE_URI` of the crate `procurauth-schema-check` follows.
- The repository, the site and the manifests use the new addresses. The version of the protocol stays 1.1.
- `reference-service` and `reference-edge` read a configuration file of 4 MiB at most. A `_file` member of a text key names a file of 1 MiB at most. A `_file` member of a JSON document (`jwks`, `keys`, `key_format`) has the limit of 4 MiB. A larger file stops the program at start.
- `fake-pdp` and `access-conformance` refuse a hexadecimal value with a sign: a digest, a percent escape and the size of a chunk. Before, `+5` was a valid value.
- The workflow `ci` is fast: a job takes seconds to a few tens of seconds. The job `checks` runs `cargo fmt --check` and `cargo clippy` on the runner, with no Docker image build. The runner caches the crates and the build files of the dependencies, and only a push to `main` saves the cache. The job names `Policy (no image is published)` and `checks` do not change. The workflow `ci` has the trigger `workflow_dispatch`, and a new push cancels the run of the same pull request.
- The tests, the feature runs and the conformance runs left the workflow `ci`, because they take more than one minute. They run in the new workflow `full`, and `scripts/ci.sh` still runs them on a developer machine. A pull request no longer runs them: run `scripts/ci.sh` before you open a pull request that changes code.
- The release workflow runs the workflow `ci` before the builds. It compares the tag with `Cargo.toml` and with this file. The notes of a release are the section of its version in this file. The tar archives are deterministic.
- The development scripts use `--locked` for each cargo command. The tag of each development image is the start of the SHA-256 of its Dockerfile. `scripts/static.sh` builds for the architecture of the host: x86_64 or aarch64.
- The site uses the family `site.css` and `site.js`. The footer of each page links the home page of the organisation and the other programs.
- `scripts/check-docs.py` shows all the warnings. It does not count ids and section numbers as words.
- The normative Markdown has one Markdown paragraph for each numbered paragraph. The text does not change.

### Added

- The page `implementations.html`, the Open Graph image, the icons, the page 404, `sitemap.xml` and `robots.txt` of the site.
- `SECURITY.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, the issue forms, the template of a pull request and the file `CODEOWNERS`.
- The script `scripts/check-docs.py`. It checks the links, the site, the rules of ASD-STE100 and the hygiene of the files.
- `.editorconfig`, `.gitattributes` and `THIRD-PARTY-NOTICES.md`.
- The script `scripts/changelog-section.sh`. It prints the section of one version of a changelog.
- The workflow `ci` has the job `Policy (no image is published)`.
- The workflow `full`. It runs each night, on request (`workflow_dispatch`) and before a release. The job `tests` runs `scripts/ci.sh`. The job `msrv` runs `scripts/msrv.sh`, which no workflow ran before. The release workflow runs `full` before the builds.
- The section "Continuous integration" in `devdocs/architecture.md`, and the badge of the workflow `full` in the README.
- `SECURITY.md` has the section "Known limits" about ES256 signatures and RUSTSEC-2026-0387, and it lists each test value.

## [1.1] - 2026-11-23

Version 1.1 aligns the protocol with the portal, ClickHouse Dash and Stowr.

- Protocol 9.3 and 9.4. The claims `iat`, `exp` and `nbf` are JSON integers with digits only: no fraction, no exponent and no sign. A service refuses another form with HTTP 401, and it never rounds the value. This closes GitHub issue 1.
- Protocol 9.3 and 9.4. A token MAY have `nbf`. A service refuses a token whose `nbf` is later than its clock plus the skew. The edge does not need to give it.
- Protocol 9.4. When its key set holds more than one key, a service refuses a token without `kid`.
- Protocol 17.4. The approval API answers a request with no credential, or with an unknown credential, with HTTP 401 and `unauthorized`. A PDP answers 403 `forbidden` only when it knows the credential and gives no approval. The schema `approval-verdict` lists each code of 17.4.
- Protocol 6.1. The `unit` of a ceiling is a plural noun, for example `rows`. The `unit` of a budget is a singular noun, for example `request`. `access-lint` gives the warnings `ceiling_unit_plural` and `budget_unit_singular`. The examples and the data of the cases use `rows` and `seconds`.
- Protocol 3.9 and the new 10.7b. A PDP publishes the age bound of its directory in the metadata member `procurauth_directory_max_age_seconds`. A PDP with no copy of its directory publishes 1.
- Protocol 10.0, 10.1, the new 10.1c, 8.3, 9.4 and 1.3. A health-probe route needs no token and no question. Each other route that runs no action, also a static file, asks the action `access` on the service. The open point 6 of section 15 is Unused.
- MCP binding 5.7a. A 422 `explanation_required` and a 409 `cannot_supersede` give the tool error `invalid_argument`. A 422 `invalid_evaluation` makes the PEP ask one more time, and a second one gives `approval_expired`. Any other code stays HTTP 503.
- MCP binding section 12. The binding is the source for a ProcurAuth service. The ten open points are final rules, and a table lists each difference from the contract of the MCP servers. Section 11 says what the binding claims about COAZ-MCP.
- The protocol and the binding have the version 1.1. The workspace has the version 1.1.1.
- The conformance data has 245 cases: 127 for a PEP, 98 for a PDP and 20 for an edge. The new cases are U118 to U127, D97, D98 and E20.
- `IMPLEMENTATIONS.md` lists ClickHouse Dash and the portal. Neither links a library of this repository. The text that said that spec question 53 was still open is gone.
- Conformance runner. A token of a case has the members `number_forms` and `nbf`. A rule of a script has the member `times`. The runner has the endpoint `approval_requests`, the operations `read_health` and `use_no_action`, and the condition `no_pdp_credential`. The condition `single_key_set` holds for a PEP profile with one signing key.
- `procurauth-core` gives the number forms of a time claim, and it checks `nbf`. `access-token check` prints 14 checks, and `access-token make` has the option `--number-form`.
- `fake-pdp` publishes `procurauth_directory_max_age_seconds`. It gives a code of 17.4 for each error of the approval API, and it has the failure kinds `submit_*`.
- `procurauth-pep` has the optional feature `tls`, with rustls, aws-lc-rs, the system trust store and an optional CA file. It reads no environment variable. The flag `allow_plain_http` replaces `allow_insecure_pdp_link`. An `https` URL with this flag is a configuration error.
- `procurauth-pep` has an optional bounded cache of verified identity tokens, `token_cache_max_entries`, which is off by default. It reads `procurauth_directory_max_age_seconds`. It has `Pep::decide_route_without_action` for a route that runs no action, `Operation::also_all` and `Identity::from_checked_token`. The README shows the use from a plain HTTP service.
- `reference-service` has the routes `/health` and `/static/about.txt`. `reference-service` and `reference-edge` use the key `pdp.allow_plain_http`.
- `docs/design/libraries.md` describes a PDP library for the portal and a C interface of the PEP library for ClickHouse Dash. Version 1.1 does not build them.

### Earlier changes of 1.1

- Protocol 17.8: the `confirm` of a grant of a person has no effect on the questions of that person. It also has no effect on the action `mcp.session`. A tool never uses `mcp.session` (binding 3.6a).
- Protocol 6.1 and 7.1. The largest whole number of a limit is 2^53-1; a label and a description have 1 character or more. An action of a `resources` item lists the type of the node. A ceiling name starts with `max_`, or the document is bad. The schemas, the reader of the core and `access-lint` follow. The rule `ceiling_name` is now an error, and `empty_text`, `action_not_on_type` and `number_too_large` are new.
- Protocol 6.9, 10.7 and 10.7a. A PDP publishes four members. They give the idle time, the size of a request, the number of items and the time of a question with usage. The new paragraph 17.4a defines the twelve members `procurauth_approval_*`. The cases D90 and D91 test them.
- Protocol 17.2. The answer is `true` only when the PDP did both the count of the units and the consume of the approval. A failure never loses an approval, and it never leaves one consumed. The units MAY stay counted after a failure.
- Protocol 3.5, 5.3a, 11.1 and 1.3a. A condition of the owner that can change later gives `false` with no reason, never `unknown_subject`. The decision of an edge for `access` does not depend on the age of the action document. A PDP MAY serve built-in actions in a reserved service id, outside section 17.
- Protocol 17.1, 17.4, 17.5 and 17.9. An `evaluation_id` has 1024 bytes at most. The list of the error codes of the approval API is longer, and a PEP treats an unknown code as an error of the PDP. The bounds of `lease_s` are in the metadata. A `view` with no member is valid, and the title is the name of the action. The core reader gives `invalid_resource` for a bad id (case D23).
- Protocol 9.1. When the edge is an authentication server that a proxy calls, the proxy removes the text of the key. The edge documents how to configure it.
- Protocol 5.5a, 10.1a, 17.1, 17.7, 17.8 and section 12 and 15, and binding 3.7. The approval follows the action of the use alone. The new member `context.permission_only` marks the question of an other permission, and the PDP ignores `confirm` for it. A `confirm` on an other permission limits the scope and never makes the operation fail closed. This closes spec question 53.
- New cases D95 and D96 (`permission_only`), and the case U34 checks that a PEP sends the member. The schema `evaluation-context` has the member.
- Conformance cases U34 and U35. Each case has a second step with the operation `mcp_call_multi`. A PEP that is an MCP server, with a tool of two permissions, runs them. A PEP declares `use_multi`, `mcp_call_multi` or both. The ids stay. The test mock of the runner has a profile with the tool form only.
- Protocol 17.7, MCP binding 4.3a, 5.7 and section 10. The retry of a call after an approval repeats the same arguments and exactly one of `approval_ref`, `lease_s` and `until`. A service MAY refuse a retry with none. `tools/list` SHOULD describe the control arguments and the retry. The description of the tools of the reference service says it.
- New cases D90 to D94 and new directory files `session.json` and `bignumber.json`. The case D23 checks the reason `invalid_resource`.
- Protocol 17.3. The effect of an operation is `sha256:` and the SHA-256 of the JCS text (RFC 8785) of `{"v":1,"service","action","resource","op"}`. The numbers of `op` are integers from -(2^53-1) to 2^53-1, and any other number makes `op` invalid (`invalid_op`). The `view` of an approval holds JSON Pointers (RFC 6901) to `op`. Section 17 is new. The header of the protocol stays at version 1.0 until the release.
- The test vectors `conformance/vectors/effect.json` (groups V01 and V02, 24 vectors) and the schema `effect-vectors`. The test `effect_vectors` and a one-off program in Python computed each vector again with separate code. `COVERAGE.md` lists the groups.
- Protocol 5.2a. A service publishes only the actions that it serves: an action of a feature that the configuration turns off is not in the action document.
- The conformance case D75. A resource of depth 1 with no chain, or with an empty chain, gets `invalid_resource` (protocol 4.1, 11.1).
- Conformance runner. Each request of an MCP case has the headers `Mcp-Method` and, for `tools/call`, `Mcp-Name` (MCP server contract 2.5). The inline steps of 18 cases and the MCP operations send them. A service that refuses a request without them passes these cases.
- Conformance runner. The operation `use_ceiling` of a profile needs a request whose result does not go past `ceiling_value`. A PEP that applies the ceiling then passes the case U37. The case does not change.
- Protocol 17.1 to 17.12 and 5.2b, the approval of an effect. An action can need the approval of the owner when a grant has `confirm`. A question has `context.approval.effect`, and a `false` answer has one of four new reasons: `approval_required`, `approval_pending`, `approval_denied` and `approval_expired`.
- The PEP learns only a verdict, through a narrow API of three routes. The paragraphs 1.3, 10.1, 10.3, 11.1, 12, 14 and 15 change to fit it.
- MCP binding 3.5a is new. The new paragraphs 5.7, 5.7a and 5.7b give the result for a request that waits, the tool errors of an approval and the tool `get_action_status`. The case U30 is rewritten. Section 12 has two more open points.
- The schemas `approval-submission`, `approval-verdict` and `approval-view` are new. The schemas `evaluation-context`, `fake-pdp-data` and `conformance-case` have the new members.
- The conformance data has 26 new cases: U100 to U110, D76 to D89 and E19, and U30 is rewritten. The directory file `approvals.json` holds the data of the series D. The condition `no_approval_support` skips each case of the series D and E that cites section 17. It applies to a PDP or an edge that gives no approval. The runner reads the new members of a case.
- The conformance data has 225 cases: 117 for a PEP, 89 for a PDP and 19 for an edge. A PDP or an edge that gives no approval skips 15 of them: D76 to D89 and E19. A PEP skips none of the cases of section 17.

- `procurauth-core` has the module `effect`: the canonical text and the effect of 17.3, the check of `op` and the check of a `view` (17.9). The vectors of `conformance/vectors/effect.json` pass with it. The reader of the data reads the member `confirm` of a grant and the list `approvals`. The reasons and the member `evaluation_id` of 17.1 are in the answer model.
- `fake-pdp` gives the approvals of section 17. A grant with `confirm` makes the PDP answer `approval_required`, `approval_pending`, `approval_denied` or `approval_expired`. The PDP consumes the approval and takes the units in one decision (17.2). It serves the three routes of 17.4, with leases, revivals and a long poll. It publishes the bounds in twelve metadata members. The admin routes `/__fake/approvals` and `/__fake/approval-refs` stand in for the owner and for the channel of the agent.
- Conformance runner. The profile key `pdp.approvals` and the key `edge.keys.confirm_access` make the condition `no_approval_support` depend on the target. The runner runs D76 to D89 against `fake-pdp` and E19 against `reference-edge`. The test key `key:ann-k3` and the directory `access.json` have a grant of `access` with `confirm`.

- `procurauth-pep` has the module `approval`. `Pep::decide_approval` follows the steps of 17.7: the question with the effect and no usage, the submission, and the question with usage after the approval. It gives the new value `Outcome::ApprovalRequired`, and the refusal `Refusal::Approval`. The PEP keeps no answer to a question with an effect, and it never reads an intent, an explanation or an `approval_ref`. `Pep::approval_stats` counts what the PEP received from the approval API. The MCP helpers give the result of 5.7, the tool errors of 5.7a and the tool `get_action_status`.
- `reference-service` has the tool `append_rows` with the action `data.append`. It has the tool `get_action_status`. The tool `write_rows` and its request for input are gone. The results of a tool that ran have no `isError`.
- Conformance runner. The member `approval` of an entry of the profile gives the runner the `op` of each call. The runner computes the variables `effect`, `effect_2` and `effect_3`, and it runs the variants of the call. The test PDP serves the approval API and counts its requests for `pdp_approvals`. The reference service runs U30 and U100 to U110, and it passes them.
- MCP binding 3.1, 3.5a and 12. The text follows section 9 of the contract of the MCP servers. The service asks the decision point. A tool that only asks the owner needs no approval. The open point 7 says that the paragraphs 9.2, 9.6, 9.7 and 9.9 are Unused; the items 9 and 10 are new. `approval_rate_limited` has `retry_after` in seconds.

- Section 17 is mandatory for a service (protocol 1.3, 5.2b). Each action of a service follows it, and no action opts out. The member `approval` of an action is removed. A reader ignores it as an unknown member, and `access-lint` gives a warning. A service MUST NOT publish it. The header of the protocol stays at version 1.0, because section 17 is in no release.
- Protocol 17.3a is new: the `op` and the `view` of any action, a read and a navigation too, and the size limit `max_command_bytes`. Protocol 17.7 is rewritten. The PEP asks first as 10.1 says, with no effect, and it computes the effect on a reason of 17.1. Protocol 17.7a is new: a call that cannot wait gets HTTP 403 with the code `approval_cannot_wait`. Paragraphs 1.3, 5.2b, 10.1, 10.3 and 17.1 change, and the open points 7 and 8 are new.
- MCP binding 3.5, 3.5a, 3.6a, 5.1, 5.7, 5.7a and 5.7b change. Each tool follows section 17. The control arguments are optional on each tool, and `get_action_status` is mandatory for a service that has a tool. A call that asks only for input is not refused for a reason of 17.1.
- The conformance data has 7 new cases: U111 to U117. The cases U30 and U100 to U110 lose the condition `no_approval_support`. The runner runs each case that uses `mcp_call_approval` once for each action of the service. The profile key `pep.approvals` and the condition `no_approval_tool` are gone. The profile of a PEP with `mcp` has the member `approval_calls`.
- The vectors `read-with-default`, `read-empty-op-on-node` and `navigation-page` are new, in the group V01.
- `fake-pdp` has the test mode `confirm_all`. `procurauth-pep` asks as 10.1 says, and it builds `op` and `view` only on a reason of 17.1. `reference-service` sends each tool through the flow, and it answers HTTP 403 with `approval_cannot_wait` on each route that cannot wait.

## [1.0] - 2026-10-26

First version.

- The protocol and the MCP binding.
- The JSON schemas.
- The conformance data: 191 cases for a PEP, a PDP and an edge.
- Seven programs: `keycheck`, `access-token`, `access-lint`, `fake-pdp`, `access-conformance`, `reference-service` and `reference-edge`.
- The Linux programs are static files (musl). They run on any Linux distribution of the same processor.
- The PEP library `procurauth-pep` in Rust.

[Unreleased]: https://github.com/ProcurAuth/procurauth/compare/v1.1...HEAD
[1.1]: https://github.com/ProcurAuth/procurauth/compare/v1.0...v1.1
[1.0]: https://github.com/ProcurAuth/procurauth/releases/tag/v1.0
