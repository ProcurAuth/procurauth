# Open points about procurauth-core

This file lists what the tools `access-token`, `fake-pdp` and the library `procurauth-pep` need from `procurauth-core` and from the schemas. Each item says if it is done or open. The numbers are for this file only.

## Done

1. `PublicKey::from_spki_pem` reads a public key from a PEM text. The configuration of `access-token` needs it for the member `pem_keys`. The error never holds the text.
2. `KeySet::into_keys` gives the keys of a set. `access-token` merges a JWK Set and the PEM keys into one set.
3. `identity::verify_for_pep` and `Claims::from_value_for_pep` read a token as a PEP does. An `owner` that is not a text, or that is empty, is read as no owner. The case U10 needs this: it sends a token with `"owner": 42` and expects the answer of the PDP. Protocol 9.5 says that a PEP never uses `owner`. `verify` and `Claims::from_value` keep their rules, so `access-token` does not change.

## Open

4. **A bad `owner` in `verify`.** `verify` could also ignore a bad `owner`. Then `verify_for_pep` is the same function, and `access-token check` reports a bad `owner` as a warning. The library works in both cases.
5. **An empty third part of a token.** `inspect` reports a token with an empty third part as a failure of `structure`. A token with `alg: none` has such a part (RFC 7519, section 6). The check that the token breaks is `alg` (9.2). `inspect` could read an empty third part as an empty signature. Then `alg` fails for `none`, and `signature` fails for another algorithm.

   `access-token` changes the empty part before it calls `inspect`. The test `p_9_2_the_algorithm_none_is_refused` of the core has the old result. The PEP refuses the token in each case.
6. **Schema `fake-pdp-data`: three members.** `fake-pdp` takes the admin switch and the credential file from the command line, because the schema is closed. The data file could have the root members `admin` (boolean) and `credential_file` (string). The `failure` definition and the reader in `directory.rs` could also have the failure kinds `close`, `invalid_json` and `no_units`.
7. **Schema `fake-pdp-data`: the description of `metadata`.** It says that a member that the file does not give is not in the metadata. `fake-pdp` publishes the four members always, with defaults. Change the description, or the tool.
8. **Counter presets for several services.** A counter names no service. `fake-pdp` applies it to each service that declares the budget. The `counter` definition could have an optional member `service`, for a file with several services that use the same budget name.
9. **A reader of the case model.** The crate `procurauth-schema-check` has no typed API for the conformance cases and their matchers. The test `conformance_d.rs` of `fake-pdp` has a local runner, and `access-conformance` has its own reader. A shared reader is possible.
10. **Order of the members of a JSON object.** `serde_json` sorts the members of an object. The log of `fake-pdp` shows the limits in sorted order, and the page shows the order of the action document. The feature `preserve_order` of `serde_json` in the workspace gives the order of the document.
