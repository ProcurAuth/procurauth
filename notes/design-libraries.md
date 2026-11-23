# Design of two libraries: a PDP library and a C ABI of the PEP

The owner asked for two libraries. This lot does not build them. This file gives the design only, and it changes no code.

- Part 1: a PDP library from `procurauth-core`, for the portal. The portal is a Rust homelab server. It is its own AuthZEN PDP, its own edge and an approval inbox.
- Part 2: a C ABI of `procurauth-pep`, for ClickHouse Dash. Dash is a C++ program with a static binary.

Each item gives one choice and the reason. The numbers are for this file only. The paths of this repository are relative. The paths of the portal and of Dash are absolute. The last section lists the points that the owner must decide.

## Part 1. A PDP library for the portal

### 1.1 What the core gives to a PDP now

The functions below are pure: no I/O, no clock, no environment. A PDP can use them now.

1. `coverage` (3.5, 4.3, 4.4, 7.3, 17.8): `covers`, `granted`, `subtree_applies`, `key_granted`, `key_subtree_applies`, `confirm_has_effect`, `confirm_required`, `subject_confirm_required`, `key_confirm_required`, `ceilings_of_side` and `ceilings_of_key`.
2. `limits` (6, 8, 11.1): `Window`, `BudgetItem`, `LimitValue`, `read_limit_values`, `window_start`, `seconds_to_window_end`, `retry_after`, `lowest` and `combine_sides`.
3. `grants` (3.5, 6): `Grant` and `read_grants`. The reader checks each grant against the tree of types and the limit declarations of the document.
4. `authzen` (4.1, 6.4, 11, 17.1): `EvaluationRequest::from_value` and `EvaluationsRequest::from_value`. `RequestContext::from_context` reads `usage`, `match`, `approval.effect` and `permission_only` with the rules of HTTP 400. Also `check_resource`, `Reason`, `Decision::{allow, deny, budget_exceeded}` and `evaluations_answer_to_value`.
5. `effect` (17.3, 17.9): `effect_hash`, `canonical_text`, `check_op`, `check_view`, `resource_text` and `is_effect`.
6. `subject` (3.1, 3.2, 3.7): `classify`, `is_local_part` and `key_id`.
7. `path`, `names` and `action_document`: `NodePath`, `encode_id`, `decode_id`, `MAX_LIMIT_NUMBER` (2^53-1) and `MAX_EVALUATION_ID_BYTES` (1024). Also `ActionDocument::{parse_str, check_path, action, limit, action_lists_budget}`.
8. `json`: the strict reader `parse_slice`. It refuses a duplicate member and a depth above 64.
9. The modules `directory` and `approvals` hold the data of the test PDP. The modules `jws`, `identity` and `key_format` serve an edge and a PEP. A PDP library does not need them.

The steps of a decision exist two times in this repository: `Pdp::decide` in `crates/fake-pdp/src/pdp.rs` and the reference PDP in `crates/procurauth-core/tests/reference_pdp.rs`. The counters are in `crates/fake-pdp/src/counters.rs`. The approval state machine is only in `crates/fake-pdp/src/approvals.rs`: `gate`, `consume`, `issue_evaluation`, `submit`, `verdict`, `answer`, `reply`, `cancel` and the private `settle`.

### 1.2 What the portal duplicates today

Each item names a file of the portal that does a part of the work of ProcurAuth again, and the differences that the code shows.

10. `/home/eliot/Documents/Work/portal/crates/inbox-core/src/effect.rs` copies `effect`. The JCS writer is the same: the UTF-16 order of names, the short escapes, `\u00xx` in lower case. The number rule is the same: integers from -(2^53-1) to 2^53-1. The portal also embeds a copy of the vectors in `/home/eliot/Documents/Work/portal/crates/inbox-core/tests/fixtures/effect-vectors.json`.
11. The effect code has four differences. The portal refuses an `op` deeper than 64 levels in the writer. `check_op` of the core has no depth bound for a `Value` from `serde_json` (up to 128 levels). The portal error is a code with a fixed text, and the core error gives the JSON Pointer. The portal bounds a `view` (12 fields, 128 and 48 characters, pointers of 256 bytes), and the core does not.
12. `/home/eliot/Documents/Work/portal/src/decision.rs` copies `coverage` and `Pdp::decide`. The prefix rule, the lowest ceiling, the two sides of a key and the rule of `subtree` are the same. The portal adds an owner whose side is always `true`, with no limit: the superadmin (portal 9.1a).
13. The portal checks each grant against the last known document at each question. With no document, no grant below the service applies. The core checks the grants one time, when the data loads.
14. For a budget, the portal reads every grant `access` on the service, and takes the lower amount for each window. `fake-pdp` reads the first grant `access` on the service only. The protocol does not say which rule applies to two such grants.
15. `/home/eliot/Documents/Work/portal/src/grants.rs` and `/home/eliot/Documents/Work/portal/src/limits.rs` copy `path`, `names` and `limits`. The percent-encoding and the largest number (`MAX_NUMBER`, 2^53-1) are the same. The portal refuses a grant with more than 8 limit values (a MAY of the protocol). The core has no bound.
16. `/home/eliot/Documents/Work/portal/src/counting.rs` and `Store::count_use` in `/home/eliot/Documents/Work/portal/src/store.rs` copy `counters`. The portal counts in SQLite or PostgreSQL in one statement, by the clock of the database (6.3, 6.5, 6.8). It computes `retry_after` from seconds as `f64`. The core uses whole milliseconds. The portal adds a memo of refusals (10 000 entries) and three causes of HTTP 503.
17. `/home/eliot/Documents/Work/portal/src/http/pdp/approval.rs` copies the gate of 17.1. The portal counts first, then consumes the approval with one compare-and-set. It returns the units when the compare-and-set fails (decision 171). `fake-pdp` does both steps under one lock.
18. The `evaluation_id` has two forms. The portal form has no state: base64url of a key id, the time, an HMAC-SHA256 tag and the sealed facts. The portal checks the bound of 1024 bytes when it makes an id. `fake-pdp` gives `ev_` and 24 hexadecimal digits, and keeps it in memory for 300 seconds, at most 100 000 ids.
19. `/home/eliot/Documents/Work/portal/src/http/pdp.rs` reads the body with `serde_json`, so a duplicate member keeps the last value. The core reader refuses it. The portal refuses a `usage` object with a member other than `units`. The core ignores such a member.
20. `/home/eliot/Documents/Work/portal/src/actions.rs` reads the action document with `serde_json`, with its own bounds (16 types, 1024 actions, 256 KiB). `/home/eliot/Documents/Work/portal/src/subject_id.rs` copies `subject`. An empty id is a person in the portal and `Unknown` in the core.
21. The requests and their statuses are in `/home/eliot/Documents/Work/portal/src/inbox/items_ops.rs` (`consume`, `Consumed`), with SQL in the same directory. The API of 17.4 is in `/home/eliot/Documents/Work/portal/src/http/approval_api.rs` and `/home/eliot/Documents/Work/portal/src/inbox/approvals.rs`.
22. `/home/eliot/Documents/Work/portal/src/identity.rs` signs the identity token with `aws-lc-rs`. This is the work of the edge, not of the PDP. The library does not include it.

### 1.3 What a PDP library still needs

23. A store of counters, as a trait. One call takes one unit from each counter of a use, or none, in one atomic step (6.5). The step is the same for all instances of the PDP (6.8). A failure gives HTTP 503 and never a decision (6.9).
24. The store also returns the units of a use. A PDP that counts first needs this when the compare-and-set of the approval fails (decision 171).
25. The store owns the clock of the windows. Paragraph 6.3 asks for one clock for all instances, and the portal uses the clock of the database. The store returns the milliseconds to the end of each window that refuses, and the library computes `retry_after`.
26. A pure state machine of the requests (17.5, 17.6). It has the seven statuses, the lease, the grace, the revivals, the longest life, `consume_before` and the deny time. It also gives the verdict and the gate of 17.1. Each function takes the time as an argument.
27. A store of approvals, as a trait. The consume of an approval is one compare-and-set, so an approval runs one time (17.2).
28. A trait for the `evaluation_id`, with two functions: make an id and open an id. The library refuses an id above 1024 bytes and answers `false` with no reason (decision 188). The memory form of `fake-pdp` and the sealed form of the portal both fit.
29. The checks of a submission (17.4, 17.9), as one pure function, in the order of decision 115.
30. A writer of the metadata members of 6.9, 10.7, 10.7a and 17.4a. The values that a PDP publishes are parameters, not constants. The portal reads them from its configuration, and `fake-pdp` has them as constants today.
31. The bounds of a PDP are parameters too. Examples are 8 limit values on a grant, 100 000 kept ids and 20 open requests for each subject.

### 1.4 The proposed crate

32. The name is `procurauth-pdp`, with the library `procurauth_pdp`. It is in this workspace and depends on `procurauth-core`, `serde_json` and `sha2` only.
33. The library has three layers. The decision layer is pure. The store layer is a set of synchronous traits. The memory layer implements the traits for one instance, from the code of `fake-pdp`.
34. An asynchronous PDP, such as the portal, uses the pure layer and does its own I/O. The pure layer tells which counters and which approval a question needs, so the caller can read them in any way.
35. `fake-pdp` becomes a thin HTTP server on `procurauth-pdp`. The reference PDP of the core tests stays separate, because it checks the cases with a second implementation.

Modules:

- `decide`: the subject, its sides, the grants, the ceilings, the counters of a use and the need of an approval.
- `count`: the trait `CounterStore`, the result of a count and `retry_after`.
- `approval`: the trait `ApprovalStore`, the statuses, the transitions, the verdict, the gate of 17.1 and the checks of a submission.
- `evaluation`: the trait `EvaluationIds` and the bound of 1024 bytes.
- `metadata`: the published values and their JSON members.
- `memory`: `MemoryCounters`, `MemoryApprovals` and `MemoryEvaluationIds`, for one instance only (6.8).

```rust
pub struct Side<'a> { pub subject: &'a str, pub grants: &'a [Grant] }
pub enum Owner<'a> { NoOwner, Person(Side<'a>), Unlimited } // Unlimited: the superadmin of the portal
pub struct Question<'a> {
    pub service: &'a str, pub action: &'a str, pub node: &'a NodePath,
    pub context: &'a RequestContext, pub document: Option<&'a ActionDocument>,
}
pub enum Step {
    Answer(Decision),                                 // final, no store
    Count(UsePlan),                                   // take the units, then answer
    Approval { key: ApprovalKey, then: UsePlan },     // read the gate first (17.1)
}
pub fn decide(subject: Side<'_>, owner: Owner<'_>, q: &Question<'_>, bounds: &Bounds) -> Step;

pub trait CounterStore {
    fn take_all(&self, uses: &[CounterUse]) -> Result<Taken, StoreError>;  // 6.5, 6.8
    fn return_all(&self, uses: &[CounterUse]) -> Result<(), StoreError>;   // decision 171
}
pub enum Taken { All, Refused { millis_left: Vec<(usize, u64)> } }
pub fn answer_count(plan: &UsePlan, taken: Result<Taken, StoreError>) -> Result<Decision, Unavailable>;

pub trait ApprovalStore {
    fn latest(&self, key: &ApprovalKey) -> Result<Option<Request>, StoreError>;       // 17.6
    fn consume(&self, key: &ApprovalKey, id: &str) -> Result<bool, StoreError>;       // compare-and-set (17.2)
}
pub fn gate(latest: Option<&Request>, now_s: u64, with_usage: bool) -> Gate;          // 17.1
pub fn settle(request: &mut Request, now_s: u64, published: &Published);              // 17.5
pub fn verdict(request: &Request, now_s: u64) -> Verdict;                             // 17.6
pub fn check_submission(s: &Submission, published: &Published) -> Result<(), ApiError>; // 17.4, 17.9

pub trait EvaluationIds {
    fn make(&self, bound: &Bound<'_>, now_ms: u64) -> Option<String>; // None: `false` with no reason
    fn open(&self, id: &str, service: &str, effect: &str, now_ms: u64) -> Result<Facts, InvalidId>;
}
pub fn metadata(published: &Published) -> serde_json::Value; // 6.9, 10.7, 10.7a, 17.4a
```

### 1.5 Steps for the portal

Do one step at a time. After each step, run the tests of the portal and `access-conformance --role pdp` (series D). The steps have letters, so that they do not mix with the numbers of the items.

- A. Add `procurauth-core` and `procurauth-pdp` as git dependencies with a fixed `rev`.
- B. Raise the `rust-version` of the portal from 1.88 to 1.89. The ProcurAuth crates need 1.89 and the edition 2024.
- C. Replace the body of `/home/eliot/Documents/Work/portal/crates/inbox-core/src/effect.rs` with calls to `procurauth_core::effect`. Keep the bounds of the view of the portal in a wrapper. Keep the codes `invalid_op` and `invalid_view`.
- D. Build the Android crate `inbox-android` again. The core needs no C compiler, but it adds `ed25519-dalek` and `p256` to the phone.
- E. Convert the grants of the portal to `procurauth_core::grants::Grant` when the directory loads. Replace `decision::person` and `decision::key` with `procurauth_pdp::decide`.
- F. Implement `CounterStore` for `Store` (SQLite and PostgreSQL) with the current statements. Keep the memo of refusals outside the trait.
- G. Use the pure functions `gate`, `settle` and `verdict` over the asynchronous inbox store. Keep the compare-and-set of `items_ops::consume`.
- H. Implement `EvaluationIds` with the sealed form of the portal.
- I. Read the AuthZEN body with the strict reader of the core. A duplicate member then gets HTTP 400. Write this change in the upgrade notes of the portal.
- J. Remove the code that items 10 to 20 list, when the library does the same work.

### 1.6 Risks of Part 1

36. Asynchronous and synchronous code: the portal runs on `tokio`. Its counters use clients that block (`rusqlite`, `postgres`), and its inbox uses `tokio-postgres`. A synchronous trait does not fit the inbox, so the pure layer must be complete without the traits.
37. Storage: the library has no SQL. The tests of the traits use the memory layer and the cases D. The portal keeps its own tests of SQLite and PostgreSQL, because only the portal knows its schema.
38. No `unsafe`: the library keeps `unsafe_code = "forbid"`, as the whole workspace does.
39. Versions: the ProcurAuth crates have `publish = false` and the version 1.0.0. The portal must pin a commit, and a new lot can change the API. The portal pins its other crates with `=`. The caret ranges of ProcurAuth accept these pins.
40. Size: the core holds the code of tokens and keys, which a PDP does not need. A feature of the core can remove these modules. The owner decides whether the gain is worth a feature.
41. Behaviour: items 11, 14, 19 and 20 are real differences. Each one needs a choice before the portal moves, or a spec question when the protocol is silent.
42. Licence: ProcurAuth is Apache-2.0. The root package of the portal says `MIT OR Apache-2.0`, and the shared crates say `LicenseRef-Proprietary`. The README of the portal says that the owner has not decided. The portal must depend on the crates, and it must not copy their code. A binary of the portal then carries the Apache-2.0 text.

## Part 2. A C ABI of procurauth-pep for ClickHouse Dash

### 2.1 What Dash does today

43. Dash is C++17 with CMake and Ninja. The Linux release uses `zig cc -target x86_64-linux-musl -static` (and `aarch64`) with `-static-pie`. The release also builds for macOS on `x86_64` and `arm64` (`/home/eliot/Documents/Work/Clickhouse-dash/.github/workflows/release.yaml`).
44. The release links a static OpenSSL (`CHDASH_TLS=ON`, `/home/eliot/Documents/Work/Clickhouse-dash/tools/build_openssl.sh`). The same OpenSSL serves ClickHouse (`secure=1`) and the PDP (`https`).
45. Dash has its own PEP in C++, in `/home/eliot/Documents/Work/Clickhouse-dash/src/`. `access_identity.cpp` and `signature_verify.cpp` check the token with Monocypher (Ed25519) and micro-ecc (P-256). `access_pdp.cpp` (1608 lines) reads the metadata, sends the questions, keeps answers and handles the approval API.
46. `pdp_transport.cpp` (874 lines) is its own HTTP/1.1 client with OpenSSL. It has a pool, one deadline for each request and `post_once` for a question with `context.usage`. `approval_effect.cpp` has its own JCS. `mcp_approval.cpp` runs the steps of 17.7 for each tool.
47. A thread pool of `cpp-httplib` serves the requests (`server.worker_threads`, `server.max_worker_threads`). A call to the PDP blocks its thread today.
48. Dash has functions that `procurauth-pep` does not have. They are Resource Search (`access.pdp.search`), `batch_timeout`, a cache bound in bytes and PEM keys. Dash also sends `X-Request-ID` and `traceparent` to the PDP.

### 2.2 The C header

49. The handles are opaque. The configuration is JSON text, not a C struct, because a new key then does not change the ABI.
50. The JSON configuration has a closed schema and no default. Each key is necessary. Dash keeps each default in its table of keys (`/home/eliot/Documents/Work/Clickhouse-dash/src/config_keys.def`), so its generated reference stays the one list.
51. The credential is a separate argument and is never in the JSON. Dash can then log the JSON, and an error never holds the secret.

```c
/* procurauth_pep.h: the C ABI of procurauth-pep. ABI version 1. */
#include <stddef.h>
#include <stdint.h>
#define PROCURAUTH_PEP_ABI_VERSION 1u

typedef struct procurauth_pep procurauth_pep;           /* shared by all threads */
typedef struct procurauth_identity procurauth_identity; /* read only */
typedef struct procurauth_outcome procurauth_outcome;   /* read only */
typedef struct procurauth_errors procurauth_errors;     /* the errors of the configuration */
typedef struct procurauth_response procurauth_response; /* the transport fills it */
typedef struct { const uint8_t *ptr; size_t len; } procurauth_bytes; /* UTF-8 text or bytes, no NUL */

typedef enum { PROCURAUTH_OK = 0, PROCURAUTH_E_NULL = 1, PROCURAUTH_E_UTF8 = 2, PROCURAUTH_E_CONFIG = 3,
               PROCURAUTH_E_ABI = 4, PROCURAUTH_E_PANIC = 5 } procurauth_status;
typedef enum { PROCURAUTH_ALLOWED = 0, PROCURAUTH_APPROVAL_PENDING = 1, PROCURAUTH_REFUSED = 2,
               PROCURAUTH_VERDICT = 3 } procurauth_kind;
typedef enum { PROCURAUTH_R_NONE = 0, PROCURAUTH_R_UNAUTHENTICATED = 1, PROCURAUTH_R_FORBIDDEN = 2,
               PROCURAUTH_R_RATE_LIMITED = 3, PROCURAUTH_R_APPROVAL_DENIED = 4, PROCURAUTH_R_APPROVAL_EXPIRED = 5,
               PROCURAUTH_R_INVALID_ARGUMENT = 6, PROCURAUTH_R_NO_DEVICE = 7, PROCURAUTH_R_APPROVAL_RATE_LIMITED = 8,
               PROCURAUTH_R_UNAVAILABLE = 9, PROCURAUTH_R_INTERNAL = 10 } procurauth_refusal;
typedef enum { PROCURAUTH_T_OK = 0, PROCURAUTH_T_NOT_SENT = 1, PROCURAUTH_T_TIMEOUT = 2, PROCURAUTH_T_LOST = 3,
               PROCURAUTH_T_MALFORMED = 4, PROCURAUTH_T_TOO_LARGE = 5 } procurauth_transport_result;

typedef struct { procurauth_bytes name, value; } procurauth_header;
typedef struct { procurauth_bytes type, id; } procurauth_segment;  /* one node below the service */
typedef struct { procurauth_bytes action; const procurauth_segment *below; size_t depth; } procurauth_permission;
typedef struct { const procurauth_permission *uses; size_t use_count;     /* 1 or more, in order (10.1) */
                 const procurauth_permission *others; size_t other_count; /* 5.5a */ } procurauth_operation;
typedef struct { procurauth_bytes op_json, view_json, approval_ref, until, supersedes; /* empty: absent */
                 uint64_t lease_s, wait_s; /* 0: absent */ } procurauth_approval_call;
typedef int (*procurauth_build_fn)(void *user, procurauth_approval_call *out); /* 0: good */

typedef struct {
  uint32_t abi_version;
  void *user;
  int (*supports_scheme)(void *user, procurauth_bytes scheme);
  procurauth_transport_result (*send)(void *user, void *call, procurauth_bytes method, procurauth_bytes url,
      const procurauth_header *headers, size_t header_count, procurauth_bytes body, uint64_t timeout_ms,
      procurauth_response *out);
  void (*set_idle_close)(void *user, uint64_t idle_ms);                /* NULL: no effect */
} procurauth_transport;

uint32_t procurauth_pep_abi_version(void);
procurauth_status procurauth_pep_new(procurauth_bytes config_json, procurauth_bytes action_document_json,
    procurauth_bytes credential, const procurauth_transport *transport, procurauth_pep **out, procurauth_errors **errors);
void procurauth_pep_free(procurauth_pep *pep);

/* values: each value of the token header; 0 values or 2 or more give a refusal (9.4). */
procurauth_status procurauth_pep_authenticate(const procurauth_pep *pep, const procurauth_bytes *values, size_t count,
    procurauth_identity **identity, procurauth_outcome **refusal);
procurauth_bytes procurauth_identity_sub(const procurauth_identity *identity);
void procurauth_identity_free(procurauth_identity *identity);

/* take_units = 1: decide (with context.usage for a use); 0: check (binding 3.5). call goes to send(). */
procurauth_status procurauth_pep_decide(const procurauth_pep *pep, const procurauth_identity *who,
    const procurauth_operation *op, int take_units, void *call, procurauth_outcome **out);
/* build runs only when the PDP asks for an approval (17.7). */
procurauth_status procurauth_pep_decide_approval(const procurauth_pep *pep, const procurauth_identity *who,
    const procurauth_operation *op, procurauth_build_fn build, void *user, void *call, procurauth_outcome **out);
procurauth_status procurauth_pep_approval_status(const procurauth_pep *pep, const procurauth_identity *who,
    procurauth_bytes request_id, int keepalive, void *call, procurauth_outcome **out);

procurauth_kind procurauth_outcome_kind(const procurauth_outcome *o);
procurauth_refusal procurauth_outcome_refusal(const procurauth_outcome *o);
uint16_t procurauth_outcome_http_status(const procurauth_outcome *o);   /* 10.3 */
procurauth_bytes procurauth_outcome_http_body(const procurauth_outcome *o); /* {"error": "<word>"} */
uint64_t procurauth_outcome_retry_after(const procurauth_outcome *o);   /* 0: no Retry-After */
uint64_t procurauth_outcome_units(const procurauth_outcome *o);
size_t procurauth_outcome_used(const procurauth_outcome *o);            /* the index in uses */
/* part 0: the use; part 1 to n: the other permissions, in order (7.5). */
size_t procurauth_outcome_ceiling_count(const procurauth_outcome *o, size_t part);
procurauth_status procurauth_outcome_ceiling(const procurauth_outcome *o, size_t part, size_t index,
    procurauth_bytes *name, uint64_t *value);
procurauth_bytes procurauth_outcome_request_id(const procurauth_outcome *o); /* APPROVAL_PENDING */
procurauth_bytes procurauth_outcome_code(const procurauth_outcome *o);       /* APPROVAL_PENDING */
procurauth_bytes procurauth_outcome_verdict(const procurauth_outcome *o);    /* VERDICT (17.6) */
procurauth_bytes procurauth_outcome_log_text(const procurauth_outcome *o);   /* never the token */
void procurauth_outcome_free(procurauth_outcome *o);

void procurauth_response_set(procurauth_response *r, uint16_t status, procurauth_bytes body);
void procurauth_response_add_header(procurauth_response *r, procurauth_bytes name, procurauth_bytes value);

size_t procurauth_errors_count(const procurauth_errors *e);
void procurauth_errors_item(const procurauth_errors *e, size_t i,
    procurauth_bytes *key, procurauth_bytes *expected, procurauth_bytes *found);
void procurauth_errors_free(procurauth_errors *e);
```

### 2.3 Rules of ownership and safety

52. The library allocates each handle. The caller frees it one time with its `_free` function. A `_free` function with `NULL` does nothing.
53. An input buffer belongs to the caller. It must stay valid until the call returns, and the library copies what it keeps.
54. The transport struct, its `user` pointer and its functions must stay valid until `procurauth_pep_free` returns. The buffers that `build` writes must stay valid until `procurauth_pep_decide_approval` returns.
55. An output buffer refers to memory in its handle. It stays valid until the handle is freed. The caller never calls `free()` on it.
56. Each text is UTF-8 with an explicit length and no NUL. A text that is not UTF-8, or that holds a NUL, gives `PROCURAUTH_E_UTF8`. A body is bytes.
57. The transport copies the answer into the library with `procurauth_response_set`. Thus no buffer moves from the C++ allocator to the Rust allocator.
58. Each exported function runs its body in `std::panic::catch_unwind` and returns `PROCURAUTH_E_PANIC` after a panic. Dash then answers HTTP 503 and writes a log line.
59. Since Rust 1.81, a panic that leaves an `extern "C"` function stops the process. A missed `catch_unwind` thus never gives undefined behaviour.
60. A C++ exception must never pass through a Rust frame. The callbacks of Dash are `noexcept`, and each one catches every exception and returns an error.
61. The `procurauth_pep` handle is shared by all threads. `Pep` holds owned data, `Mutex` values and `Arc` values of `Send + Sync` traits. A compile-time assertion in the crate proves `Send` and `Sync`.
62. The library calls `send` from many threads at the same time, so the transport must be thread-safe. An identity or an outcome can move to another thread. It is freed one time.
63. A poisoned lock does not stop the handle, because each lock of `procurauth-pep` recovers with `PoisonError::into_inner`.
64. The transport receives the credential in the header `Authorization`. It must never log a header.
65. `procurauth_pep_new` refuses a transport with another `abi_version`. Dash also compares `procurauth_pep_abi_version()` with the header at start and stops on a difference.

### 2.4 The build

66. A new crate, `crates/procurauth-pep-capi`, has `crate-type = ["staticlib", "rlib"]`. The `rlib` lets the Rust tests call the functions. The crate uses `procurauth-pep` without the feature `mcp`.
67. The crate has its own `[lints]` table. Cargo cannot change one lint under `workspace = true`, so the table copies the other lints of the workspace.
68. The table sets `unsafe_code = "deny"`, not `forbid`. Only the module `ffi` has `#![allow(unsafe_code)]`, and each `unsafe` block has a `// SAFETY:` comment. The lint `clippy::undocumented_unsafe_blocks` checks the comments.
69. The profile keeps `panic = "unwind"`, because `catch_unwind` needs it. The workspace sets no profile today.
70. The targets are the four targets of the Dash release: `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`, `x86_64-apple-darwin` and `aarch64-apple-darwin`.
71. The header is a file in the crate: `include/procurauth_pep.h`. A test runs `cbindgen` (a dev-dependency) and compares its output with the file. The code and the header then cannot differ, as with a generated configuration reference.
72. A second test starts `fake-pdp`, compiles a small C program with the header and the archive, and runs one decision. It proves the link and the rules of ownership under a sanitizer.
73. Dash builds the archive with `cargo build --locked --release --target <triple> -p procurauth-pep-capi`, at a fixed commit of ProcurAuth. Dash already pins the commit of ProcurAuth for its conformance tests.
74. CMake imports the archive as `add_library(procurauth_pep STATIC IMPORTED)` and links it to `chdash`. The command `cargo rustc --print native-static-libs` gives the system libraries to add. A new option, `CHDASH_PROCURAUTH_PEP`, selects the library or the current C++ code during the change.

### 2.5 TLS and transport

75. Option A: the client of `procurauth-pep` with its feature `tls`, on `rustls` and `aws-lc-rs`. The lot AL1 adds this feature, off by default (decisions 44 to 51 of `decisions-pep.md`). It reads the system bundle from fixed paths and an optional CA file, and it reads no environment variable. The archive of the C interface can turn it on.
76. Option A removes `pdp_transport.cpp` from Dash. The rules of 10.6 and 10.7 then run in Rust code that the cases test. But Dash keeps OpenSSL for ClickHouse, so the binary has two TLS stacks and two sets of trusted roots.
77. With option A, the library must implement the Dash keys `ca_file`, `verify` and `insecure_link` again with `rustls`. `verify = false` needs the dangerous verifier API of `rustls`. The build must give `zig cc` to `cc-rs` for `aws-lc-sys` on musl. The portal builds `aws-lc` on Alpine with `gcc`, not with `zig`.
78. Option B: a callback transport. Dash implements `send` with its current client and its static OpenSSL. The archive is then pure Rust, and the build needs only `rustup target add`.
79. Option B has a cost. The client of Dash must give the kind of each failure: `NOT_SENT` before the first byte, `LOST` after it, `TIMEOUT`. It must never send a request again, because the library owns the retries (`plain_retries`, 10.6). Today `post` sends a plain question again one time on its own.
80. Recommendation: option B. Dash already ships a static OpenSSL for ClickHouse, so a second TLS stack adds size, build risk and a second trust configuration. Option A stays possible, because the feature `tls` now exists.
81. With option B, the cases of 10.6 and 10.7 run against Dash in its conformance job. They prove that the client of Dash gives the right kind of failure.

### 2.6 What Dash still does itself

82. The MCP layer: JSON-RPC, `tools/list`, the map of a tool to an operation, `op` and `view` from the arguments, and the control arguments. Also the tool `get_action_status` and the map of a refusal to a tool error (binding 5.7a). The module `mcp` of the library is Rust only, and version 1 of the ABI does not include it.
83. Dash reads its configuration file with the rules of the owner: a closed schema, `_env` and `_file`, and secrets only through `_env` and `_file`. It also keeps `--print-config` with the source of each value, and the log of the values at start.
84. Dash gives the library typed values only, as the JSON of item 50. The library never reads an environment variable or a file.
85. The chain of each node: the type comes from the route, never from the client (4.5). The application of each ceiling to the settings of ClickHouse stays in `access_ceilings.cpp`.
86. The transport of option B, the log line `access.decision`, the metrics and Resource Search, when the owner keeps it.
87. Dash can then remove most of `access_pdp.cpp`, the PEP part of `mcp_approval.cpp`, `approval_effect.cpp` and the signature checks of the identity token. With option A, it also removes `pdp_transport.cpp`. `jwt.cpp` stays, because other files of Dash use it.

### 2.7 Changes in procurauth-pep before the ABI

88. A value for each call that reaches the transport. Dash needs it to send `X-Request-ID` and `traceparent` to the PDP, and the trait `PdpTransport` has no such value.
89. A key set from PEM text. The core already reads one key with `PublicKey::from_spki_pem`. Dash accepts PEM blocks and a JWK Set today.
90. A reader of the JSON configuration with a closed schema and no default, in the crate `procurauth-pep-capi`. It returns each error with the key, the expected value and the value found.
91. A cache bound in bytes and a deadline for a split batch. Either the library adds them, or Dash removes `cache_bytes` and `batch_timeout`.

### 2.8 Risks of Part 2

92. Two TLS stacks, with option A only: `aws-lc` puts a prefix on its symbols, so the link works with OpenSSL. The size and the second set of roots stay.
93. Size of the binary: the archive adds the Rust standard library, `serde_json`, `ed25519-dalek` and `p256`. Monocypher, micro-ecc and the C++ PEP code leave. Measure the size before and after in the first trial build.
94. musl and `aws-lc-sys`, with option A only: `cc-rs` must use `zig cc` with the same target. Check the build of `aws-lc-sys` on each of the four targets, because it compiles C code.
95. The static libc: the Rust target for musl builds `std` against its own copy of musl. A static archive does not include `libc.a`, so the final link uses the musl of Zig. Check this with `--print native-static-libs` in the first trial build.
96. The unwinder: the musl target of Rust can bring its own `libunwind`, and the `libc++` of Zig brings one too. Two copies of the `_Unwind_*` symbols can stop the link. Test this in the first trial build, before more work.
97. ABI versions: a change of a struct, or of the value of an enum, makes a new ABI version. New enum values come only at the end. The header and the archive come from the same commit.
98. Calls that block: each call blocks a thread of the pool for the timeout of the PDP at most. A call with `wait_s` blocks for 25 seconds at most. Dash must keep a bound on the number of calls that wait, for example a part of `max_worker_threads`.
99. Panics: `catch_unwind` works only with `panic = "unwind"`. A panic of the library gives HTTP 503 and a log line, and the handle stays usable (item 63).
100. Behaviour of Dash: the items of 2.7 change what Dash does. Each change goes in the release notes of Dash, and its conformance job must pass before the change.
101. Licence: Dash is MIT, and the archive is Apache-2.0. A binary of Dash must carry the Apache-2.0 text and the licences of the Rust dependencies. Option A adds the licences of `aws-lc` and `rustls`.

## Points for the owner

102. Decide the rule for two grants `access` with a budget on one service (item 14). The choices are the first grant, or the lower amount for each window.
103. Decide whether an owner with no limit (the superadmin of the portal) is part of the PDP library, or stays in the portal.
104. Decide the TLS option of Dash (items 75 to 81). Option A uses the feature `tls` of the library.
105. Decide the licence of the merged code of the portal before it depends on ProcurAuth.
106. Decide whether the core gets a feature that removes the code of tokens and keys (item 40).
