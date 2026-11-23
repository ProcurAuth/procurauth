# Third-party notices

The code of ProcurAuth is under the licence `Apache-2.0` (see `LICENSE`). The texts are under CC BY 4.0 (see
`LICENSE-TEXTS.txt`). The release programs also use the Rust crates below. Each crate keeps its own licence.

## Embedded files

The programs embed no third-party file. Only `access-conformance` embeds files: the conformance cases and the
directories under `conformance/`. These are files of this repository.

## Rust crates

The seven release programs are `keycheck`, `access-token`, `access-lint`, `fake-pdp`, `access-conformance`,
`reference-service` and `reference-edge`. The list below holds the crates that they use (`Cargo.lock` fixes each version).

The list comes from `cargo metadata --locked` for the five targets of the release workflow. It holds the normal
dependencies of the seven programs, with the procedural macros and their dependencies. It does not hold the crates
that only the build scripts or the tests use. It does not hold the crates of this workspace.

A note such as "macOS only" marks a crate that only some targets use. Each crate gives its licence expression. When
a crate offers a choice (`OR`, or the older form `/`), the choice is yours.

| Crate | Version | Licence | Repository |
| --- | --- | --- | --- |
| aho-corasick | 1.1.5 | Unlicense OR MIT | <https://github.com/BurntSushi/aho-corasick> |
| anstyle | 1.0.14 | MIT OR Apache-2.0 | <https://github.com/rust-cli/anstyle> |
| base16ct | 0.2.0 | Apache-2.0 OR MIT | <https://github.com/RustCrypto/formats/tree/master/base16ct> |
| base64 | 0.22.1 | MIT OR Apache-2.0 | <https://github.com/marshallpierce/rust-base64> |
| base64ct | 1.8.3 | Apache-2.0 OR MIT | <https://github.com/RustCrypto/formats> |
| bitflags (Linux and macOS only) | 2.13.2 | MIT OR Apache-2.0 | <https://github.com/bitflags/bitflags> |
| block-buffer | 0.10.4 | MIT OR Apache-2.0 | <https://github.com/RustCrypto/utils> |
| block2 (macOS only) | 0.6.2 | MIT | <https://github.com/madsmtm/objc2> |
| cfg-if | 1.0.5 | MIT OR Apache-2.0 | <https://github.com/rust-lang/cfg-if> |
| clap | 4.6.7 | MIT OR Apache-2.0 | <https://github.com/clap-rs/clap> |
| clap_builder | 4.6.7 | MIT OR Apache-2.0 | <https://github.com/clap-rs/clap> |
| clap_derive | 4.6.7 | MIT OR Apache-2.0 | <https://github.com/clap-rs/clap> |
| clap_lex | 1.1.1 | MIT OR Apache-2.0 | <https://github.com/clap-rs/clap> |
| const-oid | 0.9.6 | Apache-2.0 OR MIT | <https://github.com/RustCrypto/formats/tree/master/const-oid> |
| cpufeatures | 0.2.17 | MIT OR Apache-2.0 | <https://github.com/RustCrypto/utils> |
| crypto-bigint | 0.5.5 | Apache-2.0 OR MIT | <https://github.com/RustCrypto/crypto-bigint> |
| crypto-common | 0.1.6 | MIT OR Apache-2.0 | <https://github.com/RustCrypto/traits> |
| ctrlc | 3.5.2 | MIT/Apache-2.0 | <https://github.com/Detegr/rust-ctrlc> |
| curve25519-dalek | 4.1.3 | BSD-3-Clause | <https://github.com/dalek-cryptography/curve25519-dalek/tree/main/curve25519-dalek> |
| curve25519-dalek-derive (x86_64 only) | 0.1.1 | MIT/Apache-2.0 | <https://github.com/dalek-cryptography/curve25519-dalek> |
| der | 0.7.10 | Apache-2.0 OR MIT | <https://github.com/RustCrypto/formats/tree/master/der> |
| digest | 0.10.7 | MIT OR Apache-2.0 | <https://github.com/RustCrypto/traits> |
| dispatch2 (macOS only) | 0.3.1 | Zlib OR Apache-2.0 OR MIT | <https://github.com/madsmtm/objc2> |
| ecdsa | 0.16.9 | Apache-2.0 OR MIT | <https://github.com/RustCrypto/signatures/tree/master/ecdsa> |
| ed25519 | 2.2.3 | Apache-2.0 OR MIT | <https://github.com/RustCrypto/signatures/tree/master/ed25519> |
| ed25519-dalek | 2.2.0 | BSD-3-Clause | <https://github.com/dalek-cryptography/curve25519-dalek/tree/main/ed25519-dalek> |
| elliptic-curve | 0.13.8 | Apache-2.0 OR MIT | <https://github.com/RustCrypto/traits/tree/master/elliptic-curve> |
| ff | 0.13.1 | MIT/Apache-2.0 | <https://github.com/zkcrypto/ff> |
| generic-array | 0.14.9 | MIT | <https://github.com/fizyk20/generic-array> |
| getrandom | 0.2.17 | MIT OR Apache-2.0 | <https://github.com/rust-random/getrandom> |
| group | 0.13.0 | MIT/Apache-2.0 | <https://github.com/zkcrypto/group> |
| heck | 0.5.0 | MIT OR Apache-2.0 | <https://github.com/withoutboats/heck> |
| hex | 0.4.3 | MIT OR Apache-2.0 | <https://github.com/KokaKiwi/rust-hex> |
| hmac | 0.12.1 | MIT OR Apache-2.0 | <https://github.com/RustCrypto/MACs> |
| itoa | 1.0.18 | MIT OR Apache-2.0 | <https://github.com/dtolnay/itoa> |
| libc (Linux and macOS only) | 0.2.191 | MIT OR Apache-2.0 | <https://github.com/rust-lang/libc> |
| memchr | 2.8.3 | Unlicense OR MIT | <https://github.com/BurntSushi/memchr> |
| nix (Linux and macOS only) | 0.31.3 | MIT | <https://github.com/nix-rust/nix> |
| objc2 (macOS only) | 0.6.5 | MIT | <https://github.com/madsmtm/objc2> |
| objc2-encode (macOS only) | 4.1.0 | MIT | <https://github.com/madsmtm/objc2> |
| p256 | 0.13.2 | Apache-2.0 OR MIT | <https://github.com/RustCrypto/elliptic-curves/tree/master/p256> |
| pem-rfc7468 | 0.7.0 | Apache-2.0 OR MIT | <https://github.com/RustCrypto/formats/tree/master/pem-rfc7468> |
| pkcs8 | 0.10.2 | Apache-2.0 OR MIT | <https://github.com/RustCrypto/formats/tree/master/pkcs8> |
| primeorder | 0.13.6 | Apache-2.0 OR MIT | <https://github.com/RustCrypto/elliptic-curves/tree/master/primeorder> |
| proc-macro2 | 1.0.107 | MIT OR Apache-2.0 | <https://github.com/dtolnay/proc-macro2> |
| quote | 1.0.47 | MIT OR Apache-2.0 | <https://github.com/dtolnay/quote> |
| rand_core | 0.6.4 | MIT OR Apache-2.0 | <https://github.com/rust-random/rand> |
| regex | 1.13.1 | MIT OR Apache-2.0 | <https://github.com/rust-lang/regex> |
| regex-automata | 0.4.18 | MIT OR Apache-2.0 | <https://github.com/rust-lang/regex> |
| regex-syntax | 0.8.11 | MIT OR Apache-2.0 | <https://github.com/rust-lang/regex> |
| rfc6979 | 0.4.0 | Apache-2.0 OR MIT | <https://github.com/RustCrypto/signatures/tree/master/rfc6979> |
| sec1 | 0.7.3 | Apache-2.0 OR MIT | <https://github.com/RustCrypto/formats/tree/master/sec1> |
| serde | 1.0.229 | MIT OR Apache-2.0 | <https://github.com/serde-rs/serde> |
| serde_core | 1.0.229 | MIT OR Apache-2.0 | <https://github.com/serde-rs/serde> |
| serde_derive | 1.0.229 | MIT OR Apache-2.0 | <https://github.com/serde-rs/serde> |
| serde_json | 1.0.154 | MIT OR Apache-2.0 | <https://github.com/serde-rs/json> |
| sha2 | 0.10.9 | MIT OR Apache-2.0 | <https://github.com/RustCrypto/hashes> |
| signature | 2.2.0 | Apache-2.0 OR MIT | <https://github.com/RustCrypto/traits/tree/master/signature> |
| spki | 0.7.3 | Apache-2.0 OR MIT | <https://github.com/RustCrypto/formats/tree/master/spki> |
| strsim | 0.11.1 | MIT | <https://github.com/rapidfuzz/strsim-rs> |
| subtle | 2.6.1 | BSD-3-Clause | <https://github.com/dalek-cryptography/subtle> |
| syn (x86_64 only) | 2.0.119 | MIT OR Apache-2.0 | <https://github.com/dtolnay/syn> |
| syn | 3.0.9 | MIT OR Apache-2.0 | <https://github.com/dtolnay/syn> |
| thiserror | 2.0.21 | MIT OR Apache-2.0 | <https://github.com/dtolnay/thiserror> |
| thiserror-impl | 2.0.21 | MIT OR Apache-2.0 | <https://github.com/dtolnay/thiserror> |
| typenum | 1.20.1 | MIT OR Apache-2.0 | <https://github.com/paholg/typenum> |
| unicode-ident | 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 | <https://github.com/dtolnay/unicode-ident> |
| windows-link (Windows only) | 0.2.1 | MIT OR Apache-2.0 | <https://github.com/microsoft/windows-rs> |
| windows-sys (Windows only) | 0.61.2 | MIT OR Apache-2.0 | <https://github.com/microsoft/windows-rs> |
| zeroize | 1.9.1 | Apache-2.0 OR MIT | <https://github.com/RustCrypto/utils> |
| zmij | 1.0.23 | MIT | <https://github.com/dtolnay/zmij> |
