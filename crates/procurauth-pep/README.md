# procurauth-pep

`procurauth-pep` is the reference PEP library of ProcurAuth. A service that follows the protocol is a PEP (policy enforcement point). The library gives the seven steps of paragraph 10.0 as a small API.

The API does not depend on a web framework. The application gives the headers of a request. The library gives back the identity, the ceilings to apply, or the refusal with its HTTP status.

The crate `reference-service` is a small service that uses this library. It is the test subject of the series U of the conformance cases.

## Build and test

Build with Docker only. Do not install Rust on the host.

```text
scripts/cargo.sh build -p procurauth-pep
scripts/cargo.sh test -p procurauth-pep
scripts/cargo.sh test -p procurauth-pep --no-default-features
scripts/cargo.sh test -p procurauth-pep --features tls
```

The tests start the test PDP `fake-pdp` in the same process. They use real HTTP on the loopback address only. The tests of the feature `tls` start a small PDP over TLS on the loopback address. Its CA, certificate and key are in `tests/data`. They are for the tests only.

## Features

| Feature | Default | Content |
| --- | --- | --- |
| `mcp` | on | The module `mcp`: the helpers of the MCP binding. |
| `tls` | off | The module `tls`: `TlsTransport`, an HTTP/1.1 client over TLS 1.2 or 1.3 for the link to the PDP. It uses rustls with the crypto provider aws-lc-rs. |

Without the feature `tls`, the build compiles no TLS crate. With it, the crate `aws-lc-sys` compiles C code. The image `rust:1-bookworm` of `scripts/cargo.sh` has the C compiler that it needs. The build uses no `cmake`, no NASM and no bindgen on Linux x86_64.

## The seven steps

| Step | What the PEP does | Where |
| --- | --- | --- |
| 1 | Check the identity token, and send `sub` as the subject. | `Pep::authenticate` |
| 2 | Refuse a request with no valid token. | `Refusal::to_http` |
| 3 | Give each use one action and one resource with its chain. | `Operation`, `ActionDocumentBuilder` |
| 4 | Ask the PDP. Keep answers for `cache_ttl` at most. | `Pep::decide`, `Pep::check`, `Pep::display` |
| 5 | Send `context.usage` for a use of an action that lists a budget. | `Pep::decide` |
| 6 | Apply each ceiling of `context.limits`. | `Ceilings` |
| 7 | Map each refusal with 10.3. | `Refusal::to_http` |

## Use

An application does these jobs at start. It reads its own configuration. It builds a `PepConfig` and an action document. It gives them to `Pep::new`, with a transport for the link to the PDP.

For each request, the application does these jobs.

1. Call `pep.authenticate(headers)`. A refusal gives HTTP 401.
2. Build the `Operation` from its own data: the action, the node and the chain of the node (4.5).
3. Call `pep.decide(&identity, &operation)`.
4. Send `refusal.to_http()` for a refusal, or apply the ceilings of the `Allowed` value.

```rust
use procurauth_pep::operation::{Operation, node};

let who = pep.authenticate(request_headers)?;
let table = node("datastore", &[("host", "prod"), ("database", "test2"), ("table", "orders")])?;
match pep.decide(&who, &Operation::new("data.read", table)).into_result() {
    Ok(allowed) => {
        // The ceiling lowers the number of rows that the client asked for (7.5).
        let rows = allowed.ceilings().cap("max_rows", requested_rows).value;
    }
    Err(refusal) => { /* send refusal.to_http() */ }
}
```

The type `Refusal` has the status of 10.3. The reason of the PDP stays in the log text of the refusal. The body of the reply never has it.

| Answer of the PDP | `Refusal` | HTTP |
| --- | --- | --- |
| `false`, with no reason or another reason | `Forbidden` | 403 |
| `false` with `budget_exceeded` | `RateLimited` | 429 with `Retry-After` |
| No answer, an HTTP error, or an answer that is not valid | `Unavailable` | 503 |
| `true` with no `context.usage.units` for a use (6.12) | `Forbidden` | 403 |
| `true` with a ceiling that the action does not list (7.5) | `Forbidden` | 403 |
| No valid identity token | `Unauthenticated` | 401 |
| A mistake of the service, for example an action that the document does not declare | `Internal` | 500 |

## The API

| Item | Content |
| --- | --- |
| `PepConfig`, `PdpConfig` | The typed configuration. `check` returns every error. |
| `Pep` | `authenticate`, `authenticate_token`, `decide`, `decide_approval`, `check`, `display`, `options`, `actions_reply`, `metadata`, `refresh_metadata`, `kept_answers`, `kept_tokens`, `approval_status`, `approval_keepalive`, `approval_stats`. |
| `Operation`, `Permission` | One use, the other permissions (5.5a), and several actions for the use (10.1). |
| `Outcome`, `Allowed`, `Granted` | The result of `decide`: the units, and the ceilings of each permission. `Outcome::ApprovalRequired` is the result of `decide_approval` for a request that waits. |
| `approval` | `ApprovalCall`, `Control`, `Verdict`, `ApprovalRefusal` and `ApprovalStats` for each action (protocol 5.2b, 17). |
| `Ceilings` | `cap`, `cap_or_ceiling`, `check` and `cap_duration`. |
| `ActionDocumentBuilder`, `load_file`, `ServedDocument` | Build, load and serve the action document (section 5). |
| `OptionsQuery`, `OptionsSource` | The options route (5.6 to 5.8). |
| `PdpTransport`, `HttpTransport`, `transport::for_pdp` | The link to the PDP. `for_pdp` gives the transport of the library for the scheme of `pdp.url`. |
| `tls::TlsTransport`, `tls::SYSTEM_CA_BUNDLES` | The link over TLS (feature `tls`). |
| `PdpMetadata` | The endpoints and the five members of this protocol. |
| `refusal::APPROVAL_CANNOT_WAIT` | The stable code of a call that cannot wait for an approval (17.7a). |
| `mcp` | The helpers of the MCP binding. The feature `mcp` is on by default. |
| `Clock`, `SystemClock`, `ManualClock` | The clock. A test moves a `ManualClock` by hand. |

### Methods of the PEP

- `decide` asks the other permissions first, with `context.permission_only` and without `context.usage` (5.5a). Then it asks the use. For an action that lists a budget, the question of the use has `context.usage`. A route that cannot wait calls `decide`. For a reason of 17.1, the refusal maps to HTTP 403 with the code `approval_cannot_wait` (17.7a).
- `check` asks without `context.usage` and takes no unit. Use it for a call that asks the client for input (binding 3.5).
- `decide_approval` follows the steps of 17.7 for each action. It asks as 10.1 says, with no effect. On a reason of 17.1, it calls the function `build` of the caller. This function gives `op`, `view` and the control.
- Then the PEP asks with the effect and no usage, and it submits the request on `approval_required` or `approval_pending`. After a `true` answer, it asks with `context.usage` and the same effect. A call with no control cannot submit, and it gets `ApprovalRefusal::InvalidArgument`.
- `approval_status` reads the verdict with `renew=true` and the subject of the token. `approval_keepalive` renews the lease. Call them only for a call of the agent that names the request. The PEP never renews on a timer.
- `approval_stats` gives the counters of what the PEP received from the approval API. The PEP reads only `verdict`, `request_id` and `code`. It never reads an intent, an explanation or an `approval_ref`.
- `display` asks with `context.match` equal to `subtree` for a list of permissions (4.4). The answers are for the display only. They never let an operation run.
- `options` builds the answer of the options route. It asks one question for each child (5.6).

## Rules that the library follows

| Rule | Paragraph |
| --- | --- |
| The subject has the type `user`. Its id is the `sub` of the token, as opaque text. | 3.3 |
| The question holds no claim other than `sub`. | 9.6 |
| The PEP builds the chain from the data of the service. | 4.5 |
| The PEP reads the token only from the configured header. It never reads `Authorization` or a cookie. | 9.4 |
| The PEP ignores a claim that it does not know. It never uses `owner` in a decision. | 9.4, 9.5 |
| A use of an action that lists a budget sends one question with `context.usage` equal to 1. | 6.4 |
| The PEP never sends the same question without `context.usage` before it. | 10.1 |
| The PEP keeps an answer for `cache_ttl` at most. It never keeps a `true` answer with `units` above 0. | 10.1 |
| The PEP never keeps an answer with a reason of 17.1, and never an answer to a question with `context.approval`. | 10.1, 17.7 |
| The PEP keeps a `budget_exceeded` answer for the lower of `retry_after` and `cache_ttl`. | 10.1 |
| The PEP never sends a question with `context.usage` again after a lost answer. | 10.6 |
| The PEP sends a question again only when the request did not leave the PEP. | 10.6 |
| The PEP waits longer than `procurauth_usage_max_seconds`. | 10.6 |
| The PEP keeps a small fixed number of connections, and it closes an idle one after half of `procurauth_idle_max_seconds`. | 10.7 |
| The PEP splits its questions at the maxima of the PDP. | 10.7a |
| A ceiling that the action does not list makes the answer `false`. | 7.5 |
| The PEP ignores `context.limits` in a `false` answer. | 7.6 |

## Configuration

The application reads the configuration. The library reads no configuration file and no environment variable. `PepConfig::check` follows the rules for the configuration of a program: each error names the key, the expected value and the value found. The check changes no value. Two keys that contradict each other give an error.

| Key | Default |
| --- | --- |
| `pdp.allow_plain_http` | `false` |
| `pdp.tls_ca_file` | none (feature `tls`) |
| `pdp.tls_system_roots` | `true` (feature `tls`) |
| `pdp.request_timeout` | 5 seconds |
| `pdp.usage_wait_margin` | 1 second |
| `pdp.idle_close_after` | 5 seconds |
| `pdp.max_connections` | 4 |
| `pdp.evaluations_max_bytes` | 32768 |
| `pdp.evaluations_max_items` | 50 |
| `pdp.max_response_bytes` | 1048576 |
| `pdp.plain_retries` | 1 |
| `pdp.metadata_max_age` | 3600 seconds |
| `cache_ttl` | 0, no kept answer |
| `cache_max_entries` | 10000 |
| `token_cache_max_entries` | 0, no kept token result |
| `display_max_questions` | 50 |

The credential of the PDP is a `Secret`. Its debug text and its errors never show it.

## The link policy

The PEP must secure the link to the PDP (10.0, step 4). The flag `pdp.allow_plain_http` and the scheme of `pdp.url` give the result below.

| `pdp.url` | `pdp.allow_plain_http` | Result of `check` |
| --- | --- | --- |
| `https` | `false` | Accepted. |
| `https` | `true` | Error for `pdp.allow_plain_http`: the two keys contradict each other. |
| `http` | `false` | Error for `pdp.url`: the link is not secured. |
| `http` | `true` | Accepted. Use it only for a test on the loopback address, or for a link that another layer secures. |

With the feature `tls`, two more rules apply. An `http` URL with `pdp.tls_ca_file`, or with `pdp.tls_system_roots` false, is a contradiction. An `https` URL with `pdp.tls_system_roots` false and no `pdp.tls_ca_file` has no trusted root, and `check` refuses it.

## The transports

`HttpTransport` is a synchronous HTTP/1.1 client over `std::net`, for `http` URLs. It has no TLS. It keeps a small fixed number of connections open.

`TlsTransport` (feature `tls`) is the same client over TLS, for `https` URLs. The two clients share the HTTP/1.1 code: the pool, the request and the reader of the answer.

- It accepts TLS 1.3 and TLS 1.2 only. The crypto provider is aws-lc-rs.
- The server name comes from the host of `pdp.url`. An IP address is also a valid name.
- It trusts the CA bundle of the system when `pdp.tls_system_roots` is true. It reads the first file of `tls::SYSTEM_CA_BUNDLES` that holds a certificate.
- It also trusts each certificate of `pdp.tls_ca_file`, a PEM file. Each section of this file must be a valid CA certificate.
- It reads no environment variable. `SSL_CERT_FILE` and `SSL_CERT_DIR` have no effect.
- `TlsTransport::new` reads the files one time. An error names the key `pdp.tls_ca_file` or `pdp.tls_system_roots`. A transport with no trusted root is an error.
- `trusted_roots` and `system_bundle` tell what the transport read. Write them to the log at start.

| Path in `tls::SYSTEM_CA_BUNDLES` | System |
| --- | --- |
| `/etc/ssl/certs/ca-certificates.crt` | Debian, Ubuntu |
| `/etc/pki/tls/certs/ca-bundle.crt` | Fedora, RHEL |
| `/etc/pki/ca-trust/extracted/pem/tls-ca-bundle.pem` | Fedora, RHEL 7 and later |
| `/etc/ssl/cert.pem` | Alpine |
| `/etc/ssl/ca-bundle.pem` | openSUSE |

The trait `PdpTransport` tells the PEP how an exchange failed. A request that did not leave the PEP (`NotSent`) is the one case where the PEP can send a question with `context.usage` again (10.6). `TlsTransport` makes the handshake before it writes a byte of the request. Thus a failed handshake is `NotSent`. A failure after the handshake is `Lost` or `Timeout`.

`transport::for_pdp(&config.pdp)` gives `HttpTransport` for an `http` URL and `TlsTransport` for an `https` URL. Without the feature `tls`, an `https` URL is an error for `pdp.url`.

## Kept token results (9.7)

A service MAY keep the result of a token check for that exact token (9.7). Set `token_cache_max_entries` above 0 to use this table. The default 0 keeps no result, and the PEP checks each token again.

- The key is the SHA-256 hash of the exact text of the token. The table holds no token text.
- The value is the identity of the check: `sub`, `owner`, `iat` and `exp`.
- A kept result serves while the clock, in whole seconds, is not later than `exp` plus `clock_skew_seconds`. This is the rule of the check of `exp`. The PEP reads the time from its `Clock`.
- The PEP keeps only the result of a token that passes every check. It never keeps a refusal.
- When the table is full, the PEP deletes the results that do not serve any more. When it is still full, it keeps no new result. This is the rule of 10.1b.
- `Pep::kept_tokens` gives the number of kept results.

## Use from a plain HTTP service

A service with no MCP endpoint uses the same `Pep`. The test file `tests/plain_http.rs` shows each item below. It also runs without the feature `mcp`.

| Need | API |
| --- | --- |
| Build the transport for `pdp.url`. | `transport::for_pdp`, `HttpTransport`, `tls::TlsTransport` |
| Check the identity token. The PEP reads only the header that `token_header` names. | `Pep::authenticate(headers)`, `Pep::token_header` |
| Ask an operation with other permissions. The PEP asks them first, in one request, with `context.permission_only` and without `context.usage` (5.5a). | `Operation::new(..).also(..)` or `.also_all(list)`, `Pep::decide`, `Allowed::others` |
| Ask for a later request of a subject whose token the service checked and kept. | `Identity::from_checked_token` |
| Take one unit for a use of an action that lists a budget. The question of the use has `context.usage` (6.4). | `Pep::decide`, `Allowed::units` |
| Refuse a call that cannot wait for an approval: HTTP 403 with the body `{"error":"approval_cannot_wait"}` (17.7a). | `Pep::decide(..).into_result()`, then `Refusal::to_http` |
| Keep answers for a short time (10.1). | `cache_ttl`, `cache_max_entries`, `Pep::kept_answers` |
| Read the limits that the PDP publishes. | `Pep::metadata`, `PdpMetadata` |
| Keep the results of the token checks (9.7). | `token_cache_max_entries`, `Pep::kept_tokens` |
| Serve a route that runs no action of the action document, for example a static file. The PEP asks `access` on the service (10.1c). A health-probe route asks nothing and needs no token. | `Pep::decide_route_without_action` |

For each request, the service does these steps.

1. Call `pep.authenticate(headers)`. On a refusal, send `refusal.to_http()`: HTTP 401.
2. Build the `Operation` from the data of the service. Add each other permission with `also`.
3. Call `pep.decide(&who, &operation).into_result()`.
4. On `Err(refusal)`, send `refusal.to_http()`. On `Ok(allowed)`, apply the ceilings and run the operation.

```rust
let reply = match pep.decide(&who, &operation).into_result() {
    Ok(allowed) => run(&allowed),
    // 401, 403, 429 or 503 (10.3). A reason of 17.1 gives 403 and `approval_cannot_wait` (17.7a).
    Err(refusal) => refusal.to_http(),
};
```

`decide` never submits an approval request. A plain route cannot wait, so a reason of 17.1 gives the refusal of 17.7a. The constant `refusal::APPROVAL_CANNOT_WAIT` holds the code. A route that can wait calls `decide_approval`.

The kept answers follow 10.1. The PEP never keeps an answer with a reason of 17.1. It never keeps an answer to a question with `context.approval`: only `decide_approval` sends this member, and it asks the PDP without the table. It never keeps a `true` answer with `units` above 0.

`Pep::metadata` gives `PdpMetadata` with the members of this protocol. Each member is optional, and an absent member is `None`.

| Member | Field | Use |
| --- | --- | --- |
| `procurauth_usage_max_seconds` (6.9) | `usage_max_seconds` | The wait for a question with `context.usage` is this time plus `pdp.usage_wait_margin`, and at least `pdp.request_timeout` (10.6). |
| `procurauth_idle_max_seconds` (10.7) | `idle_max_seconds` | The transport closes an idle connection after half of this time, or after `pdp.idle_close_after`. The lower value applies. |
| `procurauth_evaluations_max_bytes` (10.7a) | `evaluations_max_bytes` | The largest Access Evaluations request. The lower of this value and `pdp.evaluations_max_bytes` applies. |
| `procurauth_evaluations_max_items` (10.7a) | `evaluations_max_items` | The most items of an Access Evaluations request. The lower of this value and `pdp.evaluations_max_items` applies. |
| `procurauth_directory_max_age_seconds` (3.9) | `directory_max_age_seconds()` | The age bound of the copy of the directory that the PDP uses. It is for the logs. The PEP takes no decision on it. |

A member with a value that is not a whole number of 1 or more makes the metadata not valid. The PEP then fails closed with HTTP 503.

## The MCP helpers

The module `mcp` follows the MCP binding. It is a thin layer over `Pep`.

- `ToolMap` holds the tools and their actions (3.2).
- `McpGate::list_tools` asks one display question for each action of the tools (4.1).
- `McpGate::session` asks the action `mcp.session` for a method that has no tool (3.6a).
- `McpGate::call` asks for one call, at the stage `Ask` or `Run` (3.4, 3.5, 3.5a, 3.7).
- `refusal_reply` maps a refusal to the HTTP status and the JSON-RPC error of section 5. It never copies a value of the request.

## Not in the library

- A TLS client without the feature `tls`.
- A server. The application owns its transport and its routes.
- The Search API of AuthZEN.
