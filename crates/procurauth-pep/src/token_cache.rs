//! The kept results of the check of an identity token (9.7).
//!
//! A service MAY keep the result of a token check for that exact token
//! (9.7). This table is off by default: `token_cache_max_entries` is 0.
//!
//! - The key is the SHA-256 hash of the exact text of the token. The table
//!   holds no token text.
//! - The value is the identity that the check gave: `sub`, `owner`, `iat` and `exp`.
//! - A kept result serves while the clock, in whole seconds, is not later than
//!   `exp` plus the clock skew. This is the rule of the check of `exp` (9.4).
//!   Thus a kept result never serves a token that a new check refuses for its time.
//! - The table keeps only a result of a token that passed every check. It never
//!   keeps a refusal.
//! - When the table is full, it first deletes the results that do not serve any
//!   more. When it is still full, it keeps no new result (the rule of 10.1b).

use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard, PoisonError};

use sha2::{Digest, Sha256};

use crate::identity::Identity;

struct Entry {
    identity: Identity,
    /// The last second in which the result serves: `exp` plus the skew.
    last_second: u64,
}

/// The table of kept token results. It has its own lock.
pub(crate) struct TokenCache {
    max_entries: usize,
    skew_seconds: u64,
    map: Mutex<HashMap<[u8; 32], Entry>>,
}

fn digest(token: &str) -> [u8; 32] {
    Sha256::digest(token.as_bytes()).into()
}

impl TokenCache {
    pub(crate) fn new(max_entries: usize, skew_seconds: u64) -> Self {
        Self {
            max_entries,
            skew_seconds,
            map: Mutex::new(HashMap::new()),
        }
    }

    fn lock(&self) -> MutexGuard<'_, HashMap<[u8; 32], Entry>> {
        self.map.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The number of kept results now.
    pub(crate) fn len(&self) -> usize {
        if self.max_entries == 0 {
            return 0;
        }
        self.lock().len()
    }

    /// The kept identity for this exact token, when it still serves at `now_seconds`.
    pub(crate) fn get(&self, token: &str, now_seconds: u64) -> Option<Identity> {
        if self.max_entries == 0 {
            return None;
        }
        let key = digest(token);
        let mut map = self.lock();
        let entry = map.get(&key)?;
        if now_seconds > entry.last_second {
            map.remove(&key);
            return None;
        }
        Some(entry.identity.clone())
    }

    /// Keep the identity of a token that passed every check at `now_seconds`.
    pub(crate) fn put(&self, token: &str, identity: &Identity, now_seconds: u64) {
        if self.max_entries == 0 {
            return;
        }
        let last_second = identity.expires_at().saturating_add(self.skew_seconds);
        if now_seconds > last_second {
            return;
        }
        let key = digest(token);
        let mut map = self.lock();
        if map.len() >= self.max_entries && !map.contains_key(&key) {
            map.retain(|_, entry| now_seconds <= entry.last_second);
            if map.len() >= self.max_entries {
                return;
            }
        }
        map.insert(
            key,
            Entry {
                identity: identity.clone(),
                last_second,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::time::Duration;

    use access_token::make::{Shape, make_token};
    use procurauth_core::jws::{KeySet, SigningKey};
    use serde_json::json;

    use crate::Pep;
    use crate::clock::ManualClock;
    use crate::config::{PdpConfig, PepConfig, Secret};
    use crate::document::ActionDocumentBuilder;
    use crate::refusal::{AuthFailure, Refusal};
    use crate::transport::{HttpResponse, PdpTransport, TransportError};

    const NOW: u64 = 1_792_569_648;
    const SKEW: u64 = 30;

    fn who(sub: &str, exp: u64) -> Identity {
        Identity::new(sub.to_owned(), None, NOW - 10, exp)
    }

    #[test]
    fn p_9_7_a_result_serves_until_exp_plus_the_skew_and_never_after() {
        let cache = TokenCache::new(10, SKEW);
        cache.put("token-a", &who("alice", NOW + 100), NOW);
        assert_eq!(cache.get("token-a", NOW).unwrap().sub(), "alice");
        assert!(cache.get("token-a", NOW + 100 + SKEW).is_some());
        assert!(cache.get("token-a", NOW + 100 + SKEW + 1).is_none());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn p_9_7_a_result_serves_only_for_the_exact_token_text() {
        let cache = TokenCache::new(10, SKEW);
        cache.put("token-a", &who("alice", NOW + 100), NOW);
        assert!(cache.get("token-a ", NOW).is_none());
        assert!(cache.get("token-b", NOW).is_none());
    }

    #[test]
    fn p_9_7_zero_entries_keeps_nothing() {
        let cache = TokenCache::new(0, SKEW);
        cache.put("token-a", &who("alice", NOW + 100), NOW);
        assert!(cache.get("token-a", NOW).is_none());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn p_10_1b_a_full_table_keeps_no_new_result() {
        let cache = TokenCache::new(2, SKEW);
        cache.put("t1", &who("a", NOW + 100), NOW);
        cache.put("t2", &who("b", NOW + 100), NOW);
        cache.put("t3", &who("c", NOW + 100), NOW);
        assert_eq!(cache.len(), 2);
        assert!(cache.get("t3", NOW).is_none());
        assert!(cache.get("t1", NOW).is_some());
        // A result that does not serve any more leaves room.
        cache.put("t4", &who("d", NOW + 1_000), NOW + 100 + SKEW + 1);
        assert!(cache.get("t4", NOW + 100 + SKEW + 1).is_some());
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn the_table_holds_no_token_text() {
        let cache = TokenCache::new(10, SKEW);
        cache.put("secret-token-text", &who("alice", NOW + 100), NOW);
        let map = cache.lock();
        let key = map.keys().next().unwrap();
        assert_eq!(key, &digest("secret-token-text"));
        assert_ne!(&key[..], b"secret-token-text".as_slice());
    }

    struct NoPdp;

    impl PdpTransport for NoPdp {
        fn supports_scheme(&self, _: &str) -> bool {
            true
        }
        fn post_json(
            &self,
            _: &str,
            _: &[(String, String)],
            _: &[u8],
            _: Duration,
        ) -> Result<HttpResponse, TransportError> {
            Err(TransportError::NotSent("no PDP in this test".to_owned()))
        }
        fn get_json(
            &self,
            _: &str,
            _: &[(String, String)],
            _: Duration,
        ) -> Result<HttpResponse, TransportError> {
            Err(TransportError::NotSent("no PDP in this test".to_owned()))
        }
    }

    fn signing_key() -> SigningKey {
        SigningKey::from_ed25519_seed(&[7; 32])
    }

    fn token_at(iat: u64, exp: u64) -> String {
        let shape = Shape {
            alg: None,
            typ: None,
            kid: None,
            crit: None,
            bad_signature: false,
        };
        let claims = json!({
            "iss": "https://edge.example", "sub": "alice", "aud": "notes",
            "iat": iat, "exp": exp,
        });
        make_token(&shape, &claims, &signing_key()).token
    }

    fn token(exp: u64) -> String {
        token_at(NOW - 10, exp)
    }

    fn pep(max_entries: usize) -> (Pep, ManualClock) {
        let document = ActionDocumentBuilder::new("notes")
            .resource_type("folder", "service", "One folder.")
            .action("notes.read", &["service", "folder"], "Read notes.", &[])
            .build()
            .unwrap();
        let mut pdp = PdpConfig::new("https://pdp.example", Secret::new("credential"));
        pdp.max_connections = 1;
        let keys = KeySet::from_keys(vec![signing_key().public_key(None)]);
        let mut config = PepConfig::new(
            "notes",
            "X-Identity-Token",
            "https://edge.example",
            keys,
            300,
            SKEW,
            pdp,
        );
        config.token_cache_max_entries = max_entries;
        let clock = ManualClock::at_seconds(NOW);
        let pep =
            Pep::with_clock(config, document, Arc::new(NoPdp), Arc::new(clock.clone())).unwrap();
        (pep, clock)
    }

    fn failed_check(result: Result<Identity, Refusal>) -> String {
        match result {
            Err(Refusal::Unauthenticated(AuthFailure::Token { check, .. })) => check,
            other => panic!("expected a refused token, found {other:?}"),
        }
    }

    #[test]
    fn p_9_7_a_kept_result_avoids_a_second_signature_check() {
        let (mut pep, _) = pep(10);
        let text = token(NOW + 200);
        assert_eq!(pep.authenticate_token(&text).unwrap().sub(), "alice");
        assert_eq!(pep.kept_tokens(), 1);
        // With no trusted key, a new check fails at the signature. The kept result still serves.
        pep.keys = KeySet::default();
        assert_eq!(pep.authenticate_token(&text).unwrap().sub(), "alice");
        // Another token has no kept result: its check fails.
        assert!(pep.authenticate_token(&token(NOW + 201)).is_err());
    }

    #[test]
    fn p_9_7_without_the_table_each_token_is_checked_again() {
        let (mut pep, _) = pep(0);
        let text = token(NOW + 200);
        pep.authenticate_token(&text).unwrap();
        assert_eq!(pep.kept_tokens(), 0);
        pep.keys = KeySet::default();
        assert!(pep.authenticate_token(&text).is_err());
    }

    #[test]
    fn p_9_7_a_kept_result_never_serves_after_exp_plus_the_skew() {
        let (pep, clock) = pep(10);
        let text = token(NOW + 200);
        pep.authenticate_token(&text).unwrap();
        clock.set_seconds(NOW + 200 + SKEW);
        assert!(pep.authenticate_token(&text).is_ok());
        clock.set_seconds(NOW + 200 + SKEW + 1);
        assert_eq!(failed_check(pep.authenticate_token(&text)), "exp");
        assert_eq!(pep.kept_tokens(), 0);
    }

    #[test]
    fn p_9_7_a_token_that_fails_a_check_is_never_kept() {
        let (pep, _) = pep(10);
        let mut text = token(NOW + 200);
        text.push('x');
        assert!(pep.authenticate_token(&text).is_err());
        assert!(pep.authenticate_token(&text).is_err());
        assert_eq!(pep.kept_tokens(), 0);
        // An expired token is not kept either.
        let old = token_at(NOW - 200, NOW - 100);
        assert_eq!(failed_check(pep.authenticate_token(&old)), "exp");
        assert_eq!(pep.kept_tokens(), 0);
    }
}
