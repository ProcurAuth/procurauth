# Technical decisions of procurauth-pep and reference-service

Each item gives one choice and the reason. The numbers are for this file only.

## Library

1. Replaced by 44.
2. Replaced by 52.
3. `HttpTransport` is a synchronous client on `std::net`, with a small pool. It needs no runtime, no `unsafe` code and no dependency. The PDP link has few connections, so a thread for each caller is enough.
4. The transport tells the PEP how an exchange failed: `NotSent`, `Timeout`, `Lost`, `Malformed`, `TooLarge` or `Unsupported`. Protocol 10.6 lets the PEP send a question with `context.usage` again only when the request did not leave the PEP. Thus the PEP must know the kind.
5. A failure of the first write is `NotSent`, and a failure after the first byte is `Lost`. The client checks with a non-blocking `peek` that the PDP did not close an idle connection, before it reuses the connection. This keeps the case of a stale connection very rare.
6. The pool closes an idle connection at the next use of the pool, not in a thread of its own. The rule of 10.7 is that the PEP never uses a connection that the PDP can have closed. A thread that only closes sockets adds a lock and a clock for no gain.
7. The PEP asks a question without `context.usage` again after `NotSent` or `Lost`, `plain_retries` times. It never asks again after a timeout, because a timeout already took the whole wait. A question with `context.usage` goes again only after `NotSent`, one time.
8. The wait for a question with `context.usage` is the time that the PDP publishes plus a margin, and at least the normal timeout. Protocol 10.6 says that the PEP SHOULD wait longer than this published time.
9. The PEP reads the metadata at the first question, and again after `metadata_max_age`. It keeps the old metadata when a new read fails. A status 404, 405, 410 or 501 means no metadata, and the PEP uses the default paths of AuthZEN (10.0, step 4). After a failed new read, the PEP keeps the old metadata and tries again 5 seconds later.
10. The PEP refuses an endpoint of the metadata that has another origin than the PDP. A bad metadata document could send the credential of the service to another host.
11. The metadata must have `policy_decision_point` equal to the URL of the PDP, as AuthZEN 1.0 requires. A member of this protocol that is not a whole number of 1 or more makes the metadata not valid. The PEP then fails closed.
12. The PEP sends questions without `context.usage` in one Access Evaluations request when there are two or more. A request holds the subject and `context.match` one time, and each item holds the action and the resource. The binding shows this form in its example.
13. A PDP without an Access Evaluations endpoint gets one request for each question. AuthZEN 1.0 says that such a PDP omits the endpoint from its metadata.
14. The PEP derives "the action lists a budget" from the action document, and `Operation` has no flag for it. The document is the declaration of the service (6.2, 7.5), so a flag could only contradict it.
15. An operation has one use and other permissions (5.5a). The other permissions come first, without `context.usage` (10.1a). An other permission that equals the use is not asked twice, because 10.1 forbids the same question without usage before the question with usage.
16. For several actions for the use, the PEP asks them one at a time with `context.usage`, in the order of preference. This is the last item of 10.1. It stops at the first `true` or at the first `budget_exceeded`. A `false` answer without `budget_exceeded` counted nothing, and the PEP keeps it as an answer without usage.
17. The PEP keeps answers in two sets: with usage and without usage. A kept `true` answer without usage never starts a use of an action that lists a budget. The PEP keeps a `true` answer with usage only when `units` is 0.
18. A kept `budget_exceeded` answer gives the time that remains to the end of the window. It does not give the time that remains to the end of the kept life. The value never grows (case U58). See question 1 of `spec-questions-pep.md`.
19. A ceiling that the action does not list, or that the document does not declare, makes the answer `false` (7.5). A name that the document declares as a budget gives HTTP 503, because a budget value never appears in an answer (7.3).
20. The PEP accepts a ceiling value outside the range `min` to `max` of its declaration. A lower value is stricter, and a value above `max` can be applied as it is.
21. A mistake of the service, for example an action that the document does not declare, gives `Refusal::Internal` with HTTP 500. The PEP sends no question. Protocol 10.3 has no status for it, and a client must not see it as a decision of the PDP.
22. A display question has no rule that the action lists the type of the node. The question `subtree` also looks below the node (4.4), and the list of the tools asks each action on the service (binding 4.1).
23. The PEP refuses a request that has the header of the token more than one time. The edge replaces this header (9.1), so a duplicate shows a fault of the proxy chain.
24. Replaced by 56.
25. The `Pep` takes a `Clock`. The library reads the system clock only in `SystemClock`. A test moves a `ManualClock`, and the tests of windows and kept answers need no sleep.
26. The error of the configuration is a list of `ConfigError` with the key, the expected value and the value found. The credential is a `Secret`. A message never holds it, also when it is not valid.
27. The feature `mcp` is on by default, so that `scripts/ci.sh` builds and tests it. `scripts/ci.sh` also checks the build with no default features.
28. The MCP module has no state and no transport. It parses a request, maps a tool to an action, asks the PEP and maps a refusal. The service does the work of the tools.
29. The MCP module reads a request with the strict JSON reader of the core. A duplicate member is not a request.
30. A tool that the map does not have gets the refusal of a `false` answer, and the PEP sends no question for it (binding 4.2). Arguments that do not name a node give the code `-32602` with the message `Invalid params`. This reply does not depend on the rights of the subject.
31. The reply of the input request follows the draft of the MCP revision 2026-07-28: `resultType` `input_required`, `inputRequests` and `inputResponses`. The binding does not define the form. The helpers are small, and the service shows the use.

## Reference service

32. The service has its own HTTP/1.1 layer on `std::net`, as `fake-pdp` has. The layer is about 400 lines with tests. It refuses a folded header, a chunked body and a bad length. Its `Reject` replies close the connection. A framework would hide these rules.
33. The configuration file is JSON, as the other tools of the repository use. The reader is the strict reader of the core. A duplicate member is an error.
34. The table of keys is in the code. The README holds the same table, and a test compares them. The forms `_env` and `_file` exist for the keys that name a place: `listen`, `issuer`, `jwks`, `pdp.url` and `pdp.credential`.
35. A secret takes only `_env` and `_file`. A secret in clear text in the file stops the program, and the error does not show it.
36. The effective configuration shows `default`, `file`, `env:VAR` or `file:/path` as the source. The program also writes it to the standard error at start.
37. Replaced by 60.
38. The default of `cache_ttl_seconds` is 0. The conformance cases use the same subject and the same resource in many cases. A kept answer from one case would change the next case.
39. The service asks the PDP before it looks for a node. A client that has no right gets 403 and learns nothing about the existence of the node.
40. The service builds the chain from the position of each id in the path. The type of the node comes from the route, never from the client (4.5).
41. A read lowers the value of `timeout_seconds` to the ceiling `max_duration`, and a write refuses a value above it. The two routes show both ways to apply a ceiling.
42. The service answers 401 with an empty body on each route. The MCP binding asks for this on the MCP endpoint (5.6a), and the other routes follow the same rule.
43. The log has the method, the path and the status for each request, and the cause of a refusal. It never has a header, a query, a body or a token.

## TLS, link policy and kept token results

44. The library has the feature `tls`, off by default. It gives `TlsTransport`, on rustls with the crypto provider aws-lc-rs. A plain HTTP service, for example the Rust version of Stowr, then needs no TLS code of its own. The default build stays without a C compiler (decision 9 of `decisions.md`) and compiles no TLS crate.
45. The provider is aws-lc-rs, at the versions of the portal: rustls 0.23.45, aws-lc-rs 1.18.1, aws-lc-sys 0.45.0 and rustls-pki-types 1.15.1. The two programs then share one crypto library. The lock file holds these versions.
46. `TlsTransport` uses the HTTP/1.1 code of `HttpTransport` over a generic stream. One pool and one reader of answers keep one set of rules: the bounds, the chunked body and the stale connection.
47. The transport reads the CA bundle of the system from a fixed list of five paths. It does not use `rustls-native-certs`, `openssl-probe` or `rustls-platform-verifier`, because they read `SSL_CERT_FILE` and `SSL_CERT_DIR`. The library reads no environment variable.
48. `pdp.tls_system_roots` true with no bundle in the list is an error, also when `pdp.tls_ca_file` gives roots. The value in the configuration is the value that applies. Set `false` for an image with no system bundle.
49. Each PEM section of `pdp.tls_ca_file` must be a valid CA certificate. A bad section or a key in this file shows a mistake of the operator. In the system bundle, the transport ignores a certificate that rustls refuses, because a large bundle can hold old ones.
50. The transport makes the whole handshake when it opens a connection, before the first byte of the request. Thus a failed handshake is `NotSent`, as decisions 4 and 5 say. After the handshake, a byte that the TLS layer took counts as sent, so a later failure is `Lost` or `Timeout`.
51. Before the transport uses an idle TLS connection again, it reads the records that wait, with no block. A `close_notify`, the end of the TCP stream or data outside an answer make the connection not usable. A session ticket of TLS 1.3 does not.
52. The flag is `allow_plain_http`, with no alias and no old name. An `http` URL needs it. An `https` URL with it is a contradiction, and the check refuses it. The flag names the scheme, not the security: another layer can secure a plain link.
53. With the feature `tls`, an `http` URL with `pdp.tls_ca_file`, or with `pdp.tls_system_roots` false, is a contradiction. These keys have no effect on a plain link, so a value shows a mistake.
54. `transport::for_pdp` gives the transport of the library for the scheme of `pdp.url`. A plain service then builds its link in one call. Without the feature `tls`, an `https` URL is an error for `pdp.url`.
55. The TLS tests use small fixed PEM files in `tests/data`, valid until 2126. A dev-dependency cannot depend on a feature, so `rcgen` would compile a crypto crate in each test build, also without `tls`.
56. The library can keep the result of a token check (9.7) when `token_cache_max_entries` is above 0. A service then makes one signature check for many requests with the same token. The default is 0, so a PEP keeps no result unless the operator asks for it.
57. The key of a kept token result is the SHA-256 hash of the exact text of the token. The table holds no bearer token, so a copy of the memory shows none. Another text is another key (9.7, "that exact token").
58. A kept token result serves while the clock in whole seconds is not later than `exp` plus the skew. This is the check of `exp` in `procurauth-core`, with the clock of the `Pep`. The other rules of time cannot change from true to false, so the PEP does not check them again.
59. A full token table keeps no new result, after it deletes the results that do not serve any more. This is the rule of 10.1b for the kept answers. A full table costs only one signature check for a new token, and the table needs no order of age.
60. The reference service and the reference edge build `procurauth-pep` without the feature `tls`. They are the test subjects of the series U and E on the loopback address, and they need no C compiler. Thus `pdp.url` takes an `http` URL, and `pdp.allow_plain_http` must be true.
61. The PEP reads the optional metadata member `procurauth_directory_max_age_seconds` (3.9), and it takes no decision on it. The age of the copy of the directory is a duty of the PDP. A value that is not a whole number of 1 or more makes the metadata not valid (decision 11).
62. A plain HTTP service gets the refusal of 17.7a with `Outcome::into_result` and `Refusal::to_http`. Thus the library adds no other helper for it. The constant `refusal::APPROVAL_CANNOT_WAIT` holds the code for a service that writes its own body.

## Rulings of the lot AL1

63. `Pep::decide_route_without_action` asks `access` on the service for a route that runs no action of the document (10.1c). It does not check the action against the document, because `access` is never in it. A reason of 17.1 in the answer gives HTTP 503, because `confirm` has no effect on `access` (17.8). The answer is kept as 10.1 says, with the key of the action `access`.
64. After a 422 `invalid_evaluation`, the approval flow asks the question with the effect one more time (binding 5.7a). It then submits again with the new `evaluation_id`. A second `invalid_evaluation` gives `ApprovalRefusal::Expired`. A 422 `explanation_required` and a 409 `cannot_supersede` give `ApprovalRefusal::InvalidArgument`. A 401 and a 403 of the approval API give HTTP 503.
65. A reason of 17.1 for an other permission (5.5a) stays an answer that is not valid, and the PEP gives HTTP 503. The case U126 tests it for a route and for a tool.
66. `Operation::also_all` adds a list of other permissions in one call. `Pep::decide` already asks them in one Access Evaluations request, with `context.permission_only`, and it keeps their answers apart. The function only makes the list easy to give.
67. `Identity::from_checked_token` builds an identity from the values of a token that the service checked before. A service that keeps a bounded map from its sessions to these values can then ask the PDP for a later request. The function checks nothing, so its documentation forbids a value that a client sends.
