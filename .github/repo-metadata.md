# Repository metadata

This file lists the settings that a maintainer applies on GitHub with `gh`. The repository files do not apply them.

## About

- Description (105 characters): `Open specification for services that let AI agents act for people, with limits. A profile of AuthZEN 1.0.`
- Website: `https://procurauth.github.io/procurauth/`
- Social preview image: upload `docs/assets/og.png` (1200 by 630 pixels).

## Topics

`procurauth`, `ai-agents`, `authorization`, `authzen`, `mcp`, `model-context-protocol`, `specification`, `access-control`, `delegated-access`, `conformance-tests`, `json-schema`, `rust`

## Settings

- Pages: serve the branch `main`, from the folder `/docs`.
- Enable the private reports of vulnerabilities (Settings, Code security). `SECURITY.md` and `CODE_OF_CONDUCT.md` link to them.
- Enable the alerts of Dependabot. `.github/dependabot.yml` updates the actions of the workflows.
- Protect `main`: require the checks `Policy (no image is published)`, `checks` and `docs` before a merge.
- The workflow `full` runs each night. Do not require it for a merge: it is not part of a pull request. Watch its result. GitHub stops a scheduled workflow after 60 days with no activity in the repository.
- Do not publish a container image. The releases hold binaries and source only.

## Labels

Create these labels. The issue forms use them: `specification`, `bug`, `conformance`, `implementation`.

## Commands

```sh
gh repo edit ProcurAuth/procurauth \
  --description "Open specification for services that let AI agents act for people, with limits. A profile of AuthZEN 1.0." \
  --homepage "https://procurauth.github.io/procurauth/" \
  --add-topic procurauth --add-topic ai-agents --add-topic authorization --add-topic authzen --add-topic mcp \
  --add-topic model-context-protocol --add-topic specification --add-topic access-control \
  --add-topic delegated-access --add-topic conformance-tests --add-topic json-schema --add-topic rust
for label in specification bug conformance implementation; do gh label create "$label" --repo ProcurAuth/procurauth || true; done
```
